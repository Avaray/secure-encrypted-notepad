use eframe::egui;
use std::path::PathBuf;
use std::time::Instant;

use crate::app_state::{FileTreeEntry, LogEntry, PendingAction};
use crate::history::DocumentWithHistory;
use crate::settings::{Settings, SensitiveSettings};
use crate::theme::{load_themes, Theme};

/// Application state
pub struct EditorApp {
    /// Current document with embedded history
    pub(crate) document: DocumentWithHistory,

    /// Path to keyfile
    pub(crate) keyfile_path: Option<PathBuf>,

    /// Path to currently open file
    pub(crate) current_file_path: Option<PathBuf>,

    /// Status message
    pub(crate) status_message: String,

    /// User preferences (non-sensitive, saved as plaintext TOML)
    pub(crate) settings: Settings,

    /// Sensitive settings (keyfile path, last directory) - memory only or encrypted
    pub(crate) sensitive_settings: SensitiveSettings,

    /// Available themes
    pub(crate) themes: Vec<Theme>,

    /// Current theme
    pub(crate) current_theme: Theme,

    /// Show Settings panel
    pub(crate) show_settings_panel: bool,

    /// Show History panel
    pub(crate) show_history_panel: bool,

    /// Show Debug panel
    pub(crate) show_debug_panel: bool,

    /// Show file tree panel
    pub(crate) show_file_tree: bool,

    /// Document has been modified
    pub(crate) is_modified: bool,

    /// Debug log entries
    pub(crate) debug_log: Vec<LogEntry>,

    /// File tree current directory
    pub(crate) file_tree_dir: Option<PathBuf>,

    /// File tree entries
    pub(crate) file_tree_entries: Vec<FileTreeEntry>,

    /// Icons
    pub(crate) icons: crate::icons::Icons,

    /// Show Theme Editor panel
    pub(crate) show_theme_editor: bool,

    /// Theme being edited
    pub(crate) editing_theme: Option<Theme>,

    /// Currently highlighted line (1-indexed)
    pub(crate) highlighted_line: Option<usize>,

    /// Show goto line dialog
    pub(crate) show_goto_line: bool,

    /// Goto line input
    pub(crate) goto_line_input: String,

    /// Show close confirmation dialog
    pub(crate) show_close_confirmation: bool,

    /// Pending action to execute after confirmation
    pub(crate) pending_action: PendingAction,

    /// Text cursor range for comment toggling (char indices)
    pub(crate) text_cursor_range: Option<std::ops::Range<usize>>,

    /// Currently loaded history index (None = current document)
    pub(crate) loaded_history_index: Option<usize>,

    /// Available system fonts
    pub(crate) available_fonts: Vec<String>,

    /// Selected font index for UI
    pub(crate) ui_font_index: usize,

    /// Selected font index for Editor
    pub(crate) editor_font_index: usize,

    // Auto-save state
    pub last_autosave_time: Option<Instant>,

    // Clipboard security state
    pub last_copy_time: Option<Instant>,

    // Style dirty flag
    pub(crate) style_dirty: bool,

    // Search state
    pub show_search_panel: bool,
    pub(crate) search_query: String,
    pub(crate) replace_query: String,
    pub(crate) search_case_sensitive: bool,
    pub(crate) search_matches: Vec<usize>,       // List of match starting indices (byte offsets)
    pub(crate) current_match_index: Option<usize>, // Index into search_matches

    // Batch Converter State
    pub(crate) show_batch_converter: bool,
    pub(crate) batch_files: Vec<PathBuf>,
    pub(crate) batch_keyfile: Option<PathBuf>,
    pub(crate) batch_output_dir: Option<PathBuf>,

    // Window state tracking
    /// True only for the very first update() call — used for the first-frame maximize trick
    pub(crate) first_frame: bool,
    /// Whether the window should start maximized (from saved settings)
    pub(crate) start_maximized: bool,
    /// Current maximized state, updated every frame from the OS viewport
    pub(crate) is_maximized: bool,

    /// Flag to reset horizontal scroll to 0 on next frame (when cursor on empty line)
    pub(crate) reset_scroll_x_pending: bool,

    /// Previous cursor byte position (to detect navigation)
    pub(crate) previous_cursor_byte_pos: Option<usize>,
}

impl EditorApp {
    pub fn from_settings(settings: Settings) -> Self {
        let sensitive_settings = SensitiveSettings::default();
        let themes = load_themes();
        let available_fonts = crate::fonts::get_system_fonts();

        let ui_font_index = available_fonts
            .iter()
            .position(|f| f == &settings.ui_font_family)
            .unwrap_or(0);
        let editor_font_index = available_fonts
            .iter()
            .position(|f| f == &settings.editor_font_family)
            .unwrap_or(1);

        let current_theme = themes
            .iter()
            .find(|t| t.name == settings.theme_name)
            .cloned()
            .unwrap_or_else(|| Theme::dark());

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

        let file_tree_dir = sensitive_settings.last_directory.clone();

        Self {
            document: DocumentWithHistory::default(),
            keyfile_path,
            current_file_path: None,
            status_message: status,
            settings: settings.clone(),
            sensitive_settings,
            themes,
            current_theme,
            show_settings_panel: false,
            show_history_panel: false,
            show_debug_panel: settings.show_debug_panel,
            show_file_tree: settings.show_file_tree,
            is_modified: false,
            debug_log: Vec::new(),
            file_tree_dir,
            file_tree_entries: Vec::new(),
            icons: crate::icons::Icons::load(&egui::Context::default()),
            show_theme_editor: false,
            editing_theme: None,
            highlighted_line: None,
            show_goto_line: false,
            goto_line_input: String::new(),
            show_close_confirmation: false,
            pending_action: PendingAction::None,
            text_cursor_range: None,
            loaded_history_index: None,
            available_fonts,
            ui_font_index,
            editor_font_index,
            show_search_panel: false,
            search_query: String::new(),
            replace_query: String::new(),
            search_case_sensitive: false,
            search_matches: Vec::new(),
            current_match_index: None,
            show_batch_converter: false,
            batch_files: Vec::new(),
            batch_keyfile: None,
            batch_output_dir: None,
            first_frame: true,
            start_maximized: settings.start_maximized,
            is_maximized: false,
            last_autosave_time: None,
            last_copy_time: None,
            
            style_dirty: true, // Apply style on startup
            reset_scroll_x_pending: false,
            previous_cursor_byte_pos: None,
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::from_settings(Settings::load())
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, settings: Settings) -> Self {
        let mut app = Self::from_settings(settings);
        app.icons = crate::icons::Icons::load(&cc.egui_ctx);
        app.current_theme.apply(&cc.egui_ctx);
        app.log_info("Application started");
        app.refresh_file_tree();
        app
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update window title dynamically
        self.update_window_title(ctx);

        // ── Window state tracking ──────────────────────────────────────────
        // Read the OS-reported maximized state every frame.
        // This correctly tracks when the user clicks the Windows maximize/restore button.
        self.is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        // First-frame maximize trick: with_maximized(true) in ViewportBuilder has a
        // race condition on Windows 10/11.  Instead we send a ViewportCommand on the
        // very first frame, which fires after the window is fully created.
        if self.first_frame {
            self.first_frame = false;
            if self.start_maximized {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
        }

        // Persist window geometry changes — only save when something actually changed
        {
            let mut changed = false;

            // Track maximize state changes
            if self.is_maximized != self.settings.start_maximized {
                self.settings.start_maximized = self.is_maximized;
                changed = true;
            }

            // When NOT maximized, save position and size so we preserve the last
            // known non-maximized geometry.  Never overwrite these while maximized,
            // because the maximized rect covers the whole screen and would clobber
            // the user's preferred restored-window position/size.
            if !self.is_maximized {
                ctx.input(|i| {
                    if let Some(rect) = i.viewport().outer_rect {
                        // Only save valid positions (negative coords = off-screen/minimized)
                        if rect.min.x >= 0.0 && rect.min.y >= 0.0 {
                            if (self.settings.window_pos_x - rect.min.x).abs() > 1.0 {
                                self.settings.window_pos_x = rect.min.x;
                                changed = true;
                            }
                            if (self.settings.window_pos_y - rect.min.y).abs() > 1.0 {
                                self.settings.window_pos_y = rect.min.y;
                                changed = true;
                            }
                        }
                        if (self.settings.window_width - rect.width()).abs() > 1.0 {
                            self.settings.window_width = rect.width();
                            changed = true;
                        }
                        if (self.settings.window_height - rect.height()).abs() > 1.0 {
                            self.settings.window_height = rect.height();
                            changed = true;
                        }
                    }
                });
            }

            if changed {
                let _ = self.settings.save();
            }
        }

        // Perform auto-save check
        self.perform_autosave();

        // Check clipboard timeout
        self.check_clipboard_timeout(ctx);

        // Handle close request
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.is_modified {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.check_changes_before_action(PendingAction::Exit);
            }
        }

        // Apply styles & font sizes if dirty
        if self.style_dirty {
            self.apply_style(ctx);
            self.style_dirty = false;
        }



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

            // Ctrl+F: Find
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::F,
            )) {
                self.show_search_panel = true;
                // Focus logic will be added later or handled by egui if possible
            }

            // Ctrl+H: Replace
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::H,
            )) {
                self.show_search_panel = true;
                // TODO: set focus to replace field
            }
        });

        // Go to Line Dialog
        self.render_goto_line_dialog(ctx);

        // Confirmation dialog
        self.render_confirmation_dialog(ctx);

        // Toolbar — height/width adapts to icon size
        let toolbar_size = self.settings.toolbar_icon_size + 16.0;
        match self.settings.toolbar_position {
            crate::settings::ToolbarPosition::Top => {
                egui::TopBottomPanel::top("toolbar")
                    .exact_height(toolbar_size)
                    .show(ctx, |ui| {
                        ui.add_space(2.0);
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Left => {
                egui::SidePanel::left("toolbar")
                    .exact_width(toolbar_size)
                    .show(ctx, |ui| {
                        ui.add_space(2.0);
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Right => {
                egui::SidePanel::right("toolbar")
                    .exact_width(toolbar_size)
                    .show(ctx, |ui| {
                        ui.add_space(2.0);
                        self.render_toolbar(ui);
                    });
            }
        }
            
        // Batch Converter Window
        if self.show_batch_converter {
            self.render_batch_converter_window(ctx);
        }

        // Search panel (below toolbar)
        egui::TopBottomPanel::top("search_panel")
            .show_animated(ctx, self.show_search_panel, |ui| {
                self.render_search_panel(ui);
            });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar")
            .min_height(32.0)
            .show(ctx, |ui| {
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);

                    if self.is_modified {
                        ui.label(egui::RichText::new(" ●").color(egui::Color32::YELLOW));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Keyfile indicator with icon
                        let small_icon_size = egui::vec2(16.0, 16.0);

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

                            let status_text = if self.settings.show_keyfile_path {
                                format!("🔑 {}", path.file_name().unwrap_or_default().to_string_lossy())
                            } else {
                                "🔑 Secured".to_string()
                            };

                            ui.label(
                                egui::RichText::new(status_text)
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

                        ui.separator();

                        // File indicator
                        if let Some(path) = &self.current_file_path {
                            ui.label(format!(
                                "📄 {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        } else {
                            ui.label("📄 Unsaved document");
                        }
                    });
                });
            });

        // File tree (left)
        if self.show_file_tree {
            egui::SidePanel::left("file_tree")
                .resizable(true)
                .default_width(self.settings.file_tree_width)
                .width_range(150.0..=f32::INFINITY)
                .show(ctx, |ui| {
                    self.render_file_tree(ui);
                });
        }

        // Theme Editor panel (right)
        if self.show_theme_editor {
            egui::SidePanel::right("theme_editor")
                .resizable(true)
                .default_width(270.0)
                .show(ctx, |ui| {
                    self.render_theme_editor_panel(ui);
                });
        }

        // Settings panel (right)
        if self.show_settings_panel {
            egui::SidePanel::right("settings_panel")
                .resizable(true)
                .default_width(350.0)
                .min_width(300.0)
                .show(ctx, |ui| {
                    self.render_settings_panel(ui);
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

        // Central editor
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_editor(ui);
        });
    }
}

impl EditorApp {
    fn check_clipboard_timeout(&mut self, ctx: &egui::Context) {
        if !self.settings.clipboard_security_enabled {
            return;
        }

        if let Some(last_time) = self.last_copy_time {
            if last_time.elapsed().as_secs() >= self.settings.clipboard_clear_timeout_secs {
                // Clear clipboard
                ctx.output_mut(|o| o.copied_text = String::new());
                self.last_copy_time = None;
                self.log_info("Clipboard cleared for security");
            }
        }
    }
}
