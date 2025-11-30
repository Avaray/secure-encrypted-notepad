use eframe::egui;
use std::path::PathBuf;
use zeroize::Zeroizing;

use crate::crypto::{encrypt_file, decrypt_file, generate_keyfile};
use crate::history::{create_snapshot, list_versions, load_version, restore_version, delete_version, cleanup_old_versions, get_history_stats, VersionInfo};
use crate::settings::Settings;

/// Application state
pub struct EditorApp {
    /// Text editor content
    text_content: String,
    
    /// Password for encryption/decryption
    password: String,
    
    /// Path to keyfile
    keyfile_path: Option<PathBuf>,
    
    /// Path to currently open file
    current_file_path: Option<PathBuf>,
    
    /// List of available versions in history
    versions: Vec<VersionInfo>,
    
    /// Status message
    status_message: String,
    
    /// User preferences
    settings: Settings,
    
    /// Show password dialog (for Open)
    show_password_dialog: bool,
    
    /// Pending path for file to open
    pending_open_path: Option<PathBuf>,
    
    /// Show Settings panel
    show_settings_panel: bool,
    
    /// Show History panel
    show_history_panel: bool,
    
    /// Document has been modified
    is_modified: bool,
    
    /// Currently previewed version (if any)
    preview_version: Option<(VersionInfo, String)>,
    
    /// Show restore confirmation dialog
    show_restore_confirm: Option<VersionInfo>,
    
    /// Show delete confirmation dialog
    show_delete_confirm: Option<VersionInfo>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let settings = Settings::load();
        
        Self {
            text_content: String::new(),
            password: String::new(),
            keyfile_path: settings.last_keyfile_path.clone(),
            current_file_path: None,
            versions: Vec::new(),
            status_message: "Ready".to_string(),
            settings,
            show_password_dialog: false,
            pending_open_path: None,
            show_settings_panel: false,
            show_history_panel: false,
            is_modified: false,
            preview_version: None,
            show_restore_confirm: None,
            show_delete_confirm: None,
        }
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // FIX: 'mut' nie jest potrzebny, apply_theme bierze &self
        let app = Self::default();
        app.apply_theme(&cc.egui_ctx);
        app
    }
    
    fn apply_theme(&self, ctx: &egui::Context) {
        if self.settings.dark_theme {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
    
    /// Refresh list of versions from history
    fn refresh_versions(&mut self) {
        if let Some(path) = &self.current_file_path {
            match list_versions(path) {
                Ok(versions) => {
                    self.versions = versions;
                }
                Err(e) => {
                    eprintln!("Failed to refresh versions: {}", e);
                }
            }
        }
    }
    
    fn new_document(&mut self) {
        if self.is_modified {
            // TODO: Dialog "Save changes?"
        }
        
        self.text_content.clear();
        self.password.clear();
        self.current_file_path = None;
        self.versions.clear();
        self.is_modified = false;
        self.preview_version = None;
        self.status_message = "New document created".to_string();
    }
    
    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Encrypted Document", &["sed"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.pending_open_path = Some(path);
            self.show_password_dialog = true;
            self.password.clear();
            self.status_message = "Enter password to decrypt".to_string();
        }
    }
    
    fn open_file_with_password(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: Select keyfile first".to_string();
            return;
        }
        
        if self.password.is_empty() {
            self.status_message = "Error: Password required".to_string();
            return;
        }
        
        // FIX: Sklonuj ścieżki, aby uniknąć borrowowania self w 'if let'
        let pending_path = self.pending_open_path.clone();
        let keyfile_path_clone = self.keyfile_path.clone();

        if let (Some(path), Some(keyfile)) = (pending_path, keyfile_path_clone) {
            let password = Zeroizing::new(self.password.clone());
            
            match decrypt_file(&password, &keyfile, &path) {
                Ok(content) => {
                    self.text_content = content;
                    self.current_file_path = Some(path.clone());
                    self.is_modified = false;
                    self.show_password_dialog = false;
                    self.pending_open_path = None;
                    
                    // Wczytaj historię wersji
                    self.refresh_versions();
                    
                    let version_count = self.versions.len();
                    self.status_message = format!(
                        "✓ File opened: {} ({} versions)",
                        path.display(),
                        version_count
                    );
                    
                    if self.settings.remember_keyfile_path {
                        self.settings.last_keyfile_path = Some(keyfile.clone());
                        let _ = self.settings.save();
                    }
                }
                Err(e) => {
                    self.status_message = format!("✗ Error: {}", e);
                }
            }
        }
    }
    
    fn save_file(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: Select keyfile first".to_string();
            return;
        }
        
        if self.password.is_empty() {
            self.status_message = "Error: Password required".to_string();
            return;
        }
        
        if let Some(path) = &self.current_file_path {
            self.perform_save(path.clone());
        } else {
            self.save_file_as();
        }
    }
    
    fn save_file_as(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: Select keyfile first".to_string();
            return;
        }
        
        if self.password.is_empty() {
            self.status_message = "Error: Password required".to_string();
            return;
        }
        
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Encrypted Document", &["sed"])
            .set_file_name("document.sed")
            .save_file()
        {
            self.perform_save(path);
        }
    }
    
    fn perform_save(&mut self, path: PathBuf) {
        // FIX: Klonujemy keyfile na starcie, aby nie trzymać referencji do self
        let keyfile = if let Some(k) = &self.keyfile_path {
            k.clone()
        } else {
            return; // Should happen checked before
        };

        let password = Zeroizing::new(self.password.clone());
        
        // 1. Zapisz główny plik
        match encrypt_file(&self.text_content, &password, &keyfile, &path) {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                
                // 2. Utwórz snapshot jeśli auto_snapshot włączone
                if self.settings.auto_snapshot_on_save {
                    match create_snapshot(&self.text_content, &password, &keyfile, &path, None) {
                        Ok(_) => {
                            self.refresh_versions();
                            self.status_message = format!(
                                "✓ Saved: {} (snapshot created, {} total versions)",
                                path.display(),
                                self.versions.len()
                            );
                        }
                        Err(e) => {
                            self.status_message = format!(
                                "⚠ File saved but snapshot failed: {}",
                                e
                            );
                        }
                    }
                } else {
                    self.status_message = format!("✓ File saved: {}", path.display());
                }
                
                if self.settings.remember_keyfile_path {
                    self.settings.last_keyfile_path = Some(keyfile.clone());
                    let _ = self.settings.save();
                }
            }
            Err(e) => {
                self.status_message = format!("✗ Error saving: {}", e);
            }
        }
    }
    
    fn select_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Key Files", &["key"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.keyfile_path = Some(path.clone());
            self.status_message = format!("Keyfile selected: {}", path.display());
            
            if self.settings.remember_keyfile_path {
                self.settings.last_keyfile_path = Some(path);
                let _ = self.settings.save();
            }
        }
    }
    
    fn generate_new_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Key Files", &["key"])
            .set_file_name("my.key")
            .save_file()
        {
            match generate_keyfile(&path) {
                Ok(_) => {
                    self.keyfile_path = Some(path.clone());
                    self.status_message = format!("✓ Keyfile generated: {}", path.display());
                    
                    if self.settings.remember_keyfile_path {
                        self.settings.last_keyfile_path = Some(path);
                        let _ = self.settings.save();
                    }
                }
                Err(e) => {
                    self.status_message = format!("✗ Error generating keyfile: {}", e);
                }
            }
        }
    }
    
    /// Preview wybranej wersji (read-only)
    fn preview_version(&mut self, version: &VersionInfo) {
        if let Some(keyfile) = &self.keyfile_path {
            let password = Zeroizing::new(self.password.clone());
            
            match load_version(version, &password, keyfile) {
                Ok(content) => {
                    self.preview_version = Some((version.clone(), content));
                    self.status_message = format!(
                        "📄 Viewing version: {}",
                        version.display_timestamp()
                    );
                }
                Err(e) => {
                    self.status_message = format!("✗ Error loading version: {}", e);
                }
            }
        }
    }
    
    /// Przywróć wybraną wersję
    fn restore_version_confirmed(&mut self, version: &VersionInfo) {
        if let (Some(keyfile), Some(path)) = (&self.keyfile_path, &self.current_file_path) {
            let password = Zeroizing::new(self.password.clone());
            
            match restore_version(version, &password, keyfile, path, true) {
                Ok(_) => {
                    // Wczytaj przywróconą treść
                    match decrypt_file(&password, keyfile, path) {
                        Ok(content) => {
                            self.text_content = content;
                            self.is_modified = false;
                            self.refresh_versions();
                            self.status_message = format!(
                                "✓ Version restored: {}",
                                version.display_timestamp()
                            );
                        }
                        Err(e) => {
                            self.status_message = format!("✗ Error reloading: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("✗ Error restoring: {}", e);
                }
            }
        }
        
        self.show_restore_confirm = None;
    }
    
    /// Usuń wybraną wersję
    fn delete_version_confirmed(&mut self, version: &VersionInfo) {
        match delete_version(version) {
            Ok(_) => {
                self.refresh_versions();
                self.status_message = format!(
                    "✓ Version deleted: {}",
                    version.display_timestamp()
                );
            }
            Err(e) => {
                self.status_message = format!("✗ Error deleting: {}", e);
            }
        }
        
        self.show_delete_confirm = None;
    }
    
    /// Cleanup starych wersji
    fn cleanup_old_versions(&mut self) {
        if let Some(path) = &self.current_file_path {
            let retention_days = self.settings.snapshot_retention_days;
            
            match cleanup_old_versions(path, retention_days) {
                Ok(count) => {
                    self.refresh_versions();
                    self.status_message = format!(
                        "✓ Cleaned up {} old version(s) (older than {} days)",
                        count,
                        retention_days
                    );
                }
                Err(e) => {
                    self.status_message = format!("✗ Cleanup error: {}", e);
                }
            }
        }
    }
    
    fn render_password_dialog(&mut self, ctx: &egui::Context) {
        let mut open_confirmed = false;
        let mut open_cancelled = false;

        egui::Window::new("🔐 Enter Password")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Enter password and keyfile to decrypt:");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .desired_width(250.0)
                    );
                    
                    if self.show_password_dialog {
                        response.request_focus();
                    }
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        open_confirmed = true;
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Keyfile:");
                    if let Some(path) = &self.keyfile_path {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                    } else {
                        ui.label("❌ Not selected");
                    }
                    
                    if ui.button("Select").clicked() {
                        self.select_keyfile();
                    }
                });
                
                ui.add_space(10.0);
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("✓ Open").clicked() {
                        open_confirmed = true;
                    }
                    
                    if ui.button("✗ Cancel").clicked() {
                        open_cancelled = true;
                    }
                });
            });
            
        if open_confirmed {
            self.open_file_with_password();
        }
        
        if open_cancelled {
            self.show_password_dialog = false;
            self.pending_open_path = None;
            self.password.clear();
            self.status_message = "Open cancelled".to_string();
        }
    }
    
    fn render_settings_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙️ Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let changed = ui.selectable_value(&mut self.settings.dark_theme, true, "🌙 Dark").clicked()
                        || ui.selectable_value(&mut self.settings.dark_theme, false, "☀️ Light").clicked();
                    
                    if changed {
                        self.apply_theme(ctx);
                        let _ = self.settings.save();
                    }
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui.add(egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0).suffix(" px")).changed() {
                        self.settings.validate_font_size();
                        let _ = self.settings.save();
                    }
                });
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Font Family:");
                    egui::ComboBox::from_id_salt("font_family")
                        .selected_text(&self.settings.font_family)
                        .show_ui(ui, |ui| {
                            if ui.selectable_value(&mut self.settings.font_family, "Monospace".to_string(), "Monospace").clicked() {
                                let _ = self.settings.save();
                            }
                            if ui.selectable_value(&mut self.settings.font_family, "Proportional".to_string(), "Proportional").clicked() {
                                let _ = self.settings.save();
                            }
                        });
                });
                
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Version Control");
                ui.add_space(10.0);
                
                if ui.checkbox(&mut self.settings.auto_snapshot_on_save, "Auto-snapshot on save").changed() {
                    let _ = self.settings.save();
                }
                ui.label("📸 Create automatic snapshots when saving");
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    ui.label("Retention:");
                    if ui.add(egui::Slider::new(&mut self.settings.snapshot_retention_days, 0..=365).suffix(" days")).changed() {
                        self.settings.validate_retention_days();
                        let _ = self.settings.save();
                    }
                });
                ui.label("🗑️ 0 = keep forever");
                
                ui.add_space(20.0);
                ui.separator();
                ui.heading("Security");
                ui.add_space(10.0);
                
                if ui.checkbox(&mut self.settings.remember_keyfile_path, "Remember keyfile path").changed() {
                    if !self.settings.remember_keyfile_path {
                        self.settings.last_keyfile_path = None;
                    }
                    let _ = self.settings.save();
                }
                
                ui.label("⚠️ Only path stored, not content");
                
                ui.add_space(20.0);
                
                if ui.button("Close").clicked() {
                    self.show_settings_panel = false;
                }
            });
    }
    
    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("📜 Version History");
            ui.add_space(5.0);
            
            // Statystyki
            if let Some(path) = &self.current_file_path {
                if let Ok(stats) = get_history_stats(path) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Versions: {}", stats.total_versions));
                        ui.separator();
                        let size_mb = stats.total_size_bytes as f64 / (1024.0 * 1024.0);
                        ui.label(format!("Total: {:.1} MB", size_mb));
                    });
                    ui.add_space(5.0);
                }
            }
            
            ui.separator();
            
            // Buttony akcji
            ui.horizontal(|ui| {
                if ui.button("🗑️ Cleanup Old").clicked() {
                    self.cleanup_old_versions();
                }
                
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_versions();
                }
            });
            
            ui.add_space(10.0);
            ui.separator();
            
            // Lista wersji
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.versions.is_empty() {
                    ui.label("No versions yet. Save to create first snapshot.");
                } else {
                    // FIX: Klonujemy listę wersji, aby móc wywoływać metody self w pętli
                    let versions = self.versions.clone();
                    for version in &versions {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("📅");
                                ui.label(version.display_timestamp());
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("💾");
                                ui.label(version.display_size());
                            });
                            
                            if let Some(comment) = &version.comment {
                                ui.horizontal(|ui| {
                                    ui.label("💬");
                                    ui.label(comment);
                                });
                            }
                            
                            ui.horizontal(|ui| {
                                if ui.button("👁 View").clicked() {
                                    self.preview_version(version);
                                }
                                
                                if ui.button("↩️ Restore").clicked() {
                                    self.show_restore_confirm = Some(version.clone());
                                }
                                
                                if ui.button("🗑️ Delete").clicked() {
                                    self.show_delete_confirm = Some(version.clone());
                                }
                            });
                        });
                        
                        ui.add_space(5.0);
                    }
                }
            });
        });
    }
    
    fn render_preview_window(&mut self, ctx: &egui::Context) {
        // FIX: Klonujemy dane preview lokalnie, aby uwolnić 'self'
        let preview_data = self.preview_version.clone();
        let mut close_clicked = false;
        
        if let Some((version, content)) = preview_data {
            let mut content_clone = content.clone();
            
            egui::Window::new(format!("📄 Preview: {}", version.display_timestamp()))
                .resizable(true)
                .default_width(800.0)
                .default_height(600.0)
                .show(ctx, |ui| {
                    ui.label("⚠️ Read-only preview");
                    ui.separator();
                    
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut content_clone)
                                .desired_width(f32::INFINITY)
                                .interactive(false)
                        );
                    });
                    
                    if ui.button("Close").clicked() {
                        close_clicked = true;
                    }
                });
        }
        
        if close_clicked {
            self.preview_version = None;
        }
    }
    
    fn render_restore_confirm(&mut self, ctx: &egui::Context) {
        // FIX: Klonujemy informację o wersji lokalnie
        let restore_data = self.show_restore_confirm.clone();
        let mut do_restore = false;
        let mut do_cancel = false;
        
        if let Some(version) = restore_data {
            let version_clone = version.clone();
            
            egui::Window::new("⚠️ Confirm Restore")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Restore this version?");
                    ui.label(format!("📅 {}", version.display_timestamp()));
                    ui.add_space(10.0);
                    ui.label("⚠️ Current version will be backed up");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("✓ Restore").clicked() {
                            do_restore = true;
                        }
                        
                        if ui.button("✗ Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            
            if do_restore {
                self.restore_version_confirmed(&version_clone);
            }
        }
        
        if do_cancel {
            self.show_restore_confirm = None;
        }
    }
    
    fn render_delete_confirm(&mut self, ctx: &egui::Context) {
        // FIX: Klonujemy informację o wersji lokalnie
        let delete_data = self.show_delete_confirm.clone();
        let mut do_delete = false;
        let mut do_cancel = false;
        
        if let Some(version) = delete_data {
            let version_clone = version.clone();
            
            egui::Window::new("⚠️ Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Delete this version permanently?");
                    ui.label(format!("📅 {}", version.display_timestamp()));
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, "⚠️ This cannot be undone");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        if ui.button("🗑️ Delete").clicked() {
                            do_delete = true;
                        }
                        
                        if ui.button("✗ Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });
            
            if do_delete {
                self.delete_version_confirmed(&version_clone);
            }
        }
        
        if do_cancel {
            self.show_delete_confirm = None;
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Dialogi
        if self.show_password_dialog {
            self.render_password_dialog(ctx);
        }
        
        if self.show_settings_panel {
            self.render_settings_panel(ctx);
        }
        
        if self.preview_version.is_some() {
            self.render_preview_window(ctx);
        }
        
        if self.show_restore_confirm.is_some() {
            self.render_restore_confirm(ctx);
        }
        
        if self.show_delete_confirm.is_some() {
            self.render_delete_confirm(ctx);
        }
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New").clicked() {
                        self.new_document();
                        ui.close_menu();
                    }
                    if ui.button("📂 Open").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save").clicked() {
                        self.save_file();
                        ui.close_menu();
                    }
                    if ui.button("💾 Save As").clicked() {
                        self.save_file_as();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("❌ Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                
                ui.menu_button("Security", |ui| {
                    if ui.button("🔑 Select Keyfile").clicked() {
                        self.select_keyfile();
                        ui.close_menu();
                    }
                    if ui.button("✨ Generate Keyfile").clicked() {
                        self.generate_new_keyfile();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("History", |ui| {
                    if ui.button("📜 Toggle Panel").clicked() {
                        self.show_history_panel = !self.show_history_panel;
                        ui.close_menu();
                    }
                    if ui.button("🗑️ Cleanup Old").clicked() {
                        self.cleanup_old_versions();
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Settings", |ui| {
                    if ui.button("⚙️ Preferences").clicked() {
                        self.show_settings_panel = true;
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Help", |ui| {
                    if ui.button("ℹ️ About").clicked() {
                        self.status_message = "SED v3.0 - With Version Control".to_string();
                        ui.close_menu();
                    }
                });
            });
        });
        
        // Auth panel
        egui::TopBottomPanel::top("auth_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔒 Authentication");
                ui.separator();
                
                ui.label("Password:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text("Enter password")
                        .desired_width(200.0)
                );
                
                ui.separator();
                
                ui.label("Keyfile:");
                if let Some(path) = &self.keyfile_path {
                    ui.label(format!("✓ {}", path.file_name().unwrap_or_default().to_string_lossy()));
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Not selected");
                }
                
                if ui.button("Select").clicked() {
                    self.select_keyfile();
                }
                
                if ui.button("Generate").clicked() {
                    self.generate_new_keyfile();
                }
            });
        });
        
        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.label(&self.status_message);
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(path) = &self.current_file_path {
                        ui.label(format!(
                            "📁 {}{}",
                            path.display(),
                            if self.is_modified { " *" } else { "" }
                        ));
                    } else {
                        ui.label("📄 Unsaved");
                    }
                    
                    ui.separator();
                    
                    if self.keyfile_path.is_some() && !self.password.is_empty() {
                        ui.colored_label(egui::Color32::GREEN, "🔐 Ready");
                    } else {
                        ui.colored_label(egui::Color32::RED, "🔓 Incomplete");
                    }
                });
            });
        });
        
        // Layout: History panel (opcjonalny) + Central editor
        egui::SidePanel::right("history_panel")
            .resizable(true)
            .default_width(300.0)
            .show_animated(ctx, self.show_history_panel, |ui| {
                self.render_history_panel(ui);
            });
        
        // Central panel - Editor
        egui::CentralPanel::default().show(ctx, |ui| {
            let style = (*ctx.style()).clone();
            
            let font_id = if self.settings.font_family == "Monospace" {
                egui::FontId::monospace(self.settings.font_size)
            } else {
                egui::FontId::proportional(self.settings.font_size)
            };
            
            let mut updated_style = style;
            updated_style.text_styles.insert(egui::TextStyle::Body, font_id.clone());
            updated_style.text_styles.insert(egui::TextStyle::Monospace, font_id.clone());
            ctx.set_style(updated_style);
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                let text_edit = egui::TextEdit::multiline(&mut self.text_content)
                    .desired_width(f32::INFINITY)
                    .desired_rows(40)
                    .code_editor();
                
                if ui.add(text_edit).changed() {
                    self.is_modified = true;
                }
            });
        });
    }
}
