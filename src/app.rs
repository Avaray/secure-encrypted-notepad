use eframe::egui;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use crate::app_state::{BatchMode, FileTreeEntry, KeyStatus, LogEntry, LogLevel, PendingAction};
use crate::history::DocumentWithHistory;
use crate::settings::Settings;
use crate::theme::{load_themes, Theme};

#[derive(Debug, Clone, Default)]
pub struct LayoutState {
    pub heights: std::collections::HashMap<String, f32>,
}

impl LayoutState {
    pub fn get_height(&mut self, id: &str) -> &mut f32 {
        self.heights.entry(id.to_string()).or_insert(0.0)
    }
}

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

    /// Set of files currently being checked for access in the background
    pub(crate) pending_access_checks: std::collections::HashSet<std::path::PathBuf>,

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

    /// Expanded directories in the file tree
    pub(crate) expanded_directories: std::collections::HashSet<PathBuf>,

    /// Icons
    pub(crate) icons: crate::icons::Icons,

    /// Show Theme Editor panel
    pub(crate) show_theme_editor: bool,

    /// Theme being edited
    pub(crate) editing_theme: Option<Theme>,
    /// Original theme state before editing started (used for reset check)
    pub(crate) original_editing_theme: Option<Theme>,
    /// Copied color in Theme Editor
    pub(crate) copied_color: Option<[u8; 3]>,
    /// Last copied color ID for animation
    pub(crate) last_copied_id: Option<egui::Id>,
    /// Time when the last color was copied
    pub(crate) last_copied_time: f64,

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
    pub(crate) fonts_dirty: bool,

    // Search state
    pub show_search_panel: bool,
    pub(crate) search_query: String,
    pub(crate) replace_query: String,
    pub(crate) search_case_sensitive: bool,
    pub(crate) search_matches: Vec<usize>, // List of match starting indices (byte offsets)
    pub(crate) current_match_index: Option<usize>, // Index into search_matches
    pub(crate) replace_undo_stack: Vec<String>, // Session-only undo stack for Replace ops

    // Batch Converter State
    pub(crate) show_batch_converter: bool,
    pub(crate) batch_mode: BatchMode,
    pub(crate) batch_files: Vec<PathBuf>,
    pub(crate) batch_keyfile: Option<PathBuf>,
    pub(crate) batch_keyfile_new: Option<PathBuf>,
    pub(crate) batch_output_dir: Option<PathBuf>,
    pub(crate) batch_file_access_cache: HashMap<PathBuf, KeyStatus>,
    pub(crate) batch_access_check_receiver: Option<std::sync::mpsc::Receiver<(PathBuf, KeyStatus)>>,
    pub(crate) batch_current_key_hash: Option<[u8; 32]>,
    pub(crate) batch_is_running: bool,
    pub(crate) batch_progress_count: usize,
    pub(crate) batch_total_count: usize,
    pub(crate) batch_success_count: usize,
    pub(crate) batch_failed_count: usize,
    pub(crate) batch_output_extension: String,
    pub(crate) batch_progress_receiver:
        Option<std::sync::mpsc::Receiver<crate::app_state::BatchProgressUpdate>>,

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
    /// Stored TextEdit ID for cursor manipulation
    pub(crate) text_edit_id: Option<egui::Id>,
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
    /// Auto-save restore prompt
    pub(crate) show_autosave_restore: bool,
    /// File system watcher for the current directory
    pub(crate) watcher: Option<notify::RecommendedWatcher>,
    /// Receiver for file system events
    pub(crate) watcher_receiver:
        Option<std::sync::mpsc::Receiver<Result<notify::Event, notify::Error>>>,
    /// Cached native window handle (Windows only, for SetWindowDisplayAffinity)
    #[cfg(target_os = "windows")]
    pub(crate) cached_hwnd: Option<windows_sys::Win32::Foundation::HWND>,
    /// Zen mode active
    pub(crate) zen_mode: bool,
    /// Flag to track if Zen mode fullscreen was applied at startup
    pub(crate) zen_mode_applied: bool,
    /// Confirmation for theme deletion
    pub(crate) show_delete_theme_confirmation: bool,
    /// Time when the application started (used for stable animations)
    pub(crate) start_time: Instant,
    /// File to open passed from command line (processed on first update)
    pub(crate) pending_file_to_open: Option<PathBuf>,
    /// IPC queue for single-instance file forwarding
    pub(crate) ipc_queue: Option<std::sync::Arc<std::sync::Mutex<Vec<PathBuf>>>>,
    /// Flag for About panel (F1)
    pub(crate) show_about_panel: bool,
    /// Layout hints for vertical alignment (cached heights)
    pub(crate) layout_state: LayoutState,
    /// Initial history size (to detect added snapshots for revert)
    pub(crate) initial_history_len: usize,
    /// Initial max history length (to detect changes for revert)
    pub(crate) initial_max_history_length: usize,
}

impl EditorApp {
    pub fn from_settings(mut settings: Settings) -> Self {
        let themes = load_themes();
        let available_fonts = crate::fonts::get_system_fonts();

        // Smart font detection on first run
        if settings.is_first_run {
            if let Some(font) =
                crate::fonts::detect_best_font(&available_fonts, crate::fonts::PREFERRED_UI_FONTS)
            {
                settings.ui_font_family = font;
            }
            if let Some(font) = crate::fonts::detect_best_font(
                &available_fonts,
                crate::fonts::PREFERRED_EDITOR_FONTS,
            ) {
                settings.editor_font_family = font;
            }
            let _ = settings.save();
        }

        let mut debug_log = Vec::new();
        if settings.is_first_run {
            debug_log.push(LogEntry::new(
                LogLevel::Info,
                format!("System language detected: {}", settings.language),
            ));
        }

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
            t!("app.status_ready_global").to_string()
        } else {
            t!("app.status_ready_none").to_string()
        };

        let file_tree_dir = settings.file_tree_starting_dir.clone();

        let restore_all = settings.preserve_all_panels;

        Self {
            document: DocumentWithHistory::default(),
            keyfile_path,
            current_key_hash: None,
            file_access_cache: HashMap::new(),
            access_check_receiver: None,
            pending_access_checks: std::collections::HashSet::new(),
            current_file_path: None,
            status_message: status,
            settings: settings.clone(),
            start_time: Instant::now(),
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
            zen_mode: if settings.remember_zen_mode {
                settings.zen_mode
            } else {
                false
            },
            zen_mode_applied: false,
            show_file_tree: settings.show_file_tree,
            is_modified: false,
            debug_log,
            file_tree_dir,
            file_tree_entries: Vec::new(),
            expanded_directories: std::collections::HashSet::new(),
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
            original_editing_theme: if restore_all && settings.show_theme_editor {
                Some(current_theme.clone())
            } else {
                None
            },
            copied_color: None,
            last_copied_id: None,
            last_copied_time: 0.0,

            highlighted_line: None,
            show_goto_line: false,
            goto_line_input: String::new(),
            show_about_panel: false,
            show_close_confirmation: false,
            pending_action: PendingAction::None,
            text_cursor_range: None,
            loaded_history_index: None,
            pending_file_to_open: None,
            ipc_queue: None,
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
            replace_undo_stack: Vec::new(),
            show_batch_converter: false,
            batch_mode: BatchMode::default(),
            batch_files: Vec::new(),
            batch_keyfile: None,
            batch_keyfile_new: None,
            batch_output_dir: None,
            batch_file_access_cache: HashMap::new(),
            batch_access_check_receiver: None,
            batch_current_key_hash: None,
            batch_is_running: false,
            batch_progress_count: 0,
            batch_total_count: 0,
            batch_success_count: 0,
            batch_failed_count: 0,
            batch_output_extension: settings.batch_last_extension.clone(),
            batch_progress_receiver: None,
            first_frame: true,
            start_maximized: settings.start_maximized,
            is_maximized: false,
            last_is_maximized: false,
            last_autosave_time: None,
            last_modification_time: Instant::now(),

            style_dirty: true, // Apply style on startup
            fonts_dirty: true, // Load fonts on startup
            reset_scroll_x_pending: false,
            previous_cursor_byte_pos: None,
            text_edit_id: None,
            focus_search: false,
            focused: true, // Default to focused on start
            show_reset_confirmation: false,
            reset_slider_val: 0.0,
            show_clear_keyfile_confirmation: false,
            show_delete_theme_confirmation: false,
            show_clear_backup_dir_confirmation: false,
            show_clear_workspace_confirmation: false,
            show_clear_history_confirmation: false,
            show_autosave_restore: false,
            watcher: None,
            watcher_receiver: None,
            #[cfg(target_os = "windows")]
            cached_hwnd: None,
            layout_state: LayoutState::default(),
            initial_history_len: 0,
            initial_max_history_length: 1000,
        }
    }

    /// Find the main application window by process ID (Windows only).
    /// Skips console windows to avoid targeting the wrong HWND in debug builds.
    #[cfg(target_os = "windows")]
    pub(crate) fn find_own_hwnd(&mut self) {
        use windows_sys::Win32::Foundation::{HWND, LPARAM};
        use windows_sys::Win32::System::Threading::GetCurrentProcessId;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowLongW, GetWindowThreadProcessId, IsWindowVisible,
            GWL_EXSTYLE,
        };

        const WS_EX_TOOLWINDOW: i32 = 0x00000080;
        const WS_EX_NOACTIVATE: i32 = 0x08000000;

        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> i32 {
            let mut pid: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            let our_pid = GetCurrentProcessId();
            if pid != our_pid || IsWindowVisible(hwnd) == 0 {
                return 1; // not ours or not visible, skip
            }

            // Skip console windows
            let mut class_name = [0u16; 64];
            let len = GetClassNameW(hwnd, class_name.as_mut_ptr(), 64);
            if len > 0 {
                let class_str = String::from_utf16_lossy(&class_name[..len as usize]);
                if class_str == "ConsoleWindowClass" {
                    return 1;
                }
            }

            // Skip tool windows, overlays, and non-activatable windows
            // (egui creates these for tooltips, popups, etc.)
            let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
            if ex_style & WS_EX_TOOLWINDOW != 0 || ex_style & WS_EX_NOACTIVATE != 0 {
                return 1; // skip overlays
            }

            *(lparam as *mut HWND) = hwnd;
            0 // found the main window
        }

        let mut found_hwnd: HWND = std::ptr::null_mut();
        unsafe {
            EnumWindows(Some(enum_callback), &mut found_hwnd as *mut HWND as LPARAM);
        }
        if !found_hwnd.is_null() {
            self.cached_hwnd = Some(found_hwnd);
        }
    }

    /// Apply or remove screen capture protection (Windows only).
    /// Removes WS_EX_NOREDIRECTIONBITMAP if present (needed for DWM composition),
    /// then sets display affinity via SetWindowDisplayAffinity.
    #[cfg(target_os = "windows")]
    pub(crate) fn apply_screen_capture_protection(&mut self) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongW, SetWindowDisplayAffinity, SetWindowLongW, SetWindowPos, GWL_EXSTYLE,
            SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
        };
        const WDA_NONE: u32 = 0x00000000;
        const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
        const WDA_MONITOR: u32 = 0x00000001;
        const WS_EX_NOREDIRECTIONBITMAP: i32 = 0x00200000;

        // If we don't have the HWND yet, try to find it now
        if self.cached_hwnd.is_none() {
            self.find_own_hwnd();
        }

        if let Some(hwnd) = self.cached_hwnd {
            if self.settings.screen_capture_protection {
                // Remove WS_EX_NOREDIRECTIONBITMAP if present, so DWM can protect the content
                let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
                if ex_style & WS_EX_NOREDIRECTIONBITMAP != 0 {
                    let new_style = ex_style & !WS_EX_NOREDIRECTIONBITMAP;
                    unsafe {
                        SetWindowLongW(hwnd, GWL_EXSTYLE, new_style);
                        SetWindowPos(
                            hwnd,
                            std::ptr::null_mut(),
                            0,
                            0,
                            0,
                            0,
                            SWP_FRAMECHANGED
                                | SWP_NOMOVE
                                | SWP_NOSIZE
                                | SWP_NOZORDER
                                | SWP_NOACTIVATE,
                        );
                    }
                }

                // Set display affinity (try EXCLUDE_FROM_CAPTURE, fallback to MONITOR)
                let result = unsafe { SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) };
                if result != 0 {
                    self.log_info(t!("app.log_capture_enabled"));
                } else {
                    let result2 = unsafe { SetWindowDisplayAffinity(hwnd, WDA_MONITOR) };
                    if result2 != 0 {
                        self.log_info(t!("app.log_capture_fallback"));
                    } else {
                        self.log_error(t!("app.log_capture_err"));
                    }
                }
            } else {
                // Disable protection
                let result = unsafe { SetWindowDisplayAffinity(hwnd, WDA_NONE) };
                if result != 0 {
                    self.log_info(t!("app.log_capture_disabled"));
                } else {
                    self.log_error(t!("app.log_capture_err_disable"));
                }
            }
        } else {
            self.log_error(t!("app.log_capture_err_hwnd"));
        }
    }

    /// Get displayable keyfile path/name based on settings
    pub(crate) fn mask_keyfile_path(&self, path: &std::path::Path) -> String {
        if self.settings.show_keyfile_paths {
            path.display().to_string()
        } else {
            rust_i18n::t!("settings.secured").to_string()
        }
    }

    /// Get displayable directory path/name based on settings
    pub(crate) fn mask_directory_path(&self, path: &std::path::Path) -> String {
        if self.settings.show_directory_paths {
            path.display().to_string()
        } else {
            rust_i18n::t!("settings.secured").to_string()
        }
    }
}

impl Default for EditorApp {
    fn default() -> Self {
        Self::from_settings(Settings::load())
    }
}

impl EditorApp {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        mut settings: Settings,
        file_to_open: Option<std::path::PathBuf>,
        ipc_queue: Option<std::sync::Arc<std::sync::Mutex<Vec<std::path::PathBuf>>>>,
    ) -> Self {
        let mut system_log = None;
        // On first run, detect system theme preference
        if settings.is_first_run {
            let is_dark = cc.egui_ctx.style().visuals.dark_mode;
            if !is_dark {
                settings.theme_name = "Light".to_string();
                let msg = t!("app.log_first_run_light");
                crate::sen_debug!("{}", msg);
                system_log = Some(msg.to_string());
            } else {
                let msg = t!("app.log_first_run_dark");
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
        app.log_info(t!("app.log_started", version = env!("CARGO_PKG_VERSION")));
        app.refresh_file_tree();
        app.setup_watcher();

        app.pending_file_to_open = file_to_open;
        app.ipc_queue = ipc_queue;

        app
    }

    pub(crate) fn render_status_bar(&mut self, ui: &mut egui::Ui, show_file_info: bool) {
        let fg_color = self
            .current_theme
            .colors
            .to_egui_color32(self.current_theme.colors.foreground);

        crate::app_helpers::center_row(ui, |ui| {
            // Status message on the left
            ui.label(egui::RichText::new(&self.status_message).color(fg_color));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Version info
                let version = format!("SEN {}", env!("CARGO_PKG_VERSION"));
                ui.add(
                    egui::Label::new(
                        egui::RichText::new(version).color(self.current_theme.colors.info_color()),
                    )
                    .selectable(false),
                );

                ui.separator();

                // Keyfile indicator
                if let Some(path) = &self.keyfile_path {
                    let icon_tint = if self.settings.show_keyfile_paths {
                        self.current_theme.colors.warning_color()
                    } else {
                        self.current_theme.colors.success_color()
                    };
                    let status_text = self.mask_keyfile_path(path);
                    ui.add(
                        egui::Label::new(egui::RichText::new(status_text).color(icon_tint))
                            .selectable(false),
                    );
                } else {
                    let icon_tint = self.current_theme.colors.warning_color();
                    let pulse_alpha = if self.keyfile_path.is_none() {
                        (0.1 + 0.9 * (self.start_time.elapsed().as_secs_f32() * 3.0).cos().abs())
                            as f32
                    } else {
                        1.0
                    };
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(rust_i18n::t!("app.no_keyfile"))
                                .color(icon_tint.gamma_multiply(pulse_alpha)),
                        )
                        .selectable(false),
                    );
                }

                if show_file_info {
                    ui.separator();

                    // File indicator
                    let fg_color = self
                        .current_theme
                        .colors
                        .to_egui_color32(self.current_theme.colors.foreground);

                    if let Some(path) = &self.current_file_path {
                        ui.label(
                            egui::RichText::new(
                                path.file_name().unwrap_or_default().to_string_lossy(),
                            )
                            .color(fg_color),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new(rust_i18n::t!("app.unsaved_document"))
                                .color(fg_color),
                        );
                    }

                    if self.is_modified {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new("*").color(fg_color));
                    }
                }
            });
        });
    }
}

impl eframe::App for EditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Intercept global scroll speed
        if (self.settings.scroll_speed_multiplier - 1.0).abs() > f32::EPSILON {
            let mult = self.settings.scroll_speed_multiplier;
            ctx.input_mut(|i| {
                // Modify raw_scroll_delta
                i.raw_scroll_delta *= mult;
                // Modify smooth_scroll_delta which EG 0.28+ ScrollAreas read directly
                i.smooth_scroll_delta *= mult;
                // Modify events (ScrollArea usually reads this directly)
                for event in &mut i.events {
                    if let egui::Event::MouseWheel { delta, .. } = event {
                        *delta *= mult;
                    }
                }
            });
        }

        // Detect focus loss
        let is_focused = ctx.input(|i| i.focused);
        if self.focused && !is_focused && self.settings.auto_save_on_focus_loss {
            // Focus lost: trigger immediate auto-save
            crate::sen_debug!("Focus lost: triggering auto-save");
            self.perform_autosave(true);
        }
        self.focused = is_focused;

        // Ensure smooth pulsing when locked
        if self.keyfile_path.is_none() {
            ctx.request_repaint();
        }

        // Process pending file to open from command line
        if let Some(path) = self.pending_file_to_open.take() {
            self.perform_open_file(path, true);
        }

        // Poll IPC queue for files forwarded from another instance
        if let Some(ref queue) = self.ipc_queue {
            let paths: Vec<PathBuf> = {
                if let Ok(mut q) = queue.try_lock() {
                    q.drain(..).collect()
                } else {
                    Vec::new()
                }
            };
            for path in paths {
                // Request window focus to bring it to the foreground
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx.send_viewport_cmd(egui::ViewportCommand::RequestUserAttention(
                    egui::UserAttentionType::Informational,
                ));

                if self.is_modified {
                    self.pending_action = crate::app_state::PendingAction::OpenFileFromIPC(path);
                    self.show_close_confirmation = true;
                } else {
                    self.perform_open_file(path, false);
                }
            }
        }

        // Apply Zen mode fullscreen state on first frame if enabled from settings
        if !self.zen_mode_applied {
            if self.zen_mode {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            }
            self.zen_mode_applied = true;
        }

        // Process results from background file access checks
        self.process_access_check_results(ctx);
        self.process_batch_access_check_results(ctx);
        self.process_batch_progress_results(ctx);

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

        // Try to capture our GUI window handle every frame until found (Windows only).
        // Can't be done on first_frame because the window may not be visible yet.
        #[cfg(target_os = "windows")]
        if self.cached_hwnd.is_none() {
            self.find_own_hwnd();
            if self.cached_hwnd.is_some() && self.settings.screen_capture_protection {
                self.apply_screen_capture_protection();
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

        // Sync all panel visibility to settings every frame (no disk write here)
        self.settings.show_debug_panel = self.show_debug_panel;
        self.settings.show_settings_panel = self.show_settings_panel;
        self.settings.show_history_panel = self.show_history_panel;
        self.settings.show_theme_editor = self.show_theme_editor;
        self.settings.show_search_panel = self.show_search_panel;
        self.settings.show_file_tree = self.show_file_tree;
        self.settings.zen_mode = self.zen_mode;

        // Handle close request
        if ctx.input(|i| i.viewport().close_requested()) {
            // Always save final panel state on close
            let _ = self.settings.save();

            if self.is_modified {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.check_changes_before_action(PendingAction::Exit);
            }
        }

        if self.fonts_dirty {
            self.load_custom_fonts(ctx);
            self.fonts_dirty = false;
        }

        // Apply styles & font sizes if dirty
        if self.style_dirty {
            self.apply_theme(ctx);
            self.apply_style(ctx);
            self.style_dirty = false;
        }

        let mut toggle_comment = false;

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
            }

            // Ctrl+Minus: Decrease Font
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Minus,
            )) {
                self.settings.editor_font_size =
                    (self.settings.editor_font_size - 1.0).clamp(8.0, 128.0);
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
                toggle_comment = true;
            }

            // Ctrl+F: Find
            if i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::F,
            )) {
                self.show_search_panel = !self.show_search_panel;
                self.settings.show_search_panel = self.show_search_panel;
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

                // Consume the scroll so it doesn't move the document
                i.raw_scroll_delta = egui::Vec2::ZERO;
                i.smooth_scroll_delta = egui::Vec2::ZERO;
            }
        });

        if toggle_comment {
            self.toggle_comment_lines(ctx);
        }

        // F11: Zen Mode
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            self.toggle_zen_mode(ctx);
        }

        // F1: About Panel
        if ctx.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_about_panel = !self.show_about_panel;
        }

        // ESC: Close optional overlays/panels
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.show_about_panel {
                self.show_about_panel = false;
            } else if self.show_goto_line {
                self.show_goto_line = false;
            } else if self.show_search_panel {
                self.show_search_panel = false;
                self.settings.show_search_panel = false;
            }
        }

        // About Panel (full-screen overlay)
        if self.show_about_panel {
            self.render_about_panel(ctx);
            // If the panel is open, we might not want to process other UI interaction, or we can just render it as an overlay.
            // Since egui builds bottom-up and Overlays are top, we will just render it as an egui::Window covering the screen.
        }

        // Go to Line Dialog
        self.render_goto_line_dialog(ctx);

        // Confirmation dialog
        self.render_confirmation_dialog(ctx);

        // Auto-save restore dialog
        self.render_autosave_restore_dialog(ctx);

        // Custom frame for toolbar (vertical positions)
        // Using Frame::none() to avoid hidden window_margin from egui's default side_top_panel
        let mut vertical_toolbar_frame = egui::Frame::NONE;
        vertical_toolbar_frame.fill = ctx.style().visuals.panel_fill;
        // 6px horizontal for more breathing room, 4px vertical for a slim profile
        vertical_toolbar_frame.inner_margin = egui::Margin::symmetric(6, 4);

        // Frame for horizontal bars (top toolbar, search, status)
        let mut bar_frame = egui::Frame::NONE;
        bar_frame.fill = ctx.style().visuals.panel_fill;
        bar_frame.inner_margin = egui::Margin::symmetric(6, 4);

        // Standard frame for all full-content panels (side panels, central editor)
        let mut content_frame = egui::Frame::side_top_panel(&ctx.style());
        content_frame.stroke = egui::Stroke::NONE;
        content_frame.inner_margin = egui::Margin::same(12);

        // Prep variants of the content frame that allow scrollbars to adhere to window edges
        let mut left_panel_frame = content_frame.clone();
        left_panel_frame.inner_margin.left = 8;
        // Use a 4px buffer instead of 0 to prevent oscillating hover jitter on the splitter
        left_panel_frame.inner_margin.right = 4;

        let mut right_panel_frame = content_frame.clone();
        if self.settings.toolbar_position != crate::settings::ToolbarPosition::Right {
            right_panel_frame.inner_margin.right = 4; // Buffer for inter-panel splitters
        }

        let mut central_panel_frame = content_frame.clone();
        if let Some(bg) = self.current_theme.colors.editor_background {
            central_panel_frame.fill = self.current_theme.colors.to_egui_color32(bg);
        }
        let any_right_panel = self.show_settings_panel
            || self.show_debug_panel
            || self.show_history_panel
            || self.show_theme_editor;
        if self.settings.toolbar_position != crate::settings::ToolbarPosition::Right
            && !any_right_panel
        {
            // Use a 4px buffer instead of 0 to prevent oscillating hover jitter on the boundary
            central_panel_frame.inner_margin.right = 4;
        }

        // Calculation:
        // Vertical: icon_size + 4 (button) + 12 (6+6 margins) + 1 (separator) = icon_size + 17
        let toolbar_v_size = self.settings.toolbar_icon_size + 17.0;
        // Horizontal: icon_size + 4 (button) + 8 (4+4 top/bottom margins) + 1 (separator) = icon_size + 13
        let toolbar_h_size = self.settings.toolbar_icon_size + 13.0;

        match self.settings.toolbar_position {
            crate::settings::ToolbarPosition::Top => {
                egui::TopBottomPanel::top("toolbar")
                    .frame(bar_frame.clone())
                    .exact_height(toolbar_h_size)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Left => {
                egui::SidePanel::left("toolbar")
                    .frame(vertical_toolbar_frame.clone())
                    .exact_width(toolbar_v_size)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
            crate::settings::ToolbarPosition::Right => {
                egui::SidePanel::right("toolbar")
                    .frame(vertical_toolbar_frame.clone())
                    .exact_width(toolbar_v_size)
                    .resizable(false)
                    .show(ctx, |ui| {
                        self.render_toolbar(ui);
                    });
            }
        }

        // Main content area
        if self.show_batch_converter {
            // Render global panels FIRST so they reserve space
            if !self.zen_mode {
                let mut status_bar_frame = bar_frame.clone();
                status_bar_frame.inner_margin.left = 12;
                status_bar_frame.inner_margin.right = 12;

                egui::TopBottomPanel::bottom("status_bar")
                    .frame(status_bar_frame)
                    .min_height(24.0)
                    .show(ctx, |ui| {
                        self.render_status_bar(ui, false);
                    });
            }

            // Batch Converter takes over the REMAINING central area
            self.render_batch_converter_panel(ctx);
        } else {
            // Standard Editor Mode
            // Search panel (below toolbar)
            if self.show_search_panel && !self.zen_mode {
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
            if !self.zen_mode {
                let mut status_bar_frame = bar_frame.clone();
                status_bar_frame.inner_margin.left = 12;
                status_bar_frame.inner_margin.right = 12;

                egui::TopBottomPanel::bottom("status_bar")
                    .frame(status_bar_frame)
                    .min_height(24.0)
                    .show(ctx, |ui| {
                        self.render_status_bar(ui, true);
                    });
            }
            let screen_w = ctx.viewport_rect().width();
            let max_panel_width = screen_w * 0.80;

            // File tree (left)
            if self.show_file_tree && !self.zen_mode {
                let min_tree_width = screen_w * 0.10;
                let max_tree_width = max_panel_width;
                let panel_res = egui::SidePanel::left("file_tree_panel")
                    .frame(left_panel_frame)
                    .resizable(true)
                    .min_width(min_tree_width)
                    .max_width(max_tree_width)
                    .default_width(self.settings.file_tree_width)
                    .show(ctx, |ui| {
                        let w = ui.available_width();
                        ui.set_max_width(w);
                        self.render_file_tree(ui);
                        ui.set_min_width(0.0); // zapobiega "wypychaniu" panelu przez content
                    });

                // Persist panel width when user resizes it
                let actual_width = panel_res.response.rect.width();
                if (actual_width - self.settings.file_tree_width).abs() > 1.0 {
                    self.settings.file_tree_width = actual_width;
                    let _ = self.settings.save();
                }
            }

            // Theme Editor panel (right)
            if self.show_theme_editor && !self.zen_mode {
                let panel_res = egui::SidePanel::right("theme_editor")
                    .frame(right_panel_frame.clone())
                    .resizable(true)
                    .default_width(self.settings.theme_editor_width)
                    .min_width(100.0)
                    .max_width(max_panel_width)
                    .show(ctx, |ui| {
                        self.render_theme_editor_panel(ui);
                    });

                let actual_width = panel_res.response.rect.width();
                if (actual_width - self.settings.theme_editor_width).abs() > 1.0 {
                    self.settings.theme_editor_width = actual_width.min(max_panel_width);
                    let _ = self.settings.save();
                }
            }

            // Settings panel (right)
            if self.show_settings_panel && !self.zen_mode {
                let panel_res = egui::SidePanel::right("settings_panel")
                    .frame(right_panel_frame.clone())
                    .resizable(true)
                    .default_width(self.settings.settings_panel_width)
                    .min_width(300.0)
                    .max_width(max_panel_width)
                    .show(ctx, |ui| {
                        self.render_settings_panel(ui);
                    });

                let actual_width = panel_res.response.rect.width();
                if (actual_width - self.settings.settings_panel_width).abs() > 1.0 {
                    self.settings.settings_panel_width = actual_width.min(max_panel_width);
                    let _ = self.settings.save();
                }
            }

            // History panel (right)
            if self.show_history_panel && !self.zen_mode {
                let mut history_panel_frame = right_panel_frame.clone();
                // Restore right margin for symmetry, as history has its own internal border & scrollbar
                // and should not be flush with the screen edge.
                history_panel_frame.inner_margin.right = history_panel_frame.inner_margin.left;

                let panel_res = egui::SidePanel::right("history")
                    .frame(history_panel_frame)
                    .resizable(true)
                    .default_width(self.settings.history_panel_width)
                    .max_width(max_panel_width)
                    .show(ctx, |ui| {
                        self.render_history_panel(ui);
                    });

                let actual_width = panel_res.response.rect.width();
                if (actual_width - self.settings.history_panel_width).abs() > 1.0 {
                    self.settings.history_panel_width = actual_width.min(max_panel_width);
                    let _ = self.settings.save();
                }
            }

            // Debug panel (right, below history if both shown)
            if self.show_debug_panel && !self.zen_mode {
                let panel_res = egui::SidePanel::right("debug")
                    .frame(right_panel_frame)
                    .resizable(true)
                    .default_width(self.settings.debug_panel_width)
                    .max_width(max_panel_width)
                    .show(ctx, |ui| {
                        self.render_debug_panel(ui);
                    });

                let actual_width = panel_res.response.rect.width();
                if (actual_width - self.settings.debug_panel_width).abs() > 1.0 {
                    self.settings.debug_panel_width = actual_width.min(max_panel_width);
                    let _ = self.settings.save();
                }
            }

            // Central editor
            egui::CentralPanel::default()
                .frame(central_panel_frame)
                .show(ctx, |ui| {
                    self.render_editor(ui);
                });
        }

        // --- GLOBAL KEYBOARD SHORTCUTS FALLBACK ---
        // These run AFTER the UI has been rendered. If a widget (like TextEdit)
        // had focus and consumed a shortcut, these will not trigger.
        //
        // 1. Session-only Undo for Search & Replace
        if ctx.input_mut(|i| {
            i.consume_shortcut(&egui::KeyboardShortcut::new(
                egui::Modifiers::CTRL,
                egui::Key::Z,
            ))
        }) {
            if let Some(prev_text) = self.replace_undo_stack.pop() {
                self.document.current_content = prev_text;
                self.is_modified = true;
                if self.show_search_panel {
                    self.perform_search();
                }
            }
        }
    }
}
