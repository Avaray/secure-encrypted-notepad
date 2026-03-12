use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use crate::app_state::{FileTreeEntry, KeyStatus, LogEntry, PendingAction};
use crate::history::DocumentWithHistory;
use crate::settings::Settings;
use crate::theme::{load_themes, Theme};

/// Application state
pub struct EditorApp {
    /// Current document with embedded history
    pub(crate) document: DocumentWithHistory,

    /// Path to keyfile
    pub(crate) keyfile_path: Option<PathBuf>,

    /// Cached hash of the current keyfile to avoid re-reading/re-hashing
    pub(crate) current_key_hash: Option<[u8; 32]>,

    /// Cache for file accessibility status in the file tree
    pub(crate) file_access_cache: HashMap<PathBuf, KeyStatus>,

    /// Receiver for background file access checks
    pub(crate) access_check_receiver: Option<std::sync::mpsc::Receiver<(PathBuf, KeyStatus)>>,

    /// Path to currently open file
    pub(crate) current_file_path: Option<PathBuf>,

    /// Status message
    pub(crate) status_message: String,

    /// User preferences (non-sensitive, saved as plaintext TOML)
    pub(crate) settings: Settings,

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
    pub(crate) last_modification_time: Instant,

    // Style dirty flag
    pub(crate) style_dirty: bool,

    // Search state
    pub show_search_panel: bool,
    pub(crate) search_query: String,
    pub(crate) replace_query: String,
    pub(crate) search_case_sensitive: bool,
    pub(crate) search_matches: Vec<usize>, // List of match starting indices (byte offsets)
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
    /// Maximized state from the previous frame to detect changes
    pub(crate) last_is_maximized: bool,

    /// Flag to reset horizontal scroll to 0 on next frame (when cursor on empty line)
    pub(crate) reset_scroll_x_pending: bool,

    /// Previous cursor byte position (to detect navigation)
    pub(crate) previous_cursor_byte_pos: Option<usize>,
    /// Flag to trigger focus on search field
    pub(crate) focus_search: bool,
    /// Track actual window focus to trigger auto-save on focus loss
    pub(crate) focused: bool,

    // Settings Reset State
    pub(crate) show_reset_confirmation: bool,
    pub(crate) reset_slider_val: f32,

    // Global Keyfile Clear Confirmation State
    pub(crate) show_clear_keyfile_confirmation: bool,
    /// Backup Directory Clear Confirmation State
    pub(crate) show_clear_backup_dir_confirmation: bool,
    /// Workspace Clear Confirmation State
    pub(crate) show_clear_workspace_confirmation: bool,
    /// History Clear Confirmation State
    pub(crate) show_clear_history_confirmation: bool,
    /// File system watcher for the current directory
    pub(crate) watcher: Option<notify::RecommendedWatcher>,
    /// Receiver for file system events
    pub(crate) watcher_receiver: Option<std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>>,
}

impl EditorApp {
    pub fn from_settings(settings: Settings) -> Self {
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

        let file_tree_dir = settings.file_tree_starting_dir.clone();

        let restore_all = settings.preserve_all_panels;

        Self {
            document: DocumentWithHistory::default(),
            keyfile_path,
            current_key_hash: None,
            file_access_cache: HashMap::new(),
            access_check_receiver: None,
            current_file_path: None,
            status_message: status,
            settings: settings.clone(),

            themes,
            current_theme: current_theme.clone(),
            show_settings_panel: if restore_all {
                settings.show_settings_panel
            } else {
                false
            },
            show_history_panel: if restore_all {
                settings.show_history_panel
            } else {
                false
            },
            show_debug_panel: if restore_all {
                settings.show_debug_panel
            } else {
                false
            },
            show_file_tree: settings.show_file_tree,
            is_modified: false,
            debug_log: Vec::new(),
            file_tree_dir,
            file_tree_entries: Vec::new(),
            icons: crate::icons::Icons::load(&egui::Context::default()),
            show_theme_editor: if restore_all {
                settings.show_theme_editor
            } else {
                false
            },
            editing_theme: if restore_all && settings.show_theme_editor {
                Some(current_theme.clone())
            } else {
                None
            },
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
            show_search_panel: if restore_all {
                settings.show_search_panel
            } else {
                false
            },
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
            last_is_maximized: false,
            last_autosave_time: None,
            last_modification_time: Instant::now(),

            style_dirty: true, // Apply style on startup
            reset_scroll_x_pending: false,
            previous_cursor_byte_pos: None,
            focus_search: false,
            focused: true, // Default to focused on start
            show_reset_confirmation: false,
            reset_slider_val: 0.0,
            show_clear_keyfile_confirmation: false,
            show_clear_backup_dir_confirmation: false,
            show_clear_workspace_confirmation: false,
            show_clear_history_confirmation: false,
            watcher: None,
            watcher_receiver: None,
        }
    }

    /// Get displayable keyfile path/name based on settings
    pub(crate) fn mask_keyfile_path(&self, path: &std::path::Path) -> String {
        if self.settings.show_keyfile_paths {
            path.display().to_string()
        } else {
            "Secured".to_string()
        }
    }

    /// Get displayable directory path/name based on settings
    pub(crate) fn mask_directory_path(&self, path: &std::path::Path) -> String {
        if self.settings.show_directory_paths {
            path.display().to_string()
        } else {
            "Secured".to_string()
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::from_settings(Settings::load())
    }
}

impl EditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>, mut settings: Settings) -> Self {
        let mut system_log = None;
        // On first run, detect system theme preference
        if settings.is_first_run {
            let is_dark = cc.egui_ctx.style().visuals.dark_mode;
            if !is_dark {
                settings.theme_name = "Light".to_string();
                let msg = "First run: System detected Light mode, setting theme to Light";
                crate::sen_debug!("{}", msg);
                system_log = Some(msg.to_string());
            } else {
                let msg = "First run: System detected Dark mode (default theme)";
                crate::sen_debug!("{}", msg);
                system_log = Some(msg.to_string());
            }
        }

        let mut app = Self::from_settings(settings);

        if let Some(msg) = system_log {
            app.log_info(msg);
        }

        app.icons = crate::icons::Icons::load(&cc.egui_ctx);
        app.current_theme.apply(&cc.egui_ctx);
        app.log_info(format!(
            "Application started (v{})",
            env!("CARGO_PKG_VERSION")
        ));
        app.refresh_file_tree();
        app.setup_watcher();
        app
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Detect focus loss
        let is_focused = ctx.input(|i| i.focused);
        if self.focused && !is_focused && self.settings.auto_save_on_focus_loss {
            // Focus lost: trigger immediate auto-save
            crate::sen_debug!("Focus lost: triggering auto-save");
            self.perform_autosave(true);
        }
        self.focused = is_focused;

        // Process results from background file access checks
        self.process_access_check_results(ctx);

        // Process file system events for file tree
        if let Some(rx) = &self.watcher_receiver {
            let mut refresh_needed = false;
            while let Ok(res) = rx.try_recv() {
                match res {
                    Ok(event) => {
                        // Refresh on any change that isn't just a simple access
                        if !event.kind.is_access() {
                            refresh_needed = true;
                        }
                    }
                    Err(_) => {
                        // On error, we might want to refresh just in case, or do nothing
                    }
                }
            }
            if refresh_needed {
                self.refresh_file_tree();
                ctx.request_repaint();
            }
        }

        // Update window title dynamically
        self.update_window_title(ctx);

        // ── Window state tracking ──────────────────────────────────────────
        // Read the OS-reported maximized state every frame.
        // This correctly tracks when the user clicks the Windows maximize/restore button.
        self.last_is_maximized = self.is_maximized;
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
            // If the window state changed (user maximized/restored via OS), sync with settings
            if self.is_maximized != self.last_is_maximized {
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
        self.perform_autosave(false);

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

            // Ctrl+Shift+O: Open Directory
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::O,
            )) {
                self.open_directory();
            }

            // Ctrl+O: Open
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::O,
            )) {
                self.open_file_dialog();
            }

            // Ctrl+Plus: Increase Font
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Plus,
            )) || i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Equals,
            )) {
                self.settings.editor_font_size =
                    (self.settings.editor_font_size + 1.0).clamp(8.0, 128.0);
                let _ = self.settings.save();
            }

            // Ctrl+Minus: Decrease Font
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Minus,
            )) {
                self.settings.editor_font_size =
                    (self.settings.editor_font_size - 1.0).clamp(8.0, 128.0);
                let _ = self.settings.save();
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
                self.show_search_panel = !self.show_search_panel;
                if self.show_search_panel {
                    self.focus_search = true;
                }
            }

            // Ctrl+H: Replace
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::H,
            )) {
                self.show_search_panel = true;
                // TODO: set focus to replace field
            }

            // Ctrl+Scroll: Zoom Font
            if i.modifiers.command
                && (i.raw_scroll_delta.y != 0.0 || i.smooth_scroll_delta.y != 0.0)
            {
                let scroll_y = i.raw_scroll_delta.y + i.smooth_scroll_delta.y;
                let zoom_speed = 0.05;
                let delta = scroll_y * zoom_speed;

                self.settings.editor_font_size =
                    (self.settings.editor_font_size + delta).clamp(8.0, 128.0);
                let _ = self.settings.save();

                // Consume the scroll so it doesn't move the document
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
            }
        });

        // Go to Line Dialog
        self.render_goto_line_dialog(ctx);

        // Confirmation dialog
        self.render_confirmation_dialog(ctx);

        // Custom frame for toolbar (vertical positions)
        let mut vertical_toolbar_frame = egui::Frame::side_top_panel(&ctx.style());
        vertical_toolbar_frame.stroke = egui::Stroke::NONE;
        vertical_toolbar_frame.inner_margin = egui::Margin {
            left: 6,
            right: 6,
            top: 6,
            bottom: 6,
        };

        // Frame for horizontal bars (top toolbar, search, status) - slim vertical, wide horizontal
        let mut bar_frame = egui::Frame::side_top_panel(&ctx.style());
        bar_frame.stroke = egui::Stroke::NONE;
        bar_frame.outer_margin = egui::Margin::ZERO;
        bar_frame.shadow = egui::Shadow::NONE;
        bar_frame.inner_margin = egui::Margin {
            left: 6,
            right: 6,
            top: 6,
            bottom: 6,
        };

        // Standard frame for all full-content panels (side panels, central editor)
        let mut content_frame = egui::Frame::side_top_panel(&ctx.style());
        content_frame.stroke = egui::Stroke::NONE;
        content_frame.inner_margin = egui::Margin::same(12);

        // Button size in the toolbar is (ico_s + 4). Each frame adds its inner_margin
        // on both sides to arrive at the total panel dimension:
        //   bar_frame (Top):            top:6 + bottom:6 = 12  →  ico_s + 4 + 12 = ico_s + 16
        //   vertical_toolbar_frame (Left/Right): left:6 + right:6 = 12  →  ico_s + 4 + 12 = ico_s + 16
        let toolbar_size_h = self.settings.toolbar_icon_size + 16.0; // Top
        let toolbar_size_v = self.settings.toolbar_icon_size + 16.0; // Left / Right

        match self.settings.toolbar_position {
            crate::settings::ToolbarPosition::Top => {
                egui::TopBottomPanel::top("toolbar")
                    .frame(bar_frame.clone())
                    .exact_height(toolbar_size_h)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Left => {
                egui::SidePanel::left("toolbar")
                    .frame(vertical_toolbar_frame.clone())
                    .exact_width(toolbar_size_v)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Right => {
                egui::SidePanel::right("toolbar")
                    .frame(vertical_toolbar_frame.clone())
                    .exact_width(toolbar_size_v)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
        }

        // Batch Converter Window
        if self.show_batch_converter {
            self.render_batch_converter_window(ctx);
        }

        // Search panel (below toolbar)
        if self.show_search_panel {
            let mut search_bar_frame = bar_frame.clone();
            search_bar_frame.inner_margin.left = 12;
            search_bar_frame.inner_margin.right = 12;

            egui::TopBottomPanel::top("search_panel")
                .frame(search_bar_frame)
                .min_height(0.0)
                .show(ctx, |ui| {
                    self.render_search_panel(ui);
                });
        }

        // Status bar
        let mut status_bar_frame = bar_frame.clone();
        status_bar_frame.inner_margin.left = 12;
        status_bar_frame.inner_margin.right = 12;

        egui::TopBottomPanel::bottom("status_bar")
            .frame(status_bar_frame)
            .min_height(24.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&self.status_message);

                    if self.is_modified {
                        ui.label(egui::RichText::new(" *").color(egui::Color32::YELLOW));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Version info
                        let version = format!("SEN {}", env!("CARGO_PKG_VERSION"));
                        ui.add(
                            egui::Label::new(
                                egui::RichText::new(version)
                                    .color(self.current_theme.colors.info_color()),
                            )
                            .selectable(false),
                        );

                        ui.separator();

                        // Keyfile indicator
                        if let Some(path) = &self.keyfile_path {
                            let icon_tint = self.current_theme.colors.success_color();
                            let status_text = self.mask_keyfile_path(path);
                            ui.add(
                                egui::Label::new(egui::RichText::new(status_text).color(icon_tint))
                                    .selectable(false),
                            );
                        } else {
                            let icon_tint = self.current_theme.colors.warning_color();
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new("No keyfile").color(icon_tint),
                                )
                                .selectable(false),
                            );
                        }

                        ui.separator();

                        // File indicator
                        if let Some(path) = &self.current_file_path {
                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                        } else {
                            ui.label("Unsaved document");
                        }
                    });
                });
            });

        // File tree (left)
        if self.show_file_tree {
            let panel_res = egui::SidePanel::left("file_tree")
                .frame(content_frame.clone())
                .resizable(true)
                .default_width(self.settings.file_tree_width)
                .width_range(150.0..=f32::INFINITY)
                .show(ctx, |ui| {
                    self.render_file_tree(ui);
                });

            // Persist panel width when user resizes it
            let actual_width = panel_res.response.rect.width();
            if (actual_width - self.settings.file_tree_width).abs() > 1.0 {
                self.settings.file_tree_width = actual_width;
                let _ = self.settings.save();
            }
        }

        // Theme Editor panel (right)
        if self.show_theme_editor {
            let panel_res = egui::SidePanel::right("theme_editor")
                .frame(content_frame.clone())
                .resizable(true)
                .default_width(self.settings.theme_editor_width)
                .show(ctx, |ui| {
                    self.render_theme_editor_panel(ui);
                });

            let actual_width = panel_res.response.rect.width();
            if (actual_width - self.settings.theme_editor_width).abs() > 1.0 {
                self.settings.theme_editor_width = actual_width;
                let _ = self.settings.save();
            }
        }

        // Settings panel (right)
        if self.show_settings_panel {
            let panel_res = egui::SidePanel::right("settings_panel")
                .frame(content_frame.clone())
                .resizable(true)
                .default_width(self.settings.settings_panel_width)
                .min_width(300.0)
                .show(ctx, |ui| {
                    self.render_settings_panel(ui);
                });

            let actual_width = panel_res.response.rect.width();
            if (actual_width - self.settings.settings_panel_width).abs() > 1.0 {
                self.settings.settings_panel_width = actual_width;
                let _ = self.settings.save();
            }
        }

        // History panel (right)
        if self.show_history_panel {
            let panel_res = egui::SidePanel::right("history")
                .frame(content_frame.clone())
                .resizable(true)
                .default_width(self.settings.history_panel_width)
                .show(ctx, |ui| {
                    self.render_history_panel(ui);
                });

            let actual_width = panel_res.response.rect.width();
            if (actual_width - self.settings.history_panel_width).abs() > 1.0 {
                self.settings.history_panel_width = actual_width;
                let _ = self.settings.save();
            }
        }

        // Debug panel (right, below history if both shown)
        if self.show_debug_panel {
            let panel_res = egui::SidePanel::right("debug")
                .frame(content_frame.clone())
                .resizable(true)
                .default_width(self.settings.debug_panel_width)
                .show(ctx, |ui| {
                    self.render_debug_panel(ui);
                });

            let actual_width = panel_res.response.rect.width();
            if (actual_width - self.settings.debug_panel_width).abs() > 1.0 {
                self.settings.debug_panel_width = actual_width;
                let _ = self.settings.save();
            }
        }

        // Central editor
        egui::CentralPanel::default()
            .frame(content_frame.clone())
            .show(ctx, |ui| {
                self.render_editor(ui);
            });
    }
}
