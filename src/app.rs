use eframe::egui;
use std::path::PathBuf;

use crate::crypto::{decrypt_file, encrypt_file, generate_keyfile};
use crate::history::DocumentWithHistory;
use crate::settings::Settings;
use crate::theme::{load_themes, Theme};

/// Debug log entry :D
#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: chrono::DateTime<chrono::Local>,
    level: LogLevel,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
enum ThemeEditorAction {
    Save(Theme),
    Apply(Theme),
    Cancel,
}

impl LogEntry {
    fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: chrono::Local::now(),
            level,
            message,
        }
    }

    fn display(&self) -> String {
        let level_str = match self.level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
        };
        format!(
            "[{}] {}: {}",
            self.timestamp.format("%H:%M:%S"),
            level_str,
            self.message
        )
    }
}

/// Application state
pub struct EditorApp {
    /// Current document with embedded history
    document: DocumentWithHistory,
    /// Path to keyfile
    keyfile_path: Option<PathBuf>,
    /// Path to currently open file
    current_file_path: Option<PathBuf>,
    /// Status message
    status_message: String,
    /// User preferences
    settings: Settings,
    /// Available themes
    themes: Vec<Theme>,
    /// Current theme
    current_theme: Theme,
    /// Show Settings panel
    show_settings_panel: bool,
    /// Show History panel
    show_history_panel: bool,
    /// Show Debug panel
    show_debug_panel: bool,
    /// Show file tree panel
    show_file_tree: bool,
    /// Document has been modified
    is_modified: bool,
    /// Debug log entries
    debug_log: Vec<LogEntry>,
    /// File tree current directory
    file_tree_dir: Option<PathBuf>,
    /// File tree entries
    file_tree_entries: Vec<PathBuf>,
    /// Icons
    icons: crate::icons::Icons,
    /// Show Theme Editor panel
    show_theme_editor: bool,
    /// Theme being edited (clone of current_theme for live editing)
    editing_theme: Option<Theme>,
    /// Currently highlighted line (1-indexed)
    highlighted_line: Option<usize>,
}

impl Default for EditorApp {
    fn default() -> Self {
        let settings = Settings::load();
        let themes = load_themes();

        // Find current theme
        let current_theme = themes
            .iter()
            .find(|t| t.name == settings.theme_name)
            .cloned()
            .unwrap_or_else(|| Theme::dark());

        // Load global keyfile if enabled
        let keyfile_path = if settings.use_global_keyfile {
            settings.global_keyfile_path.clone()
        } else {
            None
        };

        let status = if keyfile_path.is_some() {
            "Ready with global keyfile loaded".to_string()
        } else {
            "Ready - Load or generate a keyfile".to_string()
        };

        Self {
            document: DocumentWithHistory::default(),
            keyfile_path,
            current_file_path: None,
            status_message: status,
            settings: settings.clone(),
            themes,
            current_theme,
            show_settings_panel: false,
            show_history_panel: false,
            show_debug_panel: settings.show_debug_panel,
            show_file_tree: settings.show_file_tree,
            is_modified: false,
            debug_log: Vec::new(),
            file_tree_dir: settings.last_directory.clone(),
            file_tree_entries: Vec::new(),
            icons: crate::icons::Icons::load(&egui::Context::default()),
            show_theme_editor: false,
            editing_theme: None,
            highlighted_line: None,
        }
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        app.icons = crate::icons::Icons::load(&cc.egui_ctx);
        app.current_theme.apply(&cc.egui_ctx);
        app.log_info("Application started");
        app.refresh_file_tree();
        app
    }

    /// Update UI style (fonts) based on settings
    fn apply_style(&self, ctx: &egui::Context) {
        ctx.style_mut(|style| {
            use egui::{FontFamily, FontId, TextStyle};
            style.text_styles = [
                (
                    TextStyle::Heading,
                    FontId::new(self.settings.ui_font_size + 4.0, FontFamily::Proportional),
                ),
                (
                    TextStyle::Body,
                    FontId::new(self.settings.ui_font_size, FontFamily::Proportional),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(self.settings.editor_font_size, FontFamily::Monospace),
                ),
                (
                    TextStyle::Button,
                    FontId::new(self.settings.ui_font_size, FontFamily::Proportional),
                ),
                (
                    TextStyle::Small,
                    FontId::new(self.settings.ui_font_size - 4.0, FontFamily::Proportional),
                ),
            ]
            .into();
        });
    }

    /// Logging functions
    fn log_info(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Info, message.into()));
        if self.debug_log.len() > 1000 {
            self.debug_log.drain(0..100);
        }
    }

    fn log_warning(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Warning, message.into()));
    }

    fn log_error(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Error, message.into()));
    }

    /// Apply current theme
    fn apply_theme(&self, ctx: &egui::Context) {
        self.current_theme.apply(ctx);
    }

    /// New document
    fn new_document(&mut self) {
        if self.is_modified {
            // TODO: Save changes dialog
        }

        self.document = DocumentWithHistory::default();
        self.current_file_path = None;
        self.is_modified = false;
        self.status_message = "New document created".to_string();
        self.log_info("New document created");
    }

    /// Open file dialog
    fn open_file_dialog(&mut self) {
        self.log_info("Opening file dialog");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SED Files", &["sed"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.open_file(path);
        } else {
            self.log_info("File dialog cancelled");
        }
    }

    /// Open file
    fn open_file(&mut self, path: PathBuf) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to open file without keyfile");
            return;
        }

        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(format!("Opening file: {}", path.display()));
        self.log_info(format!("Using keyfile: {}", keyfile.display()));

        match decrypt_file(&keyfile, &path) {
            Ok(content) => {
                self.document = DocumentWithHistory::from_file_content(&content);
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                let history_count = self.document.get_history().len();
                self.status_message = format!(
                    "Opened: {} ({} history entries)",
                    path.display(),
                    history_count
                );
                self.log_info(format!(
                    "✓ File opened successfully with {} history entries",
                    history_count
                ));
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.log_error(format!("Failed to open file: {}", e));
            }
        }
    }

    /// Open directory for file tree
    fn open_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.log_info(format!("Opening directory: {}", path.display()));
            self.file_tree_dir = Some(path.clone());
            self.settings.last_directory = Some(path);
            let _ = self.settings.save();
            self.refresh_file_tree();
        }
    }

    /// Refresh file tree entries
    fn refresh_file_tree(&mut self) {
        self.file_tree_entries.clear();

        // Fix: Clone to release borrow on self
        let dir_opt = self.file_tree_dir.clone();
        if let Some(dir) = dir_opt {
            self.log_info(format!("Refreshing file tree for: {}", dir.display()));
            match std::fs::read_dir(dir) {
                Ok(entries) => {
                    let mut count = 0;
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "sed" {
                                    self.file_tree_entries.push(path.clone());
                                    count += 1;
                                    self.log_info(format!("Added .sed file: {}", path.display()));
                                }
                            }
                        }
                    }
                    self.log_info(format!("Found {} .sed files", count));
                }
                Err(e) => {
                    self.log_error(format!("Failed to read directory: {}", e));
                }
            }
        } else {
            self.log_warning("No directory selected for file tree");
        }

        self.file_tree_entries.sort();
    }

    /// Save file
    fn save_file(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to save without keyfile");
            return;
        }

        // Fix: Clone path to release borrow on self
        if let Some(path) = self.current_file_path.clone() {
            self.log_info("Saving to existing file path");
            self.perform_save(path);
        } else {
            self.log_info("No file path set, opening save dialog");
            self.save_file_as();
        }
    }

    /// Save file as
    fn save_file_as(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to save as without keyfile");
            return;
        }

        self.log_info("Opening save as dialog");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SED Files", &["sed"])
            .set_file_name("document.sed")
            .save_file()
        {
            self.perform_save(path);
        } else {
            self.log_info("Save as dialog cancelled");
        }
    }

    /// Perform actual save
    fn perform_save(&mut self, path: PathBuf) {
        let keyfile = self.keyfile_path.clone().unwrap();
        self.log_info(format!("Saving file: {}", path.display()));

        // Add snapshot if auto-snapshot enabled and content changed
        if self.settings.auto_snapshot_on_save && self.is_modified {
            self.document.add_snapshot(None);
            self.log_info("Snapshot created automatically");
        }

        let file_content = self.document.to_file_content();
        self.log_info(format!("Content size: {} bytes", file_content.len()));

        match encrypt_file(&file_content, &keyfile, &path) {
            Ok(_) => {
                self.current_file_path = Some(path.clone());
                self.is_modified = false;
                let history_count = self.document.get_history().len();
                self.status_message = format!(
                    "Saved: {} ({} history entries)",
                    path.display(),
                    history_count
                );
                self.log_info("✓ File saved successfully");

                // Refresh file tree in case it was a new file
                self.refresh_file_tree();
            }
            Err(e) => {
                self.status_message = format!("Error: {}", e);
                self.log_error(format!("Save failed: {}", e));
            }
        }
    }

    /// Load keyfile
    fn load_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            self.log_info(format!("Attempting to load keyfile: {}", path.display()));

            // Validate keyfile
            match std::fs::metadata(&path) {
                Ok(metadata) => {
                    let size = metadata.len();
                    self.log_info(format!("Keyfile size: {} bytes", size));
                    if size != 256 {
                        self.status_message =
                            format!("Error: Invalid keyfile (must be 256 bytes, got {})", size);
                        self.log_error(format!(
                            "Invalid keyfile size: {} bytes (expected 256)",
                            size
                        ));
                        return;
                    }

                    // Try to read the file
                    match std::fs::read(&path) {
                        Ok(content) => {
                            if content.len() != 256 {
                                self.status_message = "Error: Invalid keyfile content".to_string();
                                self.log_error("Keyfile content length mismatch");
                                return;
                            }

                            self.keyfile_path = Some(path.clone());
                            self.status_message =
                                format!("✓ Valid keyfile loaded: {}", path.display());
                            self.log_info(format!(
                                "✓ Valid keyfile loaded successfully: {}",
                                path.display()
                            ));
                        }
                        Err(e) => {
                            self.status_message = format!("Error: Cannot read keyfile: {}", e);
                            self.log_error(format!("Cannot read keyfile: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.status_message = format!("Error: Cannot access keyfile: {}", e);
                    self.log_error(format!("Cannot access keyfile: {}", e));
                }
            }
        }
    }

    /// Generate new keyfile
    fn generate_new_keyfile(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_file_name("keyfile").save_file() {
            self.log_info(format!("Generating new keyfile: {}", path.display()));
            match generate_keyfile(&path) {
                Ok(_) => {
                    self.keyfile_path = Some(path.clone());
                    self.status_message = format!("✓ Keyfile generated: {}", path.display());
                    self.log_info(format!(
                        "✓ Keyfile generated successfully (256 bytes): {}",
                        path.display()
                    ));
                }
                Err(e) => {
                    self.status_message = format!("Error: {}", e);
                    self.log_error(format!("Keyfile generation failed: {}", e));
                }
            }
        }
    }

    /// Load version from history
    fn load_history_version(&mut self, index: usize) {
        if self.document.load_version(index) {
            self.is_modified = true;
            self.status_message = "Version loaded from history".to_string();
            self.log_info(format!("Loaded history version #{}", index));
        }
    }

    /// Delete history entry
    fn delete_history_entry(&mut self, index: usize) {
        if self.document.delete_entry(index) {
            self.status_message = "History entry deleted".to_string();
            self.log_info(format!("Deleted history entry #{}", index));
        }
    }

    /// Clear all history
    fn clear_all_history(&mut self) {
        let count = self.document.get_history().len();
        self.document.clear_history();
        self.is_modified = true;
        self.status_message = format!("Cleared {} history entries", count);
        self.log_info(format!("Cleared all history ({} entries)", count));
    }

    /// Render icon toolbar
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // Stały rozmiar przycisków (kwadratowe)
            let button_size = egui::vec2(32.0, 32.0);
            let icon_size = egui::vec2(24.0, 24.0);

            // --- LEFT SIDE: Main Actions ---

            // New
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.new_doc).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("New (Ctrl+N)")
                .clicked()
            {
                self.new_document();
            }

            // Open
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.open).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Open (Ctrl+O)")
                .clicked()
            {
                self.open_file_dialog();
            }

            // Open Directory
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.open_folder).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Open Directory")
                .clicked()
            {
                self.open_directory();
            }

            // Save
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.save).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Save (Ctrl+S)")
                .clicked()
            {
                self.save_file();
            }

            // Save As
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.save_as).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Save As")
                .clicked()
            {
                self.save_file_as();
            }

            ui.separator();

            // Load Key
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.key).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Load Keyfile")
                .clicked()
            {
                self.load_keyfile();
            }

            // Generate Key
            if ui
                .add_sized(
                    button_size,
                    egui::ImageButton::new(
                        egui::Image::new(&self.icons.generate).fit_to_exact_size(icon_size),
                    )
                    .frame(false),
                )
                .on_hover_text("Generate Keyfile")
                .clicked()
            {
                self.generate_new_keyfile();
            }

            // Keyfile indicator
            if let Some(path) = &self.keyfile_path {
                ui.label(
                    egui::RichText::new(format!(
                        "🔓 {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ))
                    .color(egui::Color32::GREEN),
                );
            } else {
                ui.label(egui::RichText::new("⚠ No keyfile").color(egui::Color32::RED));
            }

            ui.add_space(20.0);

            // --- RIGHT SIDE: Toggles ---
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Theme Editor button
                if ui
                    .add_sized(
                        button_size,
                        egui::ImageButton::new(
                            egui::Image::new(&self.icons.settings).fit_to_exact_size(icon_size),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Theme Editor")
                    .clicked()
                {
                    self.show_theme_editor = true;
                    self.editing_theme = Some(self.current_theme.clone());
                }

                // Settings
                if ui
                    .add_sized(
                        button_size,
                        egui::ImageButton::new(
                            egui::Image::new(&self.icons.settings).fit_to_exact_size(icon_size),
                        )
                        .frame(false),
                    )
                    .on_hover_text("Settings")
                    .clicked()
                {
                    self.show_settings_panel = true;
                }

                ui.separator();

                // Debug toggle
                if ui
                    .add_sized(
                        button_size,
                        egui::ImageButton::new(
                            egui::Image::new(&self.icons.debug).fit_to_exact_size(icon_size),
                        )
                        .frame(self.show_debug_panel),
                    )
                    .on_hover_text("Toggle Debug")
                    .clicked()
                {
                    self.show_debug_panel = !self.show_debug_panel;
                    self.settings.show_debug_panel = self.show_debug_panel;
                    let _ = self.settings.save();
                }

                // History toggle
                if ui
                    .add_sized(
                        button_size,
                        egui::ImageButton::new(
                            egui::Image::new(&self.icons.history).fit_to_exact_size(icon_size),
                        )
                        .frame(self.show_history_panel),
                    )
                    .on_hover_text("Toggle History")
                    .clicked()
                {
                    self.show_history_panel = !self.show_history_panel;
                }

                // File tree toggle
                if ui
                    .add_sized(
                        button_size,
                        egui::ImageButton::new(
                            egui::Image::new(&self.icons.file_tree).fit_to_exact_size(icon_size),
                        )
                        .frame(self.show_file_tree),
                    )
                    .on_hover_text("Toggle File Tree")
                    .clicked()
                {
                    self.show_file_tree = !self.show_file_tree;
                    self.settings.show_file_tree = self.show_file_tree;
                    let _ = self.settings.save();
                }
            });
        });
    }

    /// Render theme editor panel
    fn render_theme_editor_panel(&mut self, ctx: &egui::Context) {
        let mut close_editor = false;
        let mut action: Option<ThemeEditorAction> = None;

        if let Some(ref mut theme) = self.editing_theme {
            egui::Window::new("🎨 Theme Editor")
                .collapsible(false)
                .resizable(true)
                .default_width(600.0)
                .show(ctx, |ui| {
                    ui.heading("Theme Editor");

                    // Theme name
                    ui.horizontal(|ui| {
                        ui.label("Theme Name:");
                        ui.text_edit_singleline(&mut theme.name);
                    });

                    ui.separator();
                    ui.heading("Colors");

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        // Background
                        ui.horizontal(|ui| {
                            ui.label("Background:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.background[0],
                                theme.colors.background[1],
                                theme.colors.background[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.background = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        // Foreground
                        ui.horizontal(|ui| {
                            ui.label("Foreground:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.foreground[0],
                                theme.colors.foreground[1],
                                theme.colors.foreground[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.foreground = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        // Panel Background
                        ui.horizontal(|ui| {
                            ui.label("Panel Background:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.panel_background[0],
                                theme.colors.panel_background[1],
                                theme.colors.panel_background[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.panel_background = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.label(egui::RichText::new("Editor Colors").strong());

                        // Selection Background
                        ui.horizontal(|ui| {
                            ui.label("Selection Background:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.selection_background[0],
                                theme.colors.selection_background[1],
                                theme.colors.selection_background[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.selection_background =
                                    [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        // ✅ NOWY: Kursor
                        ui.horizontal(|ui| {
                            ui.label("Cursor Color:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.cursor[0],
                                theme.colors.cursor[1],
                                theme.colors.cursor[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.cursor = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        // Line Number
                        ui.horizontal(|ui| {
                            ui.label("Line Number:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.line_number[0],
                                theme.colors.line_number[1],
                                theme.colors.line_number[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.line_number = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.label(egui::RichText::new("Syntax Colors").strong());

                        // Comment
                        ui.horizontal(|ui| {
                            ui.label("Comment:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.comment[0],
                                theme.colors.comment[1],
                                theme.colors.comment[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.comment = [color.r(), color.g(), color.b()];
                                theme.apply(ctx);
                            }
                        });
                    });

                    ui.separator();

                    // Buttons
                    ui.horizontal(|ui| {
                        // Save button
                        if ui.button("💾 Save Theme").clicked() {
                            action = Some(ThemeEditorAction::Save(theme.clone()));
                            close_editor = true;
                        }

                        // Apply without saving
                        if ui.button("✓ Apply").clicked() {
                            action = Some(ThemeEditorAction::Apply(theme.clone()));
                            close_editor = true;
                        }

                        // Cancel
                        if ui.button("✖ Cancel").clicked() {
                            action = Some(ThemeEditorAction::Cancel);
                            close_editor = true;
                        }

                        // Reset to defaults
                        if ui.button("🔄 Reset to Dark").clicked() {
                            *theme = crate::theme::Theme::dark();
                            theme.apply(ctx);
                        }
                    });
                });
        }

        // Handle actions outside the closure to avoid borrow conflicts
        if let Some(act) = action {
            match act {
                ThemeEditorAction::Save(theme) => match crate::theme::save_theme(&theme) {
                    Ok(_) => {
                        self.current_theme = theme.clone();
                        self.settings.theme_name = theme.name.clone();
                        let _ = self.settings.save();
                        self.themes = crate::theme::load_themes();
                        self.status_message = format!("✓ Theme '{}' saved", theme.name);
                        self.log_info(format!("Theme '{}' saved successfully", theme.name));
                    }
                    Err(e) => {
                        self.status_message = format!("Error saving theme: {}", e);
                        self.log_error(format!("Failed to save theme: {}", e));
                    }
                },
                ThemeEditorAction::Apply(theme) => {
                    self.current_theme = theme.clone();
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.status_message = "Theme applied".to_string();
                    self.log_info("Theme applied (not saved to file)");
                }
                ThemeEditorAction::Cancel => {
                    self.current_theme.apply(ctx);
                    self.log_info("Theme editing cancelled");
                }
            }
        }

        if close_editor {
            self.show_theme_editor = false;
            self.editing_theme = None;
        }
    }

    /// Render file tree panel
    fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Files");

            if let Some(dir) = &self.file_tree_dir {
                ui.label(egui::RichText::new(dir.display().to_string()).small());
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for path in &self.file_tree_entries.clone() {
                        let filename = path.file_name().unwrap_or_default().to_string_lossy();
                        if ui.button(filename.as_ref()).clicked() {
                            self.open_file(path.clone());
                        }
                    }
                });
            } else {
                ui.label("No directory opened");
                if ui.button("Open Directory").clicked() {
                    self.open_directory();
                }
            }
        });
    }

    /// Render history panel
    fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("History");

            let history_len = self.document.get_history().len();
            ui.horizontal(|ui| {
                ui.label(format!("Entries: {}/100", history_len));

                // Clear All button
                if history_len > 0 {
                    if ui.button("Clear All").clicked() {
                        self.clear_all_history();
                    }
                }
            });

            ui.separator();

            // Clone history to avoid borrow issues in closure
            let history_entries: Vec<_> = self.document.get_history().to_vec();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if history_entries.is_empty() {
                    ui.label("No history");
                } else {
                    for (index, entry) in history_entries.iter().enumerate() {
                        ui.group(|ui| {
                            ui.label(format!("📅 {}", entry.display_timestamp()));
                            ui.label(format!("💾 {}", entry.display_size()));

                            ui.horizontal(|ui| {
                                if ui.button("Load").clicked() {
                                    self.load_history_version(index);
                                }

                                if ui.button("Delete").clicked() {
                                    self.delete_history_entry(index);
                                }
                            });
                        });

                        ui.add_space(4.0);
                    }
                }
            });
        });
    }

    /// Render debug panel
    fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Debug Log");

            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.debug_log.clear();
                }

                ui.label(format!("Entries: {}", self.debug_log.len()));
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &self.debug_log {
                        let color = match entry.level {
                            LogLevel::Info => ui.style().visuals.text_color(),
                            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
                            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
                        };
                        ui.colored_label(color, entry.display());
                    }
                });
        });
    }

    /// Render settings panel
    fn render_settings_panel(&mut self, ctx: &egui::Context) {
        egui::Window::new("⚙ Settings")
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Appearance");

                // Theme selection
                ui.horizontal(|ui| {
                    ui.label("Theme:");
                    let current_name = self.current_theme.name.clone();

                    egui::ComboBox::from_id_salt("theme_selector")
                        .selected_text(&current_name)
                        .show_ui(ui, |ui| {
                            for theme in &self.themes {
                                if ui
                                    .selectable_label(theme.name == current_name, &theme.name)
                                    .clicked()
                                {
                                    self.current_theme = theme.clone();
                                    self.settings.theme_name = theme.name.clone();
                                    self.apply_theme(ctx);
                                    let _ = self.settings.save();
                                }
                            }
                        });

                    if ui.button("Refresh").clicked() {
                        self.themes = load_themes();
                        self.log_info("Themes refreshed");
                    }
                });

                // UI font size
                ui.horizontal(|ui| {
                    ui.label("UI Font Size:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.settings.ui_font_size,
                            8.0..=32.0,
                        ))
                        .changed()
                    {
                        self.settings.validate_font_sizes();
                        let _ = self.settings.save();
                    }
                });

                // Editor font size
                ui.horizontal(|ui| {
                    ui.label("Editor Font Size:");
                    if ui
                        .add(egui::Slider::new(
                            &mut self.settings.editor_font_size,
                            8.0..=32.0,
                        ))
                        .changed()
                    {
                        self.settings.validate_font_sizes();
                        let _ = self.settings.save();
                    }
                });

                ui.separator();
                ui.heading("Global Keyfile");

                if ui
                    .checkbox(
                        &mut self.settings.use_global_keyfile,
                        "Use global keyfile on startup",
                    )
                    .changed()
                {
                    let _ = self.settings.save();
                }

                ui.horizontal(|ui| {
                    ui.label("Current:");
                    if let Some(path) = &self.settings.global_keyfile_path {
                        ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                    } else {
                        ui.label("None");
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Set Global Keyfile").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.settings.global_keyfile_path = Some(path.clone());
                            if self.settings.use_global_keyfile {
                                self.keyfile_path = Some(path);
                            }
                            let _ = self.settings.save();
                            self.log_info("Global keyfile set");
                        }
                    }

                    if ui.button("Clear").clicked() {
                        self.settings.global_keyfile_path = None;
                        let _ = self.settings.save();
                    }
                });

                ui.separator();
                ui.heading("Editor");

                if ui
                    .checkbox(&mut self.settings.show_line_numbers, "Show line numbers")
                    .changed()
                {
                    let _ = self.settings.save();
                }

                if ui
                    .checkbox(
                        &mut self.settings.auto_snapshot_on_save,
                        "Auto-snapshot on save",
                    )
                    .changed()
                {
                    let _ = self.settings.save();
                }

                ui.add_space(20.0);

                if ui.button("Close").clicked() {
                    self.show_settings_panel = false;
                }
            });
    }

    /// Render text editor with line numbers
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = text.lines().count().max(1);

        // ✅ Sklonuj wartości PRZED closure, aby uniknąć multiple borrow
        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;
        let line_number_color = self.current_theme.colors.line_number_color();
        let selection_bg_color = self.current_theme.colors.selection_color();
        let highlighted_line = self.highlighted_line;

        egui::ScrollArea::vertical()
            .id_salt("editor")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    // Użyj DOKŁADNIE tego samego fontu dla numerów i edytora
                    let font_id = egui::FontId::monospace(editor_font_size);

                    // Pobierz dokładną wysokość linii i metryki czcionki
                    let (row_height, galley_pos) = ui.fonts(|f| {
                        let row_height = f.row_height(&font_id);
                        // Pobierz pozycję bazową dla galley (baseline offset)
                        let galley = f.layout_no_wrap(
                            "A".to_string(),
                            font_id.clone(),
                            egui::Color32::WHITE,
                        );
                        let galley_offset = galley.rect.top();
                        (row_height, galley_offset)
                    });

                    // Line numbers
                    if show_line_numbers {
                        let line_number_width =
                            (line_count.to_string().len() as f32 * editor_font_size * 0.6)
                                .max(40.0);

                        ui.allocate_ui_with_layout(
                            egui::vec2(line_number_width, ui.available_height()),
                            egui::Layout::top_down(egui::Align::RIGHT),
                            |ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                                for i in 1..=line_count {
                                    let (rect, response) = ui.allocate_exact_size(
                                        egui::vec2(line_number_width, row_height),
                                        egui::Sense::click(),
                                    );

                                    // ✅ Zwróć klikniętą linię zamiast bezpośrednio modyfikować self
                                    if response.clicked() {
                                        // Zapisz to w response data
                                        response.ctx.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("clicked_line"), i);
                                        });
                                    }

                                    // Podświetl tło jeśli to wybrana linia
                                    if highlighted_line == Some(i) {
                                        ui.painter().rect_filled(
                                            rect,
                                            0.0,
                                            selection_bg_color.linear_multiply(0.5), // Półprzezroczyste
                                        );
                                    }

                                    // Rysuj numer linii z DOKŁADNIE tym samym offsetem co tekst
                                    let text_pos = rect.right_top() + egui::vec2(-8.0, galley_pos);

                                    ui.painter().text(
                                        text_pos,
                                        egui::Align2::RIGHT_TOP,
                                        i.to_string(),
                                        font_id.clone(),
                                        line_number_color,
                                    );
                                }
                            },
                        );

                        // Separator - pionowa linia
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(2.0, ui.available_height()),
                            egui::Sense::hover(),
                        );
                        ui.painter().line_segment(
                            [rect.center_top(), rect.center_bottom()],
                            egui::Stroke::new(
                                1.0,
                                ui.visuals().widgets.noninteractive.bg_stroke.color,
                            ),
                        );

                        ui.add_space(4.0); // Mały odstęp
                    }

                    // Text editor
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

                        let available_height = ui.available_height();

                        let text_edit = egui::TextEdit::multiline(text)
                            .desired_width(f32::INFINITY)
                            .min_size(egui::vec2(0.0, available_height))
                            .font(font_id.clone())
                            .code_editor()
                            .frame(false)
                            .lock_focus(true)
                            .desired_rows(line_count)
                            .margin(egui::Margin::same(0.0));

                        if ui.add(text_edit).changed() {
                            // Zwróć info że coś się zmieniło
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("text_changed"), true);
                            });
                        }
                    });
                });
            });

        // ✅ PO closure - obsłuż kliknięcie i zmiany
        ui.ctx().data_mut(|d| {
            if let Some(clicked_line) = d.get_temp::<usize>(egui::Id::new("clicked_line")) {
                self.highlighted_line = Some(clicked_line);
                self.log_info(format!("Line {} selected", clicked_line));
            }

            if d.get_temp::<bool>(egui::Id::new("text_changed"))
                .unwrap_or(false)
            {
                self.is_modified = true;
            }
        });
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply styles (font sizes) every frame/update
        self.apply_style(ctx);

        // Keyboard shortcuts
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

        // Dialogs
        if self.show_settings_panel {
            self.render_settings_panel(ctx);
        }

        // Theme Editor
        if self.show_theme_editor {
            self.render_theme_editor_panel(ctx);
        }

        // Toolbar
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            self.render_toolbar(ui);
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
                if self.is_modified {
                    ui.label(egui::RichText::new("*").color(egui::Color32::YELLOW));
                }
            });
        });

        // File tree (left)
        egui::SidePanel::left("file_tree")
            .resizable(true)
            .default_width(self.settings.file_tree_width)
            .width_range(150.0..=500.0)
            .show_animated(ctx, self.show_file_tree, |ui| {
                self.render_file_tree(ui);

                // Save width if changed
                let current_width = ui.available_width();
                if (current_width - self.settings.file_tree_width).abs() > 1.0 {
                    self.settings.file_tree_width = current_width;
                    let _ = self.settings.save();
                }
            });

        // History panel (right)
        egui::SidePanel::right("history")
            .resizable(true)
            .default_width(250.0)
            .show_animated(ctx, self.show_history_panel, |ui| {
                self.render_history_panel(ui);
            });

        // Debug panel (right, below history if both shown)
        egui::SidePanel::right("debug")
            .resizable(true)
            .default_width(250.0)
            .show_animated(ctx, self.show_debug_panel, |ui| {
                self.render_debug_panel(ui);
            });

        // Central editor
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_editor(ui);
        });
    }
}
