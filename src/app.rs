use eframe::egui;
use std::path::PathBuf;

use crate::crypto::{decrypt_file, encrypt_file, generate_keyfile};
use crate::history::{
    cleanup_old_versions, create_snapshot, delete_version, get_history_stats, list_versions,
    load_version, restore_version, VersionInfo,
};
use crate::settings::Settings;

/// Application state
pub struct EditorApp {
    /// Text editor content
    text_content: String,
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
    /// Show Settings panel
    show_settings_panel: bool,
    /// Show History panel
    show_history_panel: bool,
    /// Document has been modified
    is_modified: bool,
    /// Show restore confirmation dialog
    show_restore_confirm: Option<VersionInfo>,
    /// Show delete confirmation dialog
    show_delete_confirm: Option<VersionInfo>,
    /// Show alert when trying to save without keyfile
    show_missing_keyfile_alert: bool,
    /// Show save changes confirmation when previewing with unsaved changes
    show_save_changes_dialog: Option<VersionInfo>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let settings = Settings::load();

        // Load initial keyfile based on settings
        // Priority 1: Global default keyfile (if enabled and path exists)
        // Priority 2: Last used keyfile (if "Remember" enabled)
        let initial_keyfile =
            if settings.use_default_keyfile && settings.default_keyfile_path.is_some() {
                settings.default_keyfile_path.clone()
            } else if settings.remember_keyfile_path {
                settings.last_keyfile_path.clone()
            } else {
                None
            };

        let status = if let Some(path) = &initial_keyfile {
            format!(
                "Ready. Loaded keyfile: {}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        } else {
            "Ready. Select a keyfile to begin.".to_string()
        };

        Self {
            text_content: String::new(),
            keyfile_path: initial_keyfile,
            current_file_path: None,
            versions: Vec::new(),
            status_message: status,
            settings,
            show_settings_panel: false,
            show_history_panel: false,
            is_modified: false,
            show_restore_confirm: None,
            show_delete_confirm: None,
            show_missing_keyfile_alert: false,
            show_save_changes_dialog: None,
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
            // TODO: In future add "Save changes?" dialog
        }

        self.text_content.clear();
        // Don't clear keyfile_path - user might want to use the same one (especially default)
        self.current_file_path = None;
        self.versions.clear();
        self.is_modified = false;
        self.status_message = "New document created".to_string();
    }

    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Encrypted Document", &["sed"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.open_file(path);
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: Select keyfile first".to_string();
            self.show_missing_keyfile_alert = true;
            return;
        }

        let keyfile = self.keyfile_path.clone().unwrap();

        if let Ok(metadata) = std::fs::metadata(&path) {
            eprintln!("DEBUG: Opening file, size: {} bytes", metadata.len());
        } else {
            self.status_message = "✗ Error: File not found".to_string();
            return;
        }

        eprintln!("DEBUG: Keyfile: {:?}", keyfile);

        match decrypt_file(&keyfile, &path) {
            Ok(content) => {
                eprintln!(
                    "DEBUG: File decrypted successfully, {} bytes",
                    content.len()
                );
                self.text_content = content;
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
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
                eprintln!("DEBUG: Decryption failed: {:?}", e);
                self.status_message = format!("✗ Error: {}", e);
            }
        }
    }

    fn save_file(&mut self) {
        // Check if we have keyfile
        if self.keyfile_path.is_none() {
            self.show_missing_keyfile_alert = true;
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
            self.show_missing_keyfile_alert = true;
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
            self.status_message = "✗ No keyfile selected".to_string();
            return;
        };

        eprintln!(
            "DEBUG: Saving file with {} bytes of content",
            self.text_content.len()
        );
        eprintln!("DEBUG: Keyfile: {:?}", keyfile);
        eprintln!("DEBUG: Output path: {:?}", path);

        match encrypt_file(&self.text_content, &keyfile, &path) {
            Ok(_) => {
                if let Ok(metadata) = std::fs::metadata(&path) {
                    eprintln!(
                        "DEBUG: File saved successfully, size: {} bytes",
                        metadata.len()
                    );
                }

                self.current_file_path = Some(path.clone());
                self.is_modified = false;

                if self.settings.auto_snapshot_on_save {
                    match create_snapshot(&self.text_content, &keyfile, &path, None) {
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

                if self.settings.remember_keyfile_path {
                    self.settings.last_keyfile_path = Some(keyfile.clone());
                    let _ = self.settings.save();
                }
            }
            Err(e) => {
                eprintln!("DEBUG: Encryption failed: {:?}", e);
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
        // If there are unsaved changes, show dialog
        if self.is_modified {
            self.show_save_changes_dialog = Some(version.clone());
        } else {
            // Load directly to main editor
            self.load_version_to_editor(version);
        }
    }

    fn load_version_to_editor(&mut self, version: &VersionInfo) {
        if let Some(keyfile) = &self.keyfile_path {
            match load_version(version, keyfile) {
                Ok(content) => {
                    self.text_content = content;
                    self.is_modified = false;
                    self.status_message =
                        format!("📄 Loaded version: {}", version.display_timestamp());
                }
                Err(e) => {
                    self.status_message = format!("✗ Error loading version: {}", e);
                }
            }
        } else {
            self.status_message = "✗ No keyfile selected".to_string();
        }
    }

    fn restore_version_confirmed(&mut self, version: &VersionInfo) {
        if let (Some(keyfile), Some(path)) = (&self.keyfile_path, &self.current_file_path) {
            match restore_version(version, keyfile, path, true) {
                Ok(_) => match decrypt_file(keyfile, path) {
                    Ok(content) => {
                        self.text_content = content;
                        self.is_modified = false;
                        self.refresh_versions();
                        self.status_message =
                            format!("✓ Restored: {}", version.display_timestamp());
                    }
                    Err(e) => {
                        self.status_message = format!("✗ Error reloading: {}", e);
                    }
                },
                Err(e) => {
                    self.status_message = format!("✗ Error restoring: {}", e);
                }
            }

            self.show_restore_confirm = None;
        }
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

    // UI RENDERERS CONTINUE IN PART 2...
}

// PART 2: UI Renderers and eframe::App implementation

impl EditorApp {
    fn render_keyfile_alert(&mut self, ctx: &egui::Context) {
        if self.show_missing_keyfile_alert {
            egui::Window::new("⚠ Cannot Proceed")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, "Keyfile Required!");
                    ui.add_space(5.0);
                    ui.label("To open or save files, you must:");
                    ui.label("• Select or generate a keyfile");
                    ui.add_space(10.0);

                    if ui.button("OK, I'll do that").clicked() {
                        self.show_missing_keyfile_alert = false;
                    }
                });
        }
    }

    fn render_settings_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙️ Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.heading("Appearance");

                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let changed = ui
                        .selectable_value(&mut self.settings.dark_theme, true, "🌙 Dark")
                        .clicked()
                        || ui
                            .selectable_value(&mut self.settings.dark_theme, false, "☀️ Light")
                            .clicked();

                    if changed {
                        self.apply_theme(ctx);
                        let _ = self.settings.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Font Size:");
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.font_size, 8.0..=32.0)
                                .suffix(" px"),
                        )
                        .changed()
                    {
                        self.settings.validate_font_size();
                        let _ = self.settings.save();
                    }
                });

                ui.separator();

                ui.heading("Global Default Keyfile");
                ui.label("Define a keyfile to be loaded automatically on startup.");

                if ui
                    .checkbox(
                        &mut self.settings.use_default_keyfile,
                        "Always use this keyfile globally",
                    )
                    .changed()
                {
                    let _ = self.settings.save();
                }

                ui.horizontal(|ui| {
                    ui.label("Current Default:");
                    if let Some(path) = &self.settings.default_keyfile_path {
                        ui.label(
                            egui::RichText::new(
                                path.file_name().unwrap_or_default().to_string_lossy(),
                            )
                            .strong(),
                        );
                    } else {
                        ui.colored_label(egui::Color32::YELLOW, "None set");
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("📂 Select Default Keyfile").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Key Files", &["key"])
                            .pick_file()
                        {
                            self.settings.default_keyfile_path = Some(path.clone());

                            if self.settings.use_default_keyfile {
                                self.keyfile_path = Some(path);
                                self.status_message =
                                    "Default keyfile updated and applied.".to_string();
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

                ui.heading("Version Control");

                if ui
                    .checkbox(
                        &mut self.settings.auto_snapshot_on_save,
                        "Auto-snapshot on save",
                    )
                    .changed()
                {
                    let _ = self.settings.save();
                }

                ui.horizontal(|ui| {
                    ui.label("Retention:");
                    if ui
                        .add(
                            egui::Slider::new(&mut self.settings.snapshot_retention_days, 0..=365)
                                .suffix(" days"),
                        )
                        .changed()
                    {
                        self.settings.validate_retention_days();
                        let _ = self.settings.save();
                    }
                });

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

            if let Some(path) = &self.current_file_path {
                if let Ok(stats) = get_history_stats(path) {
                    ui.horizontal(|ui| {
                        ui.label(format!("Ver: {}", stats.total_versions));
                        ui.separator();
                        let size_mb = stats.total_size_bytes as f64 / (1024.0 * 1024.0);
                        ui.label(format!("{:.1} MB", size_mb));
                    });
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("🗑️ Clean").clicked() {
                        self.cleanup_old_versions();
                    }

                    if ui.button("🔄 Refresh").clicked() {
                        self.refresh_versions();
                    }
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
                                    if ui.button("👁").on_hover_text("Load this version").clicked()
                                    {
                                        self.preview_version(version);
                                    }

                                    if ui.button("↩️").clicked() {
                                        self.show_restore_confirm = Some(version.clone());
                                    }

                                    if ui.button("🗑️").clicked() {
                                        self.show_delete_confirm = Some(version.clone());
                                    }
                                });
                            });
                            ui.add_space(2.0);
                        }
                    }
                });
            }
        });
    }

    fn render_save_changes_dialog(&mut self, ctx: &egui::Context) {
        let version_data = self.show_save_changes_dialog.clone();
        let mut do_save = false;
        let mut do_discard = false;
        let mut do_cancel = false;

        if let Some(version) = version_data {
            let version_clone = version.clone();

            egui::Window::new("⚠️ Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("You have unsaved changes in the current document.");
                    ui.add_space(5.0);
                    ui.label("Do you want to save them before loading a version?");
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save").clicked() {
                            do_save = true;
                        }

                        if ui.button("🗑️ Discard").clicked() {
                            do_discard = true;
                        }

                        if ui.button("✗ Cancel").clicked() {
                            do_cancel = true;
                        }
                    });
                });

            if do_save {
                self.save_file();
                self.load_version_to_editor(&version_clone);
                self.show_save_changes_dialog = None;
            }

            if do_discard {
                self.load_version_to_editor(&version_clone);
                self.show_save_changes_dialog = None;
            }
        }

        if do_cancel {
            self.show_save_changes_dialog = None;
        }
    }

    fn render_restore_confirm(&mut self, ctx: &egui::Context) {
        let restore_data = self.show_restore_confirm.clone();
        let mut do_restore = false;
        let mut do_cancel = false;

        if let Some(version) = restore_data {
            let version_clone = version.clone();

            egui::Window::new("Confirm Restore")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Restore this version?");

                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            do_restore = true;
                        }

                        if ui.button("No").clicked() {
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
        let delete_data = self.show_delete_confirm.clone();
        let mut do_delete = false;
        let mut do_cancel = false;

        if let Some(version) = delete_data {
            let version_clone = version.clone();

            egui::Window::new("Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Delete this version?");

                    ui.horizontal(|ui| {
                        if ui.button("Yes").clicked() {
                            do_delete = true;
                        }

                        if ui.button("No").clicked() {
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
        ctx.input_mut(|i| {
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::S,
            )) {
                self.save_file();
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::O,
            )) {
                self.open_file_dialog();
            }
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::N,
            )) {
                self.new_document();
            }
        });

        // Render dialogs
        if self.show_settings_panel {
            self.render_settings_panel(ctx);
        }

        if self.show_save_changes_dialog.is_some() {
            self.render_save_changes_dialog(ctx);
        }

        if self.show_restore_confirm.is_some() {
            self.render_restore_confirm(ctx);
        }

        if self.show_delete_confirm.is_some() {
            self.render_delete_confirm(ctx);
        }

        self.render_keyfile_alert(ctx);

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

                    if ui.button("⌫ Exit").clicked() {
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
                });

                ui.menu_button("Settings", |ui| {
                    if ui.button("⚙️ Preferences").clicked() {
                        self.show_settings_panel = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Auth Panel (Top) - Only Keyfile
        egui::TopBottomPanel::top("auth_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔒 Auth");
                ui.separator();

                ui.label("Keyfile:");
                if let Some(path) = &self.keyfile_path {
                    ui.label(
                        egui::RichText::new(path.file_name().unwrap_or_default().to_string_lossy())
                            .strong(),
                    );
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "⚠ None");
                }

                if ui.button("Select").clicked() {
                    self.select_keyfile();
                }

                if ui.button("Generate").clicked() {
                    self.generate_new_keyfile();
                }
            });
        });

        // Status Bar (Bottom)
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);

                if self.is_modified {
                    ui.colored_label(egui::Color32::YELLOW, "* Modified");
                }
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
            updated_style
                .text_styles
                .insert(egui::TextStyle::Body, font_id.clone());
            updated_style
                .text_styles
                .insert(egui::TextStyle::Monospace, font_id);

            ctx.set_style(updated_style);

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
