use eframe::egui;
use std::path::PathBuf;

use crate::crypto::{decrypt_file, encrypt_file, generate_keyfile};
use crate::history::DocumentWithHistory;
use crate::settings::Settings;
use crate::theme::{load_themes, Theme};

/// Debug log entry
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

/// File tree entry type
#[derive(Debug, Clone)]
enum FileTreeEntry {
    File(PathBuf),
    Directory(PathBuf),
}

#[derive(Debug, Clone)]
enum PendingAction {
    None,
    NewDocument,
    OpenFile,
    OpenDirectory,
    Exit,
    OpenFileFromTree(PathBuf),
    ChangeDirectory(PathBuf),
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
    file_tree_entries: Vec<FileTreeEntry>,
    /// Icons
    icons: crate::icons::Icons,
    /// Show Theme Editor panel
    show_theme_editor: bool,
    /// Theme being edited (clone of current_theme for live editing)
    editing_theme: Option<Theme>,
    /// Currently highlighted line (1-indexed)
    highlighted_line: Option<usize>,
    /// Show goto line dialog
    show_goto_line: bool,
    /// Goto line input
    goto_line_input: String,
    /// Show close confirmation dialog
    show_close_confirmation: bool,
    /// Pending action to execute after confirmation
    pending_action: PendingAction,
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
            show_goto_line: false,
            goto_line_input: String::new(),
            show_close_confirmation: false,
            pending_action: PendingAction::None,
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

    /// New document wrapper
    fn new_document(&mut self) {
        self.check_changes_before_action(PendingAction::NewDocument);
    }

    /// Open file dialog wrapper
    fn open_file_dialog(&mut self) {
        self.check_changes_before_action(PendingAction::OpenFile);
    }

    /// Open file wrapper
    fn open_file(&mut self, path: PathBuf) {
        self.check_changes_before_action(PendingAction::OpenFileFromTree(path));
    }

    /// Open directory wrapper
    fn open_directory(&mut self) {
        self.check_changes_before_action(PendingAction::OpenDirectory);
    }

    /// Change directory wrapper
    fn change_directory(&mut self, path: PathBuf) {
        self.check_changes_before_action(PendingAction::ChangeDirectory(path));
    }

    // ========================================================================
    // BRAKUJĄCE FUNKCJE POMOCNICZE I LOGIKA WYKONAWCZA
    // Wklej to wewnątrz impl EditorApp
    // ========================================================================

    /// Check for unsaved changes before action
    fn check_changes_before_action(&mut self, action: PendingAction) {
        if self.is_modified {
            self.pending_action = action;
            self.show_close_confirmation = true;
        } else {
            self.execute_pending_action(action);
        }
    }

    /// Execute pending action
    fn execute_pending_action(&mut self, action: PendingAction) {
        match action {
            PendingAction::None => {}
            PendingAction::NewDocument => self.perform_new_document(),
            PendingAction::OpenFile => self.perform_open_file_dialog(),
            PendingAction::OpenDirectory => self.perform_open_directory(),
            PendingAction::Exit => {
                // Exit is handled in update loop
            }
            PendingAction::OpenFileFromTree(path) => self.perform_open_file(path),
            PendingAction::ChangeDirectory(path) => self.perform_change_directory(path),
        }
    }

    // --- PRZYWRÓCONA LOGIKA (ZMIENIONE NAZWY NA perform_...) ---

    /// New document implementation
    fn perform_new_document(&mut self) {
        self.document = DocumentWithHistory::default();
        self.current_file_path = None;
        self.is_modified = false;
        self.status_message = "New document created".to_string();
        self.log_info("New document created");
    }

    /// Open file dialog implementation
    fn perform_open_file_dialog(&mut self) {
        self.log_info("Opening file dialog");
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("SED Files", &["sed"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.perform_open_file(path);
        } else {
            self.log_info("File dialog cancelled");
        }
    }

    /// Open file implementation
    fn perform_open_file(&mut self, path: PathBuf) {
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

    /// Open directory implementation
    fn perform_open_directory(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.log_info(format!("Opening directory: {}", path.display()));
            self.file_tree_dir = Some(path.clone());
            self.settings.last_directory = Some(path);

            // Automatically open file tree panel when directory is selected
            self.show_file_tree = true;
            self.settings.show_file_tree = true;

            let _ = self.settings.save();
            self.refresh_file_tree();
        }
    }

    /// Change directory implementation
    fn perform_change_directory(&mut self, path: PathBuf) {
        self.log_info(format!("Changing to directory: {}", path.display()));
        self.file_tree_dir = Some(path.clone());
        self.settings.last_directory = Some(path);
        let _ = self.settings.save();
        self.refresh_file_tree();
    }

    /// Refresh file tree entries
    fn refresh_file_tree(&mut self) {
        self.file_tree_entries.clear();

        let dir_opt = self.file_tree_dir.clone();
        if let Some(dir) = dir_opt {
            self.log_info(format!("Refreshing file tree for: {}", dir.display()));
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    let mut folders = Vec::new();
                    let mut files = Vec::new();

                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && self.settings.show_subfolders {
                            folders.push(FileTreeEntry::Directory(path));
                        } else if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "sed" {
                                    files.push(FileTreeEntry::File(path));
                                }
                            }
                        }
                    }

                    // Sort folders and files separately
                    folders.sort_by(|a, b| {
                        if let (FileTreeEntry::Directory(a), FileTreeEntry::Directory(b)) = (a, b) {
                            a.file_name().cmp(&b.file_name())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    files.sort_by(|a, b| {
                        if let (FileTreeEntry::File(a), FileTreeEntry::File(b)) = (a, b) {
                            a.file_name().cmp(&b.file_name())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    // Add parent directory entry if not root
                    if dir.parent().is_some() && self.settings.show_subfolders {
                        self.file_tree_entries.push(FileTreeEntry::Directory(
                            dir.parent().unwrap().to_path_buf(),
                        ));
                    }

                    self.file_tree_entries.extend(folders);
                    self.file_tree_entries.extend(files);

                    let folder_count = self
                        .file_tree_entries
                        .iter()
                        .filter(|e| matches!(e, FileTreeEntry::Directory(_)))
                        .count();
                    let file_count = self
                        .file_tree_entries
                        .iter()
                        .filter(|e| matches!(e, FileTreeEntry::File(_)))
                        .count();
                    self.log_info(format!(
                        "Found {} folders and {} .sed files",
                        folder_count, file_count
                    ));
                }
                Err(e) => {
                    self.log_error(format!("Failed to read directory: {}", e));
                }
            }
        } else {
            self.log_warning("No directory selected for file tree");
        }
    }

    /// Save file
    fn save_file(&mut self) {
        if self.keyfile_path.is_none() {
            self.status_message = "Error: No keyfile loaded".to_string();
            self.log_error("Attempted to save without keyfile");
            return;
        }

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

    /// Toggle comment on selected lines or current line
    fn toggle_comment_lines(&mut self) {
        let text = &mut self.document.current_content;
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return;
        }

        // Get cursor position or selection range
        // For now, we'll work with the entire text and detect which line to comment
        // based on highlighted_line or comment all if no highlight

        let line_to_comment = if let Some(line_num) = self.highlighted_line {
            if line_num > 0 && line_num <= lines.len() {
                Some(line_num - 1) // Convert to 0-indexed
            } else {
                None
            }
        } else {
            // If no line is highlighted, comment the first line or do nothing
            None
        };

        if line_to_comment.is_none() && lines.is_empty() {
            return;
        }

        // Process all lines
        let mut new_lines: Vec<String> = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            // Check if we should process this line
            let should_process = if let Some(target_idx) = line_to_comment {
                idx == target_idx
            } else {
                // If no specific line, comment all non-empty lines
                !line.trim().is_empty()
            };

            if should_process {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    // Uncomment: remove "//" and following space if present
                    let uncommented = if trimmed.starts_with("// ") {
                        trimmed.strip_prefix("// ").unwrap()
                    } else {
                        trimmed.strip_prefix("//").unwrap()
                    };
                    // Preserve original indentation
                    let indent = line.len() - trimmed.len();
                    new_lines.push(format!("{}{}", " ".repeat(indent), uncommented));
                } else {
                    // Comment: add "//" at start (preserving indentation)
                    let indent_count = line.len() - trimmed.len();
                    new_lines.push(format!("{}// {}", " ".repeat(indent_count), trimmed));
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        // Update document
        *text = new_lines.join("\n");
        self.is_modified = true;

        // Log action
        if let Some(line_num) = line_to_comment {
            self.log_info(format!("Toggled comment on line {}", line_num + 1));
        } else {
            self.log_info("Toggled comments");
        }
    }

    /// Render icon toolbar with hover colors
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            let button_size = egui::vec2(32.0, 32.0);
            let icon_size = egui::vec2(24.0, 24.0);
            let hover_tint = self.current_theme.colors.icon_hover_color();

            // Helper closure to render icon button with hover effect
            let icon_button = |ui: &mut egui::Ui,
                               icon: &egui::TextureHandle,
                               tooltip: &str,
                               selected: bool|
             -> egui::Response {
                let (rect, mut response) =
                    ui.allocate_exact_size(button_size, egui::Sense::click());

                if response.clicked() {
                    response.mark_changed();
                }

                // Draw background if selected
                if selected {
                    ui.painter()
                        .rect_filled(rect, 4.0, ui.visuals().widgets.active.bg_fill);
                } else if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
                }

                // Draw icon with tint on hover
                let icon_rect = egui::Rect::from_center_size(rect.center(), icon_size);
                let tint = if response.hovered() || selected {
                    hover_tint
                } else {
                    egui::Color32::WHITE
                };

                ui.painter().image(
                    icon.id(),
                    icon_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );

                response.on_hover_text(tooltip)
            };

            // --- LEFT SIDE: Main Actions ---

            if icon_button(ui, &self.icons.new_doc, "New (Ctrl+N)", false).clicked() {
                self.new_document();
            }

            if icon_button(ui, &self.icons.open, "Open (Ctrl+O)", false).clicked() {
                self.open_file_dialog();
            }

            if icon_button(ui, &self.icons.open_folder, "Open Directory", false).clicked() {
                self.open_directory();
            }

            if icon_button(ui, &self.icons.save, "Save (Ctrl+S)", false).clicked() {
                self.save_file();
            }

            if icon_button(ui, &self.icons.save_as, "Save As", false).clicked() {
                self.save_file_as();
            }

            ui.separator();
            ui.add_space(20.0);

            // --- RIGHT SIDE: Toggles ---
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if icon_button(ui, &self.icons.theme, "Theme Editor", false).clicked() {
                    self.show_theme_editor = true;
                    self.editing_theme = Some(self.current_theme.clone());
                }

                if icon_button(ui, &self.icons.settings, "Settings", false).clicked() {
                    self.show_settings_panel = true;
                }

                ui.separator();

                if icon_button(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel)
                    .clicked()
                {
                    self.show_debug_panel = !self.show_debug_panel;
                    self.settings.show_debug_panel = self.show_debug_panel;
                    let _ = self.settings.save();
                }

                if icon_button(
                    ui,
                    &self.icons.history,
                    "Toggle History",
                    self.show_history_panel,
                )
                .clicked()
                {
                    self.show_history_panel = !self.show_history_panel;
                }

                if icon_button(
                    ui,
                    &self.icons.file_tree,
                    "Toggle File Tree",
                    self.show_file_tree,
                )
                .clicked()
                {
                    self.show_file_tree = !self.show_file_tree;
                    self.settings.show_file_tree = self.show_file_tree;
                    let _ = self.settings.save();
                }
            });
        });
    }

    /// Render theme editor panel with icon hover color
    fn render_theme_editor_panel(&mut self, ui: &mut egui::Ui) {
        // Local variables to track actions (avoid borrowing issues)
        let mut action_to_perform: Option<String> = None;
        let mut theme_to_save: Option<crate::theme::Theme> = None;
        let mut should_close = false;
        let mut should_apply = false;
        let mut should_reset = false;

        ui.vertical(|ui| {
            ui.heading("🎨 Theme Editor");

            // Clone theme to work with it without long-lived mutable borrow
            if let Some(ref mut theme) = self.editing_theme {
                // Theme name
                ui.horizontal(|ui| {
                    ui.label("Theme Name:");
                    ui.text_edit_singleline(&mut theme.name);
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.heading("Colors");

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
                                theme.apply(ui.ctx());
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
                                theme.apply(ui.ctx());
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
                                theme.apply(ui.ctx());
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
                                theme.apply(ui.ctx());
                            }
                        });

                        // Cursor Color
                        ui.horizontal(|ui| {
                            ui.label("Cursor Color:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.cursor[0],
                                theme.colors.cursor[1],
                                theme.colors.cursor[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.cursor = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
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
                                theme.apply(ui.ctx());
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.label(egui::RichText::new("UI Colors").strong());

                        // Icon Hover Color
                        ui.horizontal(|ui| {
                            ui.label("Icon Hover Tint:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.icon_hover[0],
                                theme.colors.icon_hover[1],
                                theme.colors.icon_hover[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.icon_hover = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
                            }
                        });

                        // Success Color
                        ui.horizontal(|ui| {
                            ui.label("Success:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.success[0],
                                theme.colors.success[1],
                                theme.colors.success[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.success = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
                            }
                        });

                        // Info Color
                        ui.horizontal(|ui| {
                            ui.label("Info:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.info[0],
                                theme.colors.info[1],
                                theme.colors.info[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.info = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
                            }
                        });

                        // Warning Color
                        ui.horizontal(|ui| {
                            ui.label("Warning:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.warning[0],
                                theme.colors.warning[1],
                                theme.colors.warning[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.warning = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
                            }
                        });

                        // Error Color
                        ui.horizontal(|ui| {
                            ui.label("Error:");
                            let mut color = egui::Color32::from_rgb(
                                theme.colors.error[0],
                                theme.colors.error[1],
                                theme.colors.error[2],
                            );
                            if ui.color_edit_button_srgba(&mut color).changed() {
                                theme.colors.error = [color.r(), color.g(), color.b()];
                                theme.apply(ui.ctx());
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
                                theme.apply(ui.ctx());
                            }
                        });
                    });

                ui.separator();

                // Buttons - just set flags, don't mutate self
                ui.horizontal(|ui| {
                    if ui.button("💾 Save Theme").clicked() {
                        theme_to_save = Some(theme.clone());
                        action_to_perform = Some("save".to_string());
                    }

                    if ui.button("✓ Apply").clicked() {
                        should_apply = true;
                    }

                    if ui.button("🔄 Reset to Dark").clicked() {
                        should_reset = true;
                    }

                    if ui.button("✖ Close").clicked() {
                        should_close = true;
                    }
                });
            } else {
                ui.label("No theme being edited");
            }
        });

        // Now execute actions with full access to self (no borrow conflicts)
        if should_reset {
            if let Some(ref mut theme) = self.editing_theme {
                *theme = crate::theme::Theme::dark();
                theme.apply(ui.ctx());
            }
        }

        if should_apply {
            if let Some(theme) = &self.editing_theme {
                self.current_theme = theme.clone();
                self.settings.theme_name = theme.name.clone();
                self.current_theme.apply(ui.ctx());
                let _ = self.settings.save();
                self.status_message = "Theme applied".to_string();
                self.log_info("Theme applied (not saved to file)");
            }
        }

        if let Some(theme) = theme_to_save {
            match crate::theme::save_theme(&theme) {
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
            }
        }

        if should_close {
            self.show_theme_editor = false;
            self.editing_theme = None;
            self.current_theme.apply(ui.ctx());
        }
    }

    /// Render file tree panel
    /// Render file tree panel
    fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Files");

            if let Some(dir) = &self.file_tree_dir {
                ui.label(egui::RichText::new(dir.display().to_string()).small());
                ui.separator();

                // Get available width before creating ScrollArea
                let available_width = ui.available_width();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        // Set max width to prevent horizontal expansion
                        ui.set_max_width(available_width);

                        for entry in &self.file_tree_entries.clone() {
                            match entry {
                                FileTreeEntry::Directory(path) => {
                                    let is_parent = self
                                        .file_tree_dir
                                        .as_ref()
                                        .and_then(|d| d.parent())
                                        .map(|p| p == path)
                                        .unwrap_or(false);

                                    let display_name = if is_parent {
                                        "📁 ..".to_string()
                                    } else {
                                        format!(
                                            "📁 {}",
                                            path.file_name().unwrap_or_default().to_string_lossy()
                                        )
                                    };

                                    if ui.button(display_name).clicked() {
                                        self.change_directory(path.clone());
                                    }
                                }
                                FileTreeEntry::File(path) => {
                                    let filename =
                                        path.file_name().unwrap_or_default().to_string_lossy();
                                    if ui.button(format!("📄 {}", filename)).clicked() {
                                        self.open_file(path.clone());
                                    }
                                }
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

    /// Render confirmation dialog
    fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        if self.show_close_confirmation {
            egui::Modal::new(egui::Id::new("close_confirmation")).show(ctx, |ui| {
                ui.set_max_width(300.0);
                ui.heading("Unsaved Changes");
                ui.label("You have unsaved changes. Do you want to save them?");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        self.save_file();

                        // If save was successful (is_modified == false), execute pending action
                        if !self.is_modified {
                            self.show_close_confirmation = false;
                            let action = self.pending_action.clone();
                            self.pending_action = PendingAction::None;

                            if let PendingAction::Exit = action {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            } else {
                                self.execute_pending_action(action);
                            }
                        }
                    }

                    if ui.button("Don't Save").clicked() {
                        self.is_modified = false; // Force discard
                        self.show_close_confirmation = false;
                        let action = self.pending_action.clone();
                        self.pending_action = PendingAction::None;

                        if let PendingAction::Exit = action {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        } else {
                            self.execute_pending_action(action);
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        self.show_close_confirmation = false;
                        self.pending_action = PendingAction::None;
                    }
                });
            });
        }
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
                        .add(
                            egui::DragValue::new(&mut self.settings.ui_font_size)
                                .speed(0.5)
                                .range(8.0..=32.0),
                        )
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
                        .add(
                            egui::DragValue::new(&mut self.settings.editor_font_size)
                                .speed(0.5)
                                .range(8.0..=32.0),
                        )
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

                ui.separator();
                ui.heading("File Tree");

                if ui
                    .checkbox(&mut self.settings.show_subfolders, "Show subfolders")
                    .changed()
                {
                    let _ = self.settings.save();
                    self.refresh_file_tree();
                }

                ui.add_space(20.0);

                if ui.button("Close").clicked() {
                    self.show_settings_panel = false;
                }
            });
    }

    /// Render "Go to Line" dialog
    fn render_goto_line_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_goto_line {
            return;
        }

        let mut close = false;
        let mut jump_to_line: Option<usize> = None;

        egui::Window::new("🔍 Go to Line")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Line number:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.goto_line_input).desired_width(100.0),
                    );

                    // Auto-focus on open
                    response.request_focus();

                    // Check for Enter key
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                            jump_to_line = Some(line_num);
                        }
                    }
                });

                ui.horizontal(|ui| {
                    if ui.button("Go").clicked() {
                        if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                            jump_to_line = Some(line_num);
                        } else {
                            self.status_message = "Invalid line number".to_string();
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        // Handle jump outside the window closure
        if let Some(line_num) = jump_to_line {
            let max_line = self.document.current_content.lines().count().max(1);
            if line_num > 0 && line_num <= max_line {
                self.highlighted_line = Some(line_num);
                self.log_info(format!("Jumped to line {}", line_num));
                close = true;
            } else {
                self.status_message = format!("Line {} out of range (1-{})", line_num, max_line);
                self.log_warning(format!("Line {} out of range (1-{})", line_num, max_line));
            }
        }

        if close {
            self.show_goto_line = false;
            self.goto_line_input.clear();
        }
    }

    /// Advanced render with automatic current-line highlighting
    fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = text.lines().count().max(1);

        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;
        let line_number_color = self.current_theme.colors.line_number_color();
        let selection_bg_color = self.current_theme.colors.selection_color();
        let cursor_color = self.current_theme.colors.cursor_color();

        let font_id = egui::FontId::monospace(editor_font_size);
        let row_height = ui.fonts(|f| f.row_height(&font_id));

        // Calculate line number width
        let line_number_width = if show_line_numbers {
            (line_count.to_string().len() as f32 * editor_font_size * 0.6).max(50.0)
        } else {
            0.0
        };

        let mut clicked_line: Option<usize> = None;
        let mut clicked_below_content = false;

        egui::ScrollArea::both()
            .id_salt("editor_main")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

                    // === LINE NUMBERS PANEL ===
                    if show_line_numbers {
                        let panel_rect = ui
                            .allocate_ui_with_layout(
                                egui::vec2(line_number_width, line_count as f32 * row_height),
                                egui::Layout::top_down(egui::Align::RIGHT),
                                |ui| {
                                    ui.spacing_mut().item_spacing.y = 0.0;

                                    for line_num in 1..=line_count {
                                        let (rect, response) = ui.allocate_exact_size(
                                            egui::vec2(line_number_width, row_height),
                                            egui::Sense::click(),
                                        );

                                        if response.clicked() {
                                            clicked_line = Some(line_num);
                                        }

                                        // Subtle hover
                                        if response.hovered() {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                ui.visuals()
                                                    .widgets
                                                    .hovered
                                                    .bg_fill
                                                    .linear_multiply(0.2),
                                            );
                                        }

                                        // Highlight clicked line
                                        let is_selected = self.highlighted_line == Some(line_num);
                                        if is_selected {
                                            ui.painter().rect_filled(
                                                rect,
                                                0.0,
                                                cursor_color.linear_multiply(0.15),
                                            );
                                        }

                                        // Line number text - bold if selected
                                        let text_color = if is_selected {
                                            cursor_color
                                        } else {
                                            line_number_color
                                        };

                                        ui.painter().text(
                                            rect.right_center() - egui::vec2(8.0, 0.0),
                                            egui::Align2::RIGHT_CENTER,
                                            line_num.to_string(),
                                            font_id.clone(),
                                            text_color,
                                        );
                                    }
                                },
                            )
                            .response
                            .rect;

                        // Add gutter separator
                        ui.add_space(6.0);
                        let sep_rect = ui.allocate_space(egui::vec2(1.0, panel_rect.height())).1;
                        ui.painter().rect_filled(
                            sep_rect,
                            0.0,
                            ui.visuals()
                                .widgets
                                .noninteractive
                                .bg_stroke
                                .color
                                .linear_multiply(0.3),
                        );
                        ui.add_space(8.0);
                    }

                    // === TEXT EDITOR ===
                    ui.vertical(|ui| {
                        let editor_start = ui.cursor().min;

                        // Draw highlight for selected line
                        if let Some(line_num) = self.highlighted_line {
                            if line_num > 0 && line_num <= line_count {
                                let y = (line_num - 1) as f32 * row_height;
                                let width = ui.available_width().max(2000.0);

                                let highlight = egui::Rect::from_min_size(
                                    editor_start + egui::vec2(-4.0, y),
                                    egui::vec2(width, row_height),
                                );

                                ui.painter().rect_filled(
                                    highlight,
                                    2.0,
                                    selection_bg_color.linear_multiply(0.15),
                                );
                            }
                        }

                        // Text editor
                        let comment_color = self.current_theme.colors.comment_color();

                        let output = ui.add(
                            egui::TextEdit::multiline(text)
                                .desired_width(f32::INFINITY)
                                .desired_rows(line_count)
                                .font(font_id.clone())
                                .layouter(&mut |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                                    let mut layout_job = egui::text::LayoutJob::default();

                                    for line in text.split('\n') {
                                        let trimmed = line.trim_start();
                                        let is_comment = trimmed.starts_with("//");

                                        let color = if is_comment {
                                            comment_color
                                        } else {
                                            ui.visuals().text_color()
                                        };

                                        layout_job.append(
                                            line,
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color,
                                                ..Default::default()
                                            },
                                        );

                                        layout_job.append(
                                            "\n",
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                ..Default::default()
                                            },
                                        );
                                    }

                                    ui.fonts(|f| f.layout_job(layout_job))
                                })
                                .frame(false)
                                .lock_focus(true),
                        );

                        // Track changes
                        if output.changed() {
                            self.is_modified = true;
                        }

                        // Detect clicks below the last line of content
                        let editor_content_height = line_count as f32 * row_height;
                        let editor_rect = egui::Rect::from_min_size(
                            editor_start,
                            egui::vec2(
                                ui.available_width(),
                                ui.available_height().max(editor_content_height),
                            ),
                        );

                        // Check if clicked in the editor area below content
                        if ui.rect_contains_pointer(editor_rect) {
                            if ui.input(|i| i.pointer.primary_clicked()) {
                                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                                    let relative_y = pos.y - editor_start.y;
                                    if relative_y > editor_content_height {
                                        // Clicked below content - select last line
                                        clicked_below_content = true;
                                        output.request_focus();

                                        // Move caret to end
                                        if let Some(mut state) =
                                            egui::text_edit::TextEditState::load(
                                                ui.ctx(),
                                                output.id,
                                            )
                                        {
                                            let ccursor = egui::text::CCursor::new(text.len());
                                            let range = egui::text::CCursorRange::one(ccursor);
                                            state.cursor.set_char_range(Some(range));
                                            state.store(ui.ctx(), output.id);
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            });

        // Update clicked line
        if let Some(line) = clicked_line {
            self.highlighted_line = Some(line);
            self.log_info(format!("Line {} selected", line));
        } else if clicked_below_content {
            self.highlighted_line = Some(line_count);
            self.log_info(format!("Line {} selected (last line)", line_count));
        }
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle close request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.is_modified {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.check_changes_before_action(PendingAction::Exit);
            }
        }

        // Apply styles (font sizes) every frame/update
        self.apply_style(ctx);

        // Keyboard shortcuts
        ctx.input_mut(|i| {
            // Ctrl+S: Save
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::S,
            )) {
                self.save_file();
            }

            // Ctrl+O: Open
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::O,
            )) {
                self.open_file_dialog();
            }

            // Ctrl+N: New Document
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::N,
            )) {
                self.new_document();
            }

            // Ctrl+G: Go to Line
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::G,
            )) {
                self.show_goto_line = true;
            }

            // Ctrl+/: Toggle Comment
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Slash,
            )) {
                self.toggle_comment_lines();
            }
        });

        // Dialogs
        if self.show_settings_panel {
            self.render_settings_panel(ctx);
        }

        // Go to Line Dialog
        self.render_goto_line_dialog(ctx);

        // Toolbar
        egui::TopBottomPanel::top("toolbar")
            .min_height(50.0)
            .show(ctx, |ui| {
                ui.add_space(4.0);
                self.render_toolbar(ui);
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(32.0)
            .show(ctx, |ui| {
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);
                    if self.is_modified {
                        ui.label(egui::RichText::new("*").color(egui::Color32::YELLOW));
                    }

                    // Keyfile controls in bottom right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let small_btn_size = egui::vec2(20.0, 20.0); // Smaller buttons
                        let small_icon_size = egui::vec2(16.0, 16.0); // Smaller icons
                        let hover_tint = self.current_theme.colors.icon_hover_color();

                        // Helper for small icon button
                        let small_icon_btn = |ui: &mut egui::Ui,
                                              icon: &egui::TextureHandle,
                                              tooltip: &str|
                         -> egui::Response {
                            let (rect, mut response) =
                                ui.allocate_exact_size(small_btn_size, egui::Sense::click());
                            if response.clicked() {
                                response.mark_changed();
                            }
                            if response.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    3.0,
                                    ui.visuals().widgets.hovered.bg_fill,
                                );
                            }

                            let icon_rect =
                                egui::Rect::from_center_size(rect.center(), small_icon_size);
                            let tint = if response.hovered() {
                                hover_tint
                            } else {
                                egui::Color32::GRAY
                            }; // Gray when not hovered for status bar subtlety

                            ui.painter().image(
                                icon.id(),
                                icon_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                tint,
                            );
                            response.on_hover_text(tooltip)
                        };

                        if small_icon_btn(ui, &self.icons.generate, "Generate Keyfile").clicked() {
                            self.generate_new_keyfile();
                        }

                        if small_icon_btn(ui, &self.icons.key, "Load Keyfile").clicked() {
                            self.load_keyfile();
                        }

                        ui.separator();

                        // Keyfile indicator with icon
                        if let Some(path) = &self.keyfile_path {
                            let icon_tint = self.current_theme.colors.success_color();
                            let icon_rect = ui.allocate_space(small_icon_size).1;
                            ui.painter().image(
                                self.icons.secured.id(),
                                icon_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                icon_tint,
                            );
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}",
                                    path.file_name().unwrap_or_default().to_string_lossy()
                                ))
                                .color(icon_tint),
                            );
                        } else {
                            let icon_tint = self.current_theme.colors.warning_color();
                            let icon_rect = ui.allocate_space(small_icon_size).1;
                            ui.painter().image(
                                self.icons.unsecured.id(),
                                icon_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                icon_tint,
                            );
                            ui.label(egui::RichText::new("No keyfile").color(icon_tint));
                        }
                    });
                });
            });

        // File tree (left)
        if self.show_file_tree {
            egui::SidePanel::left("file_tree")
                .resizable(true)
                .default_width(self.settings.file_tree_width)
                .width_range(150.0..=500.0)
                .show(ctx, |ui| {
                    self.render_file_tree(ui);

                    // Save width if changed
                    let current_width = ui.available_width();
                    if (current_width - self.settings.file_tree_width).abs() > 1.0 {
                        self.settings.file_tree_width = current_width;
                        let _ = self.settings.save();
                    }
                });
        }

        // Theme Editor panel (right)
        if self.show_theme_editor {
            egui::SidePanel::right("theme_editor")
                .resizable(true)
                .default_width(350.0)
                .show(ctx, |ui| {
                    self.render_theme_editor_panel(ui);
                });
        }

        // History panel (right)
        if self.show_history_panel {
            egui::SidePanel::right("history")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.render_history_panel(ui);
                });
        }

        // Debug panel (right, below history if both shown)
        if self.show_debug_panel {
            egui::SidePanel::right("debug")
                .resizable(true)
                .default_width(250.0)
                .show(ctx, |ui| {
                    self.render_debug_panel(ui);
                });
        }

        self.render_confirmation_dialog(ctx);

        // Central editor
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_editor(ui);
        });
    }
}
