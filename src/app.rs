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

    /// Show alert when trying to save without auth
    show_missing_auth_alert: bool,
}

impl Default for EditorApp {
    fn default() -> Self {
        let settings = Settings::load();
        
        // --- NOWA LOGIKA: Ładowanie klucza początkowego ---
        // Priorytet 1: Globalny domyślny klucz (jeśli opcja włączona i ścieżka istnieje)
        // Priorytet 2: Ostatnio używany klucz (jeśli opcja "Remember" włączona)
        let initial_keyfile = if settings.use_default_keyfile && settings.default_keyfile_path.is_some() {
            settings.default_keyfile_path.clone()
        } else if settings.remember_keyfile_path {
            settings.last_keyfile_path.clone()
        } else {
            None
        };

        let status = if let Some(path) = &initial_keyfile {
            format!("Ready. Loaded keyfile: {}", path.file_name().unwrap_or_default().to_string_lossy())
        } else {
            "Ready.".to_string()
        };
        
        Self {
            text_content: String::new(),
            password: String::new(),
            keyfile_path: initial_keyfile,
            current_file_path: None,
            versions: Vec::new(),
            status_message: status,
            settings,
            show_password_dialog: false,
            pending_open_path: None,
            show_settings_panel: false,
            show_history_panel: false,
            is_modified: false,
            preview_version: None,
            show_restore_confirm: None,
            show_delete_confirm: None,
            show_missing_auth_alert: false,
        }
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
            // TODO: W przyszłości dialog "Czy zapisać zmiany?"
        }
        
        self.text_content.clear();
        
        // Nie czyścimy klucza (keyfile_path), bo użytkownik może chcieć użyć tego samego (zwłaszcza domyślnego).
        // Ale czyścimy hasło dla bezpieczeństwa przy nowym dokumencie.
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
                    
                    self.refresh_versions();
                    
                    let version_count = self.versions.len();
                    self.status_message = format!(
                        "✓ File opened: {} ({} versions)",
                        path.display(),
                        version_count
                    );
                    
                    // Zapamiętaj jako "Last Used", jeśli włączone w opcjach
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
        // Sprawdzenie czy mamy klucz i hasło. Jeśli nie -> Alert.
        if self.keyfile_path.is_none() || self.password.is_empty() {
            self.show_missing_auth_alert = true;
            return;
        }
        
        if let Some(path) = &self.current_file_path {
            self.perform_save(path.clone());
        } else {
            self.save_file_as();
        }
    }
    
    fn save_file_as(&mut self) {
        if self.keyfile_path.is_none() || self.password.is_empty() {
            self.show_missing_auth_alert = true;
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
        let keyfile = if let Some(k) = &self.keyfile_path {
            k.clone()
        } else {
            return; 
        };

        let password = Zeroizing::new(self.password.clone());
        
        match encrypt_file(&self.text_content, &password, &keyfile, &path) {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                
                if self.settings.auto_snapshot_on_save {
                    match create_snapshot(&self.text_content, &password, &keyfile, &path, None) {
                        Ok(_) => {
                            self.refresh_versions();
                            self.status_message = format!(
                                "✓ Saved: {} (snapshot created, {} versions)",
                                path.display(),
                                self.versions.len()
                            );
                        }
                        Err(e) => {
                            self.status_message = format!("⚠ Saved, but snapshot failed: {}", e);
                        }
                    }
                } else {
                    self.status_message = format!("✓ File saved: {}", path.display());
                }
                
                // Aktualizacja ustawień last_keyfile_path
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
    
    fn preview_version(&mut self, version: &VersionInfo) {
        if let Some(keyfile) = &self.keyfile_path {
            let password = Zeroizing::new(self.password.clone());
            
            match load_version(version, &password, keyfile) {
                Ok(content) => {
                    self.preview_version = Some((version.clone(), content));
                    self.status_message = format!("📄 Viewing version: {}", version.display_timestamp());
                }
                Err(e) => {
                    self.status_message = format!("✗ Error loading version: {}", e);
                }
            }
        }
    }
    
    fn restore_version_confirmed(&mut self, version: &VersionInfo) {
        if let (Some(keyfile), Some(path)) = (&self.keyfile_path, &self.current_file_path) {
            let password = Zeroizing::new(self.password.clone());
            
            match restore_version(version, &password, keyfile, path, true) {
                Ok(_) => {
                    match decrypt_file(&password, keyfile, path) {
                        Ok(content) => {
                            self.text_content = content;
                            self.is_modified = false;
                            self.refresh_versions();
                            self.status_message = format!("✓ Restored: {}", version.display_timestamp());
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
    
    fn delete_version_confirmed(&mut self, version: &VersionInfo) {
        match delete_version(version) {
            Ok(_) => {
                self.refresh_versions();
                self.status_message = format!("✓ Deleted: {}", version.display_timestamp());
            }
            Err(e) => {
                self.status_message = format!("✗ Error deleting: {}", e);
            }
        }
        self.show_delete_confirm = None;
    }
    
    fn cleanup_old_versions(&mut self) {
        if let Some(path) = &self.current_file_path {
            let retention_days = self.settings.snapshot_retention_days;
            match cleanup_old_versions(path, retention_days) {
                Ok(count) => {
                    self.refresh_versions();
                    self.status_message = format!("✓ Cleaned up {} old versions", count);
                }
                Err(e) => {
                    self.status_message = format!("✗ Cleanup error: {}", e);
                }
            }
        }
    }
    
    // --- UI RENDERERS ---
    
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
                    let response = ui.add(egui::TextEdit::singleline(&mut self.password).password(true).desired_width(250.0));
                    if self.show_password_dialog { response.request_focus(); }
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) { open_confirmed = true; }
                });
                ui.horizontal(|ui| {
                    ui.label("Keyfile:");
                    if let Some(path) = &self.keyfile_path {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy().to_string());
                    } else {
                        ui.label("❌ Not selected");
                    }
                    if ui.button("Select").clicked() { self.select_keyfile(); }
                });
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("✓ Open").clicked() { open_confirmed = true; }
                    if ui.button("✗ Cancel").clicked() { open_cancelled = true; }
                });
            });
            
        if open_confirmed { self.open_file_with_password(); }
        if open_cancelled {
            self.show_password_dialog = false;
            self.pending_open_path = None;
            self.password.clear();
            self.status_message = "Open cancelled".to_string();
        }
    }

    fn render_auth_alert(&mut self, ctx: &egui::Context) {
        if self.show_missing_auth_alert {
            egui::Window::new("⚠️ Cannot Save")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, "Authentication Missing!");
                    ui.add_space(5.0);
                    ui.label("To save a file, you must:");
                    ui.label("1. Select a Keyfile");
                    ui.label("2. Enter a Password");
                    ui.add_space(10.0);
                    
                    if ui.button("OK, I'll do that").clicked() {
                        self.show_missing_auth_alert = false;
                    }
                });
        }
    }
    
    // --- UAKTUALNIONY PANEL USTAWIEŃ ---
    fn render_settings_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙️ Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(450.0) // Nieco szersze dla ścieżek
            .show(ctx, |ui| {
                // SEKCJA WYGLĄDU
                ui.heading("Appearance");
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let changed = ui.selectable_value(&mut self.settings.dark_theme, true, "🌙 Dark").clicked()
                        || ui.selectable_value(&mut self.settings.dark_theme, false, "☀️ Light").clicked();
                    if changed {
                        self.apply_theme(ctx);
                        let _ = self.settings.save();
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui.add(egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0).suffix(" px")).changed() {
                        self.settings.validate_font_size();
                        let _ = self.settings.save();
                    }
                });

                ui.separator();
                
                // --- NOWA SEKCJA: GLOBALNY KLUCZ DOMYŚLNY ---
                ui.heading("Global Default Keyfile");
                ui.label("Define a keyfile to be loaded automatically on startup.");
                
                // Checkbox włączający funkcję
                if ui.checkbox(&mut self.settings.use_default_keyfile, "Always use this keyfile globally").changed() {
                    let _ = self.settings.save();
                }
                
                // Wyświetlanie aktualnej ścieżki
                ui.horizontal(|ui| {
                    ui.label("Current Default:");
                    if let Some(path) = &self.settings.default_keyfile_path {
                        ui.label(egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy()).strong());
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "None set");
                    }
                });
                
                // Przyciski wyboru / czyszczenia
                ui.horizontal(|ui| {
                    if ui.button("📂 Select Default Keyfile").clicked() {
                         if let Some(path) = rfd::FileDialog::new().add_filter("Key Files", &["key"]).pick_file() {
                             self.settings.default_keyfile_path = Some(path.clone());
                             
                             // UX: Jeśli włączono opcję globalną, od razu podmień też aktywny klucz w sesji
                             if self.settings.use_default_keyfile {
                                 self.keyfile_path = Some(path);
                                 self.status_message = "Default keyfile updated and applied.".to_string();
                             }
                             
                             let _ = self.settings.save();
                         }
                    }
                    
                    if ui.button("🗑️ Clear").clicked() {
                        self.settings.default_keyfile_path = None;
                        let _ = self.settings.save();
                    }
                });

                ui.separator();

                // SEKCJA HISTORII
                ui.heading("Version Control");
                if ui.checkbox(&mut self.settings.auto_snapshot_on_save, "Auto-snapshot on save").changed() {
                    let _ = self.settings.save();
                }
                ui.horizontal(|ui| {
                    ui.label("Retention:");
                    if ui.add(egui::Slider::new(&mut self.settings.snapshot_retention_days, 0..=365).suffix(" days")).changed() {
                        self.settings.validate_retention_days();
                        let _ = self.settings.save();
                    }
                });
                
                ui.add_space(20.0);
                if ui.button("Close").clicked() { self.show_settings_panel = false; }
            });
    }
    
    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("📜 Version History");
            ui.add_space(5.0);
            if let Some(path) = &self.current_file_path {
                if let Ok(stats) = get_history_stats(path) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Ver: {}", stats.total_versions));
                        ui.separator();
                        let size_mb = stats.total_size_bytes as f64 / (1024.0 * 1024.0);
                        ui.label(format!("{:.1} MB", size_mb));
                    });
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("🗑️ Clean").clicked() { self.cleanup_old_versions(); }
                if ui.button("🔄 Refresh").clicked() { self.refresh_versions(); }
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                if self.versions.is_empty() {
                    ui.label("No versions.");
                } else {
                    let versions = self.versions.clone();
                    for version in &versions {
                        ui.group(|ui| {
                            ui.label(format!("📅 {}", version.display_timestamp()));
                            ui.horizontal(|ui| {
                                if ui.button("👁").clicked() { self.preview_version(version); }
                                if ui.button("↩️").clicked() { self.show_restore_confirm = Some(version.clone()); }
                                if ui.button("🗑️").clicked() { self.show_delete_confirm = Some(version.clone()); }
                            });
                        });
                        ui.add_space(2.0);
                    }
                }
            });
        });
    }
    
    fn render_preview_window(&mut self, ctx: &egui::Context) {
        let preview_data = self.preview_version.clone();
        let mut close_clicked = false;
        if let Some((version, content)) = preview_data {
            let mut content_clone = content.clone();
            egui::Window::new(format!("Preview: {}", version.display_timestamp()))
                .resizable(true).default_width(600.0).default_height(400.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut content_clone).desired_width(f32::INFINITY).interactive(false));
                    });
                    if ui.button("Close").clicked() { close_clicked = true; }
                });
        }
        if close_clicked { self.preview_version = None; }
    }
    
    fn render_restore_confirm(&mut self, ctx: &egui::Context) {
        let restore_data = self.show_restore_confirm.clone();
        let mut do_restore = false;
        let mut do_cancel = false;
        if let Some(version) = restore_data {
            let version_clone = version.clone();
            egui::Window::new("Confirm Restore").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Restore this version?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() { do_restore = true; }
                        if ui.button("No").clicked() { do_cancel = true; }
                    });
                });
            if do_restore { self.restore_version_confirmed(&version_clone); }
        }
        if do_cancel { self.show_restore_confirm = None; }
    }
    
    fn render_delete_confirm(&mut self, ctx: &egui::Context) {
        let delete_data = self.show_delete_confirm.clone();
        let mut do_delete = false;
        let mut do_cancel = false;
        if let Some(version) = delete_data {
            let version_clone = version.clone();
            egui::Window::new("Confirm Delete").collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Delete this version?");
                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() { do_delete = true; }
                        if ui.button("No").clicked() { do_cancel = true; }
                    });
                });
            if do_delete { self.delete_version_confirmed(&version_clone); }
        }
        if do_cancel { self.show_delete_confirm = None; }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Renderuj dialogi
        if self.show_password_dialog { self.render_password_dialog(ctx); }
        if self.show_settings_panel { self.render_settings_panel(ctx); }
        if self.preview_version.is_some() { self.render_preview_window(ctx); }
        if self.show_restore_confirm.is_some() { self.render_restore_confirm(ctx); }
        if self.show_delete_confirm.is_some() { self.render_delete_confirm(ctx); }
        
        // Alert o braku uprawnień
        self.render_auth_alert(ctx);
        
        // Menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📄 New").clicked() { self.new_document(); ui.close_menu(); }
                    if ui.button("📂 Open").clicked() { self.open_file_dialog(); ui.close_menu(); }
                    if ui.button("💾 Save").clicked() { self.save_file(); ui.close_menu(); }
                    if ui.button("💾 Save As").clicked() { self.save_file_as(); ui.close_menu(); }
                    ui.separator();
                    if ui.button("❌ Exit").clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
                });
                ui.menu_button("Security", |ui| {
                    if ui.button("🔑 Select Keyfile").clicked() { self.select_keyfile(); ui.close_menu(); }
                    if ui.button("✨ Generate Keyfile").clicked() { self.generate_new_keyfile(); ui.close_menu(); }
                });
                ui.menu_button("History", |ui| {
                    if ui.button("📜 Toggle Panel").clicked() { self.show_history_panel = !self.show_history_panel; ui.close_menu(); }
                });
                ui.menu_button("Settings", |ui| {
                    if ui.button("⚙️ Preferences").clicked() { self.show_settings_panel = true; ui.close_menu(); }
                });
            });
        });
        
        // Auth Panel (Top)
        egui::TopBottomPanel::top("auth_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔒 Auth");
                ui.separator();
                ui.label("Pass:");
                ui.add(egui::TextEdit::singleline(&mut self.password).password(true).desired_width(120.0));
                ui.separator();
                ui.label("Key:");
                if let Some(path) = &self.keyfile_path {
                    ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "None");
                }
                if ui.button("Select").clicked() { self.select_keyfile(); }
                if ui.button("Gen").clicked() { self.generate_new_keyfile(); }
            });
        });
        
        // Status Bar (Bottom)
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if self.is_modified { ui.colored_label(egui::Color32::YELLOW, "* Modified"); }
            });
        });
        
        // History Panel (Right)
        egui::SidePanel::right("history_panel")
            .resizable(true)
            .default_width(250.0)
            .show_animated(ctx, self.show_history_panel, |ui| {
                self.render_history_panel(ui);
            });
        
        // Central Panel (Editor)
        egui::CentralPanel::default().show(ctx, |ui| {
            let style = (*ctx.style()).clone();
            let font_id = if self.settings.font_family == "Monospace" {
                egui::FontId::monospace(self.settings.font_size)
            } else {
                egui::FontId::proportional(self.settings.font_size)
            };
            
            let mut updated_style = style;
            updated_style.text_styles.insert(egui::TextStyle::Body, font_id.clone());
            updated_style.text_styles.insert(egui::TextStyle::Monospace, font_id);
            ctx.set_style(updated_style);
            
            // FIX: Naprawa layoutu edytora - wypełnia dostępną przestrzeń
            let available_height = ui.available_height();
            egui::ScrollArea::vertical().show(ui, |ui| {
                let text_edit = egui::TextEdit::multiline(&mut self.text_content)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::vec2(0.0, available_height))
                    .code_editor();
                
                if ui.add(text_edit).changed() {
                    self.is_modified = true;
                }
            });
        });
    }
}
