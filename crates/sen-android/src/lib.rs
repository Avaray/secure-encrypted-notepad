use android_activity::AndroidApp;

/// Custom debug logging macro that only prints to console in debug builds.
/// This prevents sensitive data leakage in release versions.
#[allow(unused_macros)]
macro_rules! sen_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(debug_assertions)]
            eprintln!("[SEN-ANDROID] {}", format!($($arg)*));

            #[cfg(not(debug_assertions))]
            if false {
                let _ = format_args!($($arg)*);
            }
        }
    };
}

/// Wrapper around the localized translate macro.
macro_rules! t {
    ($key:expr) => {
        sen_translations::_rust_i18n_translate(&*sen_translations::locale(), $key)
    };
    ($key:expr, $($name:ident = $val:expr),+ $(,)?) => {
        {
            let mut s = sen_translations::_rust_i18n_translate(&*sen_translations::locale(), $key).into_owned();
            $(
                s = s.replace(concat!("%{", stringify!($name), "}"), &format!("{}", $val));
            )+
            std::borrow::Cow::<'static, str>::Owned(s)
        }
    };
}
// pub(crate) use sen_debug;
use eframe::egui;
use jni::objects::{JByteArray, JObject, JString, JValue};
use jni::sys::jint;
use jni::JavaVM;
use sen_core::models::FileTreeEntry;
use sen_core::settings::Settings;
use sen_core::theme::{load_themes, Theme};
use sen_core::theme_egui::{ThemeColorsExt, ThemeExt};
use std::sync::{Mutex, OnceLock};
// use serde::{Deserialize, Serialize};

pub mod android_fs;
pub mod icons;

#[derive(Clone)]
enum InjectedInputEvent {
    Text(String),
    Key(egui::Key),
}

static INJECTED_INPUT_QUEUE: OnceLock<Mutex<Vec<InjectedInputEvent>>> = OnceLock::new();

fn injected_input_queue() -> &'static Mutex<Vec<InjectedInputEvent>> {
    INJECTED_INPUT_QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    let app_clone = app.clone();
    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(app);

    let _ = eframe::run_native(
        "SEN Android",
        options,
        Box::new(|cc| Ok(Box::new(SenAndroidApp::new(cc, app_clone)))),
    );
}

// ──────────────────────────────────────────────
// JNI Exports (Called by Kotlin)
// ──────────────────────────────────────────────

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverOpenFile(
    mut env: jni::JNIEnv,
    _class: JObject,
    data: JByteArray,
    name: JString,
) {
    let data_bytes: Vec<u8> = env.convert_byte_array(&data).unwrap_or_default();
    let name_str: String = env
        .get_string(&name)
        .map(|s| s.into())
        .unwrap_or_else(|_| "unknown.sen".to_string());

    android_fs::jni_deliver_open_file(data_bytes, name_str);
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverSaveUri(
    mut env: jni::JNIEnv,
    _class: JObject,
    uri: JString,
) {
    let uri_str: String = env.get_string(&uri).map(|s| s.into()).unwrap_or_default();
    android_fs::jni_deliver_save_uri(uri_str);
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverKeyfile(
    mut env: jni::JNIEnv,
    _class: JObject,
    uri: JString,
    data: JByteArray,
) {
    let uri_str: String = env.get_string(&uri).map(|s| s.into()).unwrap_or_default();
    let data_bytes: Vec<u8> = env.convert_byte_array(&data).unwrap_or_default();

    android_fs::jni_deliver_keyfile(uri_str, data_bytes);
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverDirectoryUri(
    mut env: jni::JNIEnv,
    _class: JObject,
    uri: JString,
) {
    let uri_str: String = env.get_string(&uri).map(|s| s.into()).unwrap_or_default();
    android_fs::jni_deliver_directory_uri(uri_str);
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverBiometricResult(
    _env: jni::JNIEnv,
    _class: JObject,
    success: jni::sys::jboolean,
) {
    android_fs::jni_deliver_biometric_result(success != 0);
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeAppPaused(
    _env: jni::JNIEnv,
    _class: JObject,
) {
    android_fs::jni_app_paused();
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverTypedText(
    mut env: jni::JNIEnv,
    _class: JObject,
    text: JString,
) {
    let text_str: String = env.get_string(&text).map(|s| s.into()).unwrap_or_default();
    if text_str.is_empty() {
        return;
    }
    if let Ok(mut queue) = injected_input_queue().lock() {
        queue.push(InjectedInputEvent::Text(text_str));
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sen_android_MainActivity_nativeDeliverSpecialKey(
    _env: jni::JNIEnv,
    _class: JObject,
    key_code: jint,
) {
    const KEYCODE_ENTER: jint = 66;
    const KEYCODE_DEL: jint = 67;
    const KEYCODE_FORWARD_DEL: jint = 112;
    const KEYCODE_NUMPAD_ENTER: jint = 160;

    let mapped_key = match key_code {
        KEYCODE_ENTER | KEYCODE_NUMPAD_ENTER => Some(egui::Key::Enter),
        KEYCODE_DEL => Some(egui::Key::Backspace),
        KEYCODE_FORWARD_DEL => Some(egui::Key::Delete),
        _ => None,
    };

    if let Some(key) = mapped_key {
        if let Ok(mut queue) = injected_input_queue().lock() {
            queue.push(InjectedInputEvent::Key(key));
        }
    }
}

fn find_urls(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut urls = Vec::new();
    let mut start = 0;
    while let Some(idx) = text[start..]
        .find("http://")
        .or_else(|| text[start..].find("https://"))
    {
        let url_start = start + idx;
        let mut url_end = url_start;
        for (i, c) in text[url_start..].char_indices() {
            if c.is_whitespace()
                || c == '<'
                || c == '>'
                || c == '"'
                || c == '\''
                || c == '`'
                || c == ')'
            {
                url_end = url_start + i;
                break;
            } else {
                url_end = url_start + i + c.len_utf8();
            }
        }
        urls.push(url_start..url_end);
        start = url_end;
    }
    urls
}

// ──────────────────────────────────────────────
// Application State
// ──────────────────────────────────────────────

struct SenAndroidApp {
    settings: Settings,
    icons: Option<icons::Icons>,

    /// Current document with history
    document: sen_core::history::DocumentWithHistory,

    is_modified: bool,
    current_file_name: Option<String>,
    current_file_uri: Option<String>,

    /// Available themes loaded at startup
    available_themes: Vec<Theme>,
    /// Current applied theme
    current_theme: Theme,

    /// Last selected keyfile (persistent during session)
    keyfile_content: Vec<u8>,
    keyfile_uri: Option<String>,

    /// File tree state
    file_tree_entries: Vec<FileTreeEntry>,
    selected_directory_uri: Option<String>,

    // UI state
    show_menu: bool,
    show_settings: bool,
    show_file_tree: bool,
    show_history: bool,
    loaded_history_index: Option<usize>,

    // Stealth state
    stealth_mode: bool,
    _stealth_scan: bool,

    // Search state
    search_query: String,
    search_matches: Vec<usize>,
    show_search: bool,

    // Biometric state
    biometric_enabled: bool,
    is_locked: bool,

    /// Tracks whether we launched a SAF picker (to suppress biometric re-lock)
    picker_active: bool,

    notification: Option<(String, f64)>, // (message, expire_time)
    last_autosave_time: f64,

    android_app: AndroidApp,
}

impl SenAndroidApp {
    pub fn new(cc: &eframe::CreationContext<'_>, android_app: AndroidApp) -> Self {
        let config_dir = android_app.internal_data_path();
        let settings = Settings::load(config_dir);

        // Initialize localization
        sen_translations::set_locale(&settings.language);

        let icons = Some(icons::Icons::load(&cc.egui_ctx));
        let available_themes = load_themes();

        let current_theme = available_themes
            .iter()
            .find(|t| t.name == settings.theme_name)
            .cloned()
            .unwrap_or_else(|| Theme::dark());

        let density = {
            let vm = unsafe { jni::JavaVM::from_raw(android_app.vm_as_ptr() as *mut _) }.unwrap();
            let mut env = vm.attach_current_thread().unwrap();
            let activity =
                unsafe { jni::objects::JObject::from_raw(android_app.activity_as_ptr() as *mut _) };
            match env.call_method(&activity, "getScreenDensity", "()F", &[]) {
                Ok(val) => val.f().unwrap_or(1.0),
                Err(_) => 1.0,
            }
        };
        cc.egui_ctx.set_pixels_per_point(density);

        // Apply theme immediately
        current_theme.apply(&cc.egui_ctx);

        let mut app = Self {
            settings: settings.clone(),
            icons,
            document: sen_core::history::DocumentWithHistory::default(),
            is_modified: false,
            current_file_name: None,
            current_file_uri: None,
            available_themes,
            current_theme,
            keyfile_content: Vec::new(),
            keyfile_uri: None,
            file_tree_entries: Vec::new(),
            selected_directory_uri: None,

            show_menu: false,
            show_settings: false,
            show_file_tree: false,
            show_history: false,
            loaded_history_index: None,

            stealth_mode: false,
            _stealth_scan: false,

            search_query: String::new(),
            search_matches: Vec::new(),
            show_search: false,

            biometric_enabled: settings.biometric_unlock,
            is_locked: settings.biometric_unlock, // Only start locked if biometric is enabled

            picker_active: false,

            notification: None,
            last_autosave_time: 0.0,
            android_app,
        };

        // Apply screen capture protection immediately
        app.call_java_set_screen_protection(app.settings.screen_capture_protection);

        // Attempt to auto-load keyfile if enabled
        if app.settings.use_global_keyfile {
            if let Some(uri_path) = &app.settings.global_keyfile_path {
                let uri_str = uri_path.to_string_lossy().to_string();
                if let Some(data) = app.call_java_read_bytes(&uri_str) {
                    app.keyfile_content = data;
                    app.keyfile_uri = Some(uri_str);
                    sen_debug!("Auto-loaded keyfile from SAF URI");
                }
            }
        }

        // Attempt to auto-load workspace
        if let Some(dir_path) = &app.settings.file_tree_starting_dir {
            let uri_str = dir_path.to_string_lossy().to_string();
            app.selected_directory_uri = Some(uri_str);
            app.refresh_file_tree();
            app.show_file_tree = true;
            sen_debug!("Auto-loaded workspace from SAF URI");
        }

        app
    }

    fn notify(&mut self, ctx: &egui::Context, msg: impl Into<String>) {
        self.notification = Some((msg.into(), ctx.input(|i| i.time) + 3.0));
    }
}

// ──────────────────────────────────────────────
// Action Implementations (Calling Java)
// ──────────────────────────────────────────────

impl SenAndroidApp {
    fn call_java_void(&self, method: &str) {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let _ = env.call_method(&activity, method, "()V", &[]);
    }

    fn call_java_open_directory(&self) {
        self.call_java_void("openDirectoryPicker");
    }

    fn call_java_save_picker(&self, suggested_name: &str) {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let name_jstr = env
            .new_string(suggested_name)
            .expect("Failed to create JString");
        let _ = env.call_method(
            &activity,
            "saveFilePicker",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&name_jstr)],
        );
    }

    fn call_java_write_bytes(&self, uri: &str, data: &[u8]) -> bool {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let uri_jstr = env.new_string(uri).expect("Failed to create JString");
        let data_jbyte = env
            .byte_array_from_slice(data)
            .expect("Failed to create JByteArray");

        match env.call_method(
            &activity,
            "writeToFileUri",
            "(Ljava/lang/String;[B)Z",
            &[JValue::Object(&uri_jstr), JValue::Object(&data_jbyte)],
        ) {
            Ok(val) => val.z().unwrap_or(false),
            Err(_) => false,
        }
    }

    fn call_java_list_directory(&self, uri: &str) -> Vec<FileTreeEntry> {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let uri_jstr = env.new_string(uri).expect("Failed to create JString");
        match env.call_method(
            &activity,
            "listDirectory",
            "(Ljava/lang/String;)Ljava/lang/String;",
            &[JValue::Object(&uri_jstr)],
        ) {
            Ok(val) => {
                let jstr = val.l().unwrap().into();
                let rust_str: String = env.get_string(&jstr).unwrap().into();
                serde_json::from_str(&rust_str).unwrap_or_default()
            }
            Err(_) => Vec::new(),
        }
    }

    fn call_java_read_bytes(&self, uri: &str) -> Option<Vec<u8>> {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let uri_jstr = env.new_string(uri).expect("Failed to create JString");
        match env.call_method(
            &activity,
            "readBytesFromUri",
            "(Ljava/lang/String;)[B",
            &[JValue::Object(&uri_jstr)],
        ) {
            Ok(val) => {
                let jobj = val.l().unwrap();
                if jobj.is_null() {
                    None
                } else {
                    Some(
                        env.convert_byte_array(JByteArray::from(jobj))
                            .unwrap_or_default(),
                    )
                }
            }
            Err(_) => None,
        }
    }

    fn call_java_set_screen_protection(&self, enabled: bool) {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let _ = env.call_method(
            &activity,
            "setScreenCaptureProtection",
            "(Z)V",
            &[JValue::Bool(if enabled { 1 } else { 0 })],
        );
    }

    fn call_java_show_biometric_prompt(&self) {
        let vm = unsafe { JavaVM::from_raw(self.android_app.vm_as_ptr() as *mut _) }.unwrap();
        let mut env = vm.attach_current_thread().unwrap();
        let activity = unsafe { JObject::from_raw(self.android_app.activity_as_ptr() as *mut _) };

        let _ = env.call_method(&activity, "showBiometricPrompt", "()V", &[]);
    }

    fn action_new_file(&mut self) {
        self.document = sen_core::history::DocumentWithHistory::default();
        self.is_modified = false;
        self.current_file_name = None;
        self.current_file_uri = None;
        self.loaded_history_index = None;
    }

    fn action_open_file(&mut self) {
        self.picker_active = true;
        self.call_java_void("openFilePicker");
    }

    fn action_save_file(&mut self, ctx: &egui::Context) {
        if let Some(uri) = self.current_file_uri.clone() {
            self.perform_save_to_uri(&uri, ctx);
        } else {
            let suggested = self
                .current_file_name
                .clone()
                .unwrap_or_else(|| "note.sen".to_string());
            self.picker_active = true;
            self.call_java_save_picker(&suggested);
        }
    }

    fn action_select_keyfile(&mut self) {
        self.picker_active = true;
        self.call_java_void("selectKeyfile");
    }

    /// Renders a standard panel header with a title and an 'X' close button.
    /// Returns true if the close button was clicked.
    fn render_panel_header(icons: Option<&icons::Icons>, ui: &mut egui::Ui, title: &str) -> bool {
        let mut closed = false;
        ui.horizontal(|ui| {
            ui.heading(title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(icons) = icons {
                    let btn_size = egui::vec2(24.0, 24.0);
                    if ui
                        .add(
                            egui::Button::image(
                                egui::Image::new(&icons.close).fit_to_exact_size(btn_size),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        closed = true;
                    }
                } else {
                    if ui.button("X").clicked() {
                        closed = true;
                    }
                }
            });
        });
        ui.separator();
        closed
    }

    fn perform_save_to_uri(&mut self, uri: &str, ctx: &egui::Context) {
        if self.keyfile_content.is_empty() {
            self.notify(ctx, "Please select a keyfile first");
            return;
        }

        // Add history snapshot before saving
        self.document.add_snapshot(None);

        // Get content string (including history)
        let content_to_encrypt = self.document.to_file_content();

        let result = if self.stealth_mode {
            sen_core::crypto::encrypt_stealth_bytes(
                content_to_encrypt.as_bytes(),
                &self.keyfile_content,
            )
        } else {
            sen_core::crypto::encrypt_content_bytes(
                content_to_encrypt.as_bytes(),
                &self.keyfile_content,
            )
        };

        match result {
            Ok(bytes) => {
                if self.call_java_write_bytes(uri, &bytes) {
                    self.is_modified = false;
                    self.notify(ctx, t!("actions.log_save_success"));
                } else {
                    self.notify(ctx, t!("actions.log_save_failed", e = "SAF Write Error"));
                }
            }
            Err(e) => self.notify(ctx, format!("Encryption error: {}", e)),
        }
    }

    fn action_close_file(&mut self) {
        self.document = sen_core::history::DocumentWithHistory::default();
        self.is_modified = false;
        self.current_file_name = None;
        self.current_file_uri = None;
        self.loaded_history_index = None;
    }

    fn refresh_file_tree(&mut self) {
        if let Some(uri) = self.selected_directory_uri.clone() {
            let mut entries = self.call_java_list_directory(&uri);
            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| {
                if a.is_dir && !b.is_dir {
                    std::cmp::Ordering::Less
                } else if !a.is_dir && b.is_dir {
                    std::cmp::Ordering::Greater
                } else {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            });
            self.file_tree_entries = entries;
        }
    }
}

// ──────────────────────────────────────────────
// UI Rendering
// ──────────────────────────────────────────────

impl eframe::App for SenAndroidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_injected_text_events(ctx);

        // Handle background events
        self.handle_background_events(ctx);

        // Update search matches
        if !self.search_query.is_empty() {
            let text = self.document.current_content.to_lowercase();
            let query = self.search_query.to_lowercase();
            self.search_matches = text.match_indices(&query).map(|(i, _)| i).collect();
        } else {
            self.search_matches.clear();
        }

        // --- Biometric Startup Logic ---
        if self.biometric_enabled && self.is_locked {
            // Automatically show prompt on first frame if enabled
            if ctx.input(|i| i.time) < 0.2 {
                self.call_java_show_biometric_prompt();
            }
        }
        if self.settings.auto_save_enabled && self.is_modified && self.current_file_uri.is_some() {
            let now = ctx.input(|i| i.time);
            if self.last_autosave_time == 0.0 {
                self.last_autosave_time = now;
            }

            // Auto-save every 30 seconds if modified
            if now - self.last_autosave_time > 30.0 {
                self.action_save_file(ctx);
                self.last_autosave_time = now;
                self.notify(ctx, t!("settings.auto_save"));
            }
        } else {
            self.last_autosave_time = 0.0;
        }

        // Force continuous repaint on Android to prevent idle freezing
        ctx.request_repaint();

        // Mobile-friendly styling
        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(12.0, 12.0);
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.interact_size = egui::vec2(44.0, 44.0);
        ctx.set_style(style);

        // UI Code continues below...
        self.render_ui(ctx);
    }
}

impl SenAndroidApp {
    fn apply_injected_text_events(&self, ctx: &egui::Context) {
        let queued = {
            if let Ok(mut queue) = injected_input_queue().lock() {
                if queue.is_empty() {
                    return;
                }
                std::mem::take(&mut *queue)
            } else {
                return;
            }
        };

        ctx.input_mut(|input| {
            for event in queued {
                match event {
                    InjectedInputEvent::Text(text) => input.events.push(egui::Event::Text(text)),
                    InjectedInputEvent::Key(key) => {
                        input.events.push(egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                        input.events.push(egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: false,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        });
                    }
                }
            }
        });
    }

    fn handle_background_events(&mut self, ctx: &egui::Context) {
        if let Ok(mut channel) = android_fs::get_file_channel().lock() {
            // Handle Keyfile selection
            if let Some((uri, data)) = channel.pending_keyfile.take() {
                self.keyfile_content = data;
                self.keyfile_uri = Some(uri);
                self.notify(ctx, t!("app.keyfile_loaded"));
            }

            // Handle Open
            if let Some((data, name)) = channel.pending_open.take() {
                // Attempt to decrypt with current keyfile
                if !self.keyfile_content.is_empty() {
                    match sen_core::crypto::decrypt_content_bytes(&data, &self.keyfile_content) {
                        Ok(decrypted) => {
                            let content_str = String::from_utf8_lossy(&decrypted).to_string();
                            self.document =
                                sen_core::history::DocumentWithHistory::from_file_content(
                                    &content_str,
                                );
                            self.current_file_name = Some(name.clone());
                            self.loaded_history_index = None;
                            self.is_modified = false;
                            self.notify(
                                ctx,
                                t!(
                                    "actions.status_opened_file_history",
                                    file = name,
                                    count = self.document.history.len()
                                ),
                            );
                            return;
                        }
                        Err(e) => {
                            self.notify(ctx, format!("{}: {}", t!("actions.log_dec_failed"), e));
                        }
                    }
                }

                // Fallback to plain text if not encrypted or decryption failed
                let content_str = String::from_utf8_lossy(&data).to_string();
                self.document =
                    sen_core::history::DocumentWithHistory::from_file_content(&content_str);
                self.current_file_name = Some(name);
                self.loaded_history_index = None;
                self.is_modified = false;
                self.notify(ctx, t!("actions.status_opened_plaintext"));
            }

            // Handle Biometric result
            if let Some(success) = channel.pending_biometric_result.take() {
                if success {
                    self.is_locked = false;
                    self.notify(ctx, t!("actions.status_unlocked"));
                } else {
                    self.notify(ctx, t!("actions.log_dec_failed"));
                }
            }

            // Handle Save URI picked
            if let Some(uri) = channel.pending_save_uri.take() {
                self.current_file_uri = Some(uri.clone());
                self.perform_save_to_uri(&uri, ctx);
            }

            // Handle Directory selection
            if let Some(uri) = channel.pending_directory_uri.take() {
                self.selected_directory_uri = Some(uri);
                self.refresh_file_tree();
                self.notify(ctx, t!("app.workspace_updated"));
            }

            // Handle App Paused (Background)
            // Only lock if we didn't just launch a SAF picker ourselves
            if channel.pending_pause {
                channel.pending_pause = false;
                if self.picker_active {
                    // A picker we launched caused onPause — don't lock
                    self.picker_active = false;
                } else if self.biometric_enabled {
                    self.is_locked = true;
                    // Reset menu states when locking
                    self.show_menu = false;
                    self.show_settings = false;
                    self.show_file_tree = false;
                    self.show_history = false;
                }
            }
        }
    }

    fn render_ui(&mut self, ctx: &egui::Context) {
        // --- App Locked Overlay ---
        if self.biometric_enabled && self.is_locked {
            egui::CentralPanel::default()
                .frame(egui::Frame::default().fill(self.current_theme.colors.background_color()))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(ui.available_height() * 0.3);
                        if let Some(icons) = &self.icons {
                            ui.add(
                                egui::Image::new(&icons.key)
                                    .fit_to_exact_size(egui::vec2(80.0, 80.0))
                                    .tint(self.current_theme.colors.icon_color()),
                            );
                        }
                        ui.add_space(20.0);
                        ui.heading(egui::RichText::new(t!("app.locked_title")).size(24.0));
                        ui.label(t!("app.locked_msg"));
                        ui.add_space(40.0);

                        let btn = ui.add_sized(
                            [200.0, 50.0],
                            egui::Button::new(t!("app.unlock_btn"))
                                .fill(self.current_theme.colors.button_bg_color()),
                        );
                        if btn.clicked() {
                            self.call_java_show_biometric_prompt();
                        }
                    });
                });
            return; // Lock screen covers everything
        }

        let mut clicked_hamburger = false;
        let mut clicked_file_tree = false;
        let mut clicked_save = false;
        let mut clicked_close = false;

        // --- Top Toolbar ---
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Apply top padding to clear the status bar on Android.
            #[cfg(target_os = "android")]
            ui.add_space(30.0);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.set_height(48.0);
                if let Some(icons) = &self.icons {
                    let btn_size = egui::vec2(32.0, 32.0);
                    let btn_frame = egui::vec2(44.0, 44.0);

                    ui.allocate_ui(btn_frame, |ui| {
                        if ui
                            .add(
                                egui::Button::image(
                                    egui::Image::new(&icons.hamburger).fit_to_exact_size(btn_size),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            clicked_hamburger = true;
                        }
                    });

                    let title = match &self.current_file_name {
                        Some(name) => {
                            format!("{}{}", if self.is_modified { "*" } else { "" }, name)
                        }
                        None => t!("app.untitled").to_string(),
                    };
                    ui.label(egui::RichText::new(title).strong());

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.allocate_ui(btn_frame, |ui| {
                            if ui
                                .add(
                                    egui::Button::image(
                                        egui::Image::new(&icons.close).fit_to_exact_size(btn_size),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                clicked_close = true;
                            }
                        });
                        ui.allocate_ui(btn_frame, |ui| {
                            if ui
                                .add(
                                    egui::Button::image(
                                        egui::Image::new(&icons.save).fit_to_exact_size(btn_size),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                clicked_save = true;
                            }
                        });
                        ui.allocate_ui(btn_frame, |ui| {
                            if ui
                                .add(
                                    egui::Button::image(
                                        egui::Image::new(&icons.file_tree)
                                            .fit_to_exact_size(btn_size),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                clicked_file_tree = true;
                            }
                        });
                    });
                }
            });

            // --- Top Status Info ---
            ui.horizontal(|ui| {
                if self.is_modified {
                    ui.label(egui::RichText::new("*").weak());
                }
                if let Some(uri) = &self.current_file_uri {
                    let short_uri = if uri.len() > 20 {
                        format!("...{}", &uri[uri.len() - 20..])
                    } else {
                        uri.clone()
                    };
                    ui.label(
                        egui::RichText::new(format!("{}: {}", t!("settings.current"), short_uri))
                            .small()
                            .weak(),
                    );
                } else {
                    ui.label(
                        egui::RichText::new(t!("app.unsaved_document"))
                            .small()
                            .weak(),
                    );
                }
            });
            ui.add_space(4.0);
        });

        // --- Search Bar ---
        if self.show_search {
            egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label("🔍");
                    let text_edit = egui::TextEdit::singleline(&mut self.search_query)
                        .id_source("search_input")
                        .hint_text(t!("search.hint"))
                        .desired_width(f32::INFINITY);
                    let res = ui.add(text_edit);

                    if ui.button(t!("dialog.btn_close")).clicked() {
                        self.show_search = false;
                        self.search_query.clear();
                    }
                });
                ui.add_space(8.0);
            });
        }

        if clicked_hamburger {
            self.show_menu = !self.show_menu;
        }
        if clicked_file_tree {
            self.show_file_tree = !self.show_file_tree;
        }
        if clicked_save {
            self.action_save_file(ctx);
        }
        if clicked_close {
            self.action_close_file();
        }

        // --- Full-Screen Panels Logic ---
        let overlay_active =
            self.show_menu || self.show_settings || self.show_file_tree || self.show_history;

        if overlay_active {
            egui::CentralPanel::default().show(ctx, |ui| {
                if self.show_menu {
                    self.render_hamburger_menu(ctx, ui);
                } else if self.show_settings {
                    self.ui_settings(ctx, ui);
                } else if self.show_file_tree {
                    self.render_file_tree(ui);
                } else if self.show_history {
                    self.render_history_panel(ctx, ui);
                }
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    const INFINITY: f32 = f32::INFINITY;
                    let font_size = self.settings.editor_font_size;
                    let word_wrap = self.settings.word_wrap;
                    let text_color = ui.visuals().widgets.active.text_color();
                    let highlight_color = self.current_theme.colors.highlight_color();
                    let link_color = self.current_theme.colors.hyperlink_color();
                    let comment_color = self.current_theme.colors.comment_color();
                    let comment_prefix = self.settings.comment_prefix.clone();
                    let search_query = self.search_query.clone();
                    let search_matches = self.search_matches.clone();

                    let layouter = move |ui: &egui::Ui,
                                         text: &dyn egui::TextBuffer,
                                         wrap_width: f32| {
                        let text_str = text.as_str();
                        let mut layout_job = egui::text::LayoutJob::default();
                        let font_id = egui::FontId::monospace(font_size);

                        if word_wrap {
                            layout_job.wrap.max_width = wrap_width;
                        } else {
                            layout_job.wrap.max_width = f32::INFINITY;
                        }

                        if text_str.is_empty() {
                            layout_job.append(
                                "",
                                0.0,
                                egui::TextFormat {
                                    font_id: font_id.clone(),
                                    color: text_color,
                                    valign: egui::Align::Center,
                                    ..Default::default()
                                },
                            );
                            return ui.fonts_mut(|f| f.layout_job(layout_job));
                        }

                        // Capture state for segments
                        let search_query_len = search_query.len();
                        let search_active = !search_query.is_empty() && !search_matches.is_empty();
                        let link_matches = find_urls(text_str);
                        let highlight_color = highlight_color;
                        let link_color = link_color;
                        let comment_color = comment_color;
                        let comment_prefix = comment_prefix.clone();

                        let mut byte_offset = 0;
                        let lines: Vec<&str> = text_str.split('\n').collect();
                        let line_count = lines.len();

                        for (i, line) in lines.into_iter().enumerate() {
                            let is_last = i == line_count - 1;
                            let line_start_byte = byte_offset;
                            let line_end_byte = byte_offset + line.len();

                            let trimmed = line.trim_start();
                            let is_comment =
                                !comment_prefix.is_empty() && trimmed.starts_with(&comment_prefix);
                            let content_color = if is_comment {
                                comment_color
                            } else {
                                text_color
                            };

                            if !line.is_empty() {
                                let mut line_links = Vec::new();
                                for link in &link_matches {
                                    if link.end <= line_start_byte || link.start >= line_end_byte {
                                        continue;
                                    }
                                    let l_start = if link.start > line_start_byte {
                                        link.start - line_start_byte
                                    } else {
                                        0
                                    };
                                    let l_end = if link.end < line_end_byte {
                                        link.end - line_start_byte
                                    } else {
                                        line.len()
                                    };
                                    line_links.push(l_start..l_end);
                                }

                                let mut segments = Vec::new();
                                let mut pos = 0;
                                for link_range in line_links {
                                    if link_range.start > pos {
                                        segments.push((pos..link_range.start, false));
                                    }
                                    segments.push((link_range.start..link_range.end, true));
                                    pos = link_range.end;
                                }
                                if pos < line.len() {
                                    segments.push((pos..line.len(), false));
                                }

                                for (seg_range, is_link) in segments {
                                    let seg_start_global = line_start_byte + seg_range.start;
                                    let seg_end_global = line_start_byte + seg_range.end;
                                    let base_color = if is_link && !is_comment {
                                        link_color
                                    } else {
                                        content_color
                                    };
                                    let underline = if is_link && !is_comment {
                                        egui::Stroke::new(1.0, link_color)
                                    } else {
                                        egui::Stroke::NONE
                                    };

                                    if search_active {
                                        let mut s_pos = seg_range.start;
                                        for &m_start in &search_matches {
                                            let m_end = m_start + search_query_len;
                                            if m_end <= seg_start_global
                                                || m_start >= seg_end_global
                                            {
                                                continue;
                                            }
                                            let l_s_start =
                                                s_pos.max(if m_start > line_start_byte {
                                                    m_start - line_start_byte
                                                } else {
                                                    0
                                                });
                                            let l_s_end =
                                                seg_range.end.min(if m_end > line_start_byte {
                                                    m_end - line_start_byte
                                                } else {
                                                    0
                                                });

                                            if l_s_start > s_pos {
                                                layout_job.append(
                                                    &line[s_pos..l_s_start],
                                                    0.0,
                                                    egui::TextFormat {
                                                        font_id: font_id.clone(),
                                                        color: base_color,
                                                        valign: egui::Align::Center,
                                                        underline,
                                                        ..Default::default()
                                                    },
                                                );
                                            }
                                            if l_s_end > l_s_start {
                                                layout_job.append(
                                                    &line[l_s_start..l_s_end],
                                                    0.0,
                                                    egui::TextFormat {
                                                        font_id: font_id.clone(),
                                                        color: base_color,
                                                        valign: egui::Align::Center,
                                                        background: highlight_color,
                                                        underline,
                                                        ..Default::default()
                                                    },
                                                );
                                            }
                                            s_pos = l_s_end;
                                        }
                                        if s_pos < seg_range.end {
                                            layout_job.append(
                                                &line[s_pos..seg_range.end],
                                                0.0,
                                                egui::TextFormat {
                                                    font_id: font_id.clone(),
                                                    color: base_color,
                                                    valign: egui::Align::Center,
                                                    underline,
                                                    ..Default::default()
                                                },
                                            );
                                        }
                                    } else {
                                        layout_job.append(
                                            &line[seg_range],
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color: base_color,
                                                valign: egui::Align::Center,
                                                underline,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                }
                            }

                            if !is_last {
                                layout_job.append(
                                    "\n",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        valign: egui::Align::Center,
                                        ..Default::default()
                                    },
                                );
                                byte_offset += line.len() + 1;
                            } else {
                                byte_offset += line.len();
                            }
                        }
                        ui.fonts_mut(|f| f.layout_job(layout_job))
                    };

                    let mut layouter = layouter; // Still need mut for &mut borrow below
                    let text_edit = egui::TextEdit::multiline(&mut self.document.current_content)
                        .id_source("main_editor")
                        .frame(false)
                        .desired_width(INFINITY)
                        .desired_rows(20)
                        .hint_text(t!("app.editor_hint"))
                        .vertical_align(egui::Align::TOP)
                        .font(egui::FontId::monospace(font_size))
                        .layouter(&mut layouter);

                    let response = ui.add(text_edit);

                    if response.changed() {
                        self.is_modified = true;
                    }
                });
            });
        }

        // --- Notification Overlay ---
        if let Some((msg, expire)) = &self.notification {
            if ctx.input(|i| i.time) < *expire {
                egui::Window::new("")
                    .title_bar(false)
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -100.0))
                    .frame(
                        egui::Frame::window(&ctx.style())
                            .fill(egui::Color32::from_black_alpha(200)),
                    )
                    .show(ctx, |ui| {
                        ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE));
                    });
            } else {
                self.notification = None;
            }
        }
    }

    fn render_hamburger_menu(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        #[derive(Default)]
        struct MenuAction {
            new_file: bool,
            open_file: bool,
            save_file: bool,
            close_file: bool,
            select_keyfile: bool,
            toggle_settings: bool,
            toggle_history: bool,
            toggle_search: bool,
            close_menu: bool,
        }
        let mut action = MenuAction::default();

        ui.add_space(8.0);
        if Self::render_panel_header(self.icons.as_ref(), ui, "SEN Mobile") {
            self.show_menu = false;
        }

        if let Some(ref icons) = self.icons {
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(t!("toolbar.file"))
                    .strong()
                    .color(self.current_theme.colors.icon_color()),
            );
            let btn_height = 44.0;
            let icon_size = egui::vec2(24.0, 24.0);

            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.new_doc).fit_to_exact_size(icon_size),
                        t!("actions.log_new"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.new_file = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.open).fit_to_exact_size(icon_size),
                        t!("toolbar.open"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.open_file = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.save).fit_to_exact_size(icon_size),
                        t!("toolbar.save"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.save_file = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.close).fit_to_exact_size(icon_size),
                        t!("toolbar.close"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.close_file = true;
                action.close_menu = true;
            }
            ui.separator();

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(t!("settings.security"))
                    .strong()
                    .color(self.current_theme.colors.icon_color()),
            );
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.key).fit_to_exact_size(icon_size),
                        t!("app.select_keyfile"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.select_keyfile = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.folder_filled).fit_to_exact_size(icon_size),
                        t!("settings.open_dir"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                self.call_java_open_directory();
                action.close_menu = true;
            }

            if let Some(uri) = &self.keyfile_uri {
                let short_uri = if uri.len() > 20 {
                    format!("...{}", &uri[uri.len() - 20..])
                } else {
                    uri.clone()
                };
                ui.label(
                    egui::RichText::new(format!("{}: {}", t!("settings.current"), short_uri))
                        .small()
                        .weak(),
                );
            }

            ui.separator();
            ui.add_space(10.0);

            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.settings).fit_to_exact_size(icon_size),
                        t!("settings.title"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.toggle_settings = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.history).fit_to_exact_size(icon_size),
                        t!("history.title"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.toggle_history = true;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.file_tree).fit_to_exact_size(icon_size),
                        t!("settings.workspace"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.toggle_search = false; // dummy or just handle manually
                self.show_file_tree = !self.show_file_tree;
                action.close_menu = true;
            }
            if ui
                .add_sized(
                    [ui.available_width(), btn_height],
                    egui::Button::image_and_text(
                        egui::Image::new(&icons.search).fit_to_exact_size(icon_size),
                        t!("search.find"),
                    )
                    .frame(false),
                )
                .clicked()
            {
                action.toggle_search = true;
                action.close_menu = true;
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.label(t!("settings.stealth_mode"));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.stealth_mode, "");
                });
            });

            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("SEN Mobile v0.1.0").small().weak());
            });
        }
        if action.new_file {
            self.action_new_file();
        }
        if action.open_file {
            self.action_open_file();
        }
        if action.save_file {
            self.action_save_file(ctx);
        }
        if action.close_file {
            self.action_close_file();
        }
        if action.select_keyfile {
            self.action_select_keyfile();
        }
        if action.toggle_settings {
            self.show_settings = !self.show_settings;
        }
        if action.toggle_history {
            self.show_history = !self.show_history;
        }
        if action.toggle_search {
            self.show_search = !self.show_search;
        }
        if action.close_menu {
            self.show_menu = false;
        }
    }

    fn ui_settings(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let mut save_required = false;

        ui.add_space(8.0);
        if Self::render_panel_header(self.icons.as_ref(), ui, &t!("settings.title")) {
            self.show_settings = false;
        }

        ui.add_space(10.0);

        // --- Editor Section ---
        ui.label(
            egui::RichText::new(t!("settings.editor"))
                .strong()
                .size(18.0),
        );
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}px",
                t!("settings.editor_font_size"),
                self.settings.editor_font_size as i32
            ));
            ui.add_space(10.0);
            if ui.button("+").clicked() {
                self.settings.editor_font_size += 1.0;
                save_required = true;
            }
            if ui.button("-").clicked() && self.settings.editor_font_size > 8.0 {
                self.settings.editor_font_size -= 1.0;
                save_required = true;
            }
        });

        ui.add_space(5.0);
        ui.horizontal(|ui| {
            if ui
                .checkbox(&mut self.settings.word_wrap, t!("settings.word_wrap"))
                .changed()
            {
                save_required = true;
            }
        });
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // --- Appearance Section ---
        ui.label(
            egui::RichText::new(t!("settings.appearance"))
                .strong()
                .size(18.0),
        );
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.label("Theme:");
            let current_theme_name = self.settings.theme_name.clone();
            sen_core::ui::Select::new(&current_theme_name)
                .with_width_hint(ui, "Avaray Dark Mode")
                .show_ui(ui, |ui| {
                    for theme in self.available_themes.clone() {
                        if ui
                            .selectable_value(
                                &mut self.settings.theme_name,
                                theme.name.clone(),
                                &theme.name,
                            )
                            .clicked()
                        {
                            self.current_theme = theme.clone();
                            self.current_theme.apply(ctx);
                            save_required = true;
                            self.notify(
                                ctx,
                                format!("{} -> {}", t!("settings.appearance"), theme.name),
                            );
                        }
                    }
                });
        });

        ui.add_space(5.0);
        if ui
            .checkbox(
                &mut self.settings.auto_save_enabled,
                t!("settings.auto_save"),
            )
            .changed()
        {
            save_required = true;
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // --- Security Section ---
        ui.label(
            egui::RichText::new(t!("settings.security"))
                .strong()
                .size(18.0),
        );
        ui.add_space(5.0);
        if ui
            .checkbox(
                &mut self.settings.screen_capture_protection,
                t!("settings.screen_capture"),
            )
            .changed()
        {
            save_required = true;
            self.call_java_set_screen_protection(self.settings.screen_capture_protection);
        }

        if ui
            .checkbox(
                &mut self.settings.biometric_unlock,
                t!("settings.biometric_unlock"),
            )
            .changed()
        {
            save_required = true;
            self.biometric_enabled = self.settings.biometric_unlock;
        }

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // --- Localization Section ---
        ui.label(
            egui::RichText::new(t!("settings.language"))
                .strong()
                .size(18.0),
        );
        ui.add_space(5.0);

        let languages = [
            ("en", "English"),
            ("pl", "Polski"),
            ("de", "Deutsch"),
            ("cz", "Čeština"),
            ("es", "Español"),
            ("fr", "Français"),
            ("uk", "Українська"),
            ("zh-CN", "简体中文"),
            ("ja", "日本語"),
            ("ru", "Русский"),
            ("it", "Italiano"),
        ];
        let current_lang_name = languages
            .iter()
            .find(|(c, _)| *c == self.settings.language)
            .map(|(_, n)| *n)
            .unwrap_or("English");
        sen_core::ui::Select::new(current_lang_name)
            .with_width_hint(ui, "Nederlands")
            .show_ui(ui, |ui| {
                for (code, name) in languages {
                    if ui
                        .selectable_value(&mut self.settings.language, code.to_string(), name)
                        .clicked()
                    {
                        settings_set_language(code);
                        save_required = true;
                        self.notify(ctx, format!("{} -> {}", t!("settings.language"), name));
                    }
                }
            });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // --- About Section ---
        ui.label(
            egui::RichText::new(t!("dialog.about_title"))
                .strong()
                .size(18.0),
        );
        ui.add_space(5.0);
        ui.label("Secure Encrypted Notepad (SEN)");
        ui.label(egui::RichText::new("Version: 0.1.0-android").small().weak());
        ui.label(
            egui::RichText::new("Open source project by Avaray")
                .small()
                .italics(),
        );

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        if ui.button(t!("dialog.btn_close")).clicked() {
            self.show_settings = false;
        }

        if save_required {
            let _ = self.settings.save(self.android_app.internal_data_path());
        }
    }

    fn render_history_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.add_space(8.0);
        if Self::render_panel_header(self.icons.as_ref(), ui, &t!("history.title")) {
            self.show_history = false;
        }

        // To avoid borrow checker issues (borrowing self as mutable in notify while document is borrowed),
        // we collect the indices and data we need first.
        let history_entries: Vec<(usize, String, String, Option<String>)> = self
            .document
            .get_visible_history()
            .iter()
            .rev()
            .map(|(idx, entry)| {
                (
                    *idx,
                    entry.display_timestamp(),
                    entry.display_size(),
                    entry.comment.clone(),
                )
            })
            .collect();

        if history_entries.is_empty() {
            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                ui.label(t!("history.no_history"));
                ui.label(egui::RichText::new(t!("history.help")).small().weak());
            });
        } else {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut to_load = None;
                let mut to_restore = None;

                for (idx, timestamp, size, comment) in history_entries {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(timestamp).strong());
                            ui.label(t!("history.size", size = size));
                            if let Some(comment_text) = comment {
                                ui.label(egui::RichText::new(comment_text).small().italics());
                            }

                            ui.horizontal(|ui| {
                                if ui.button(t!("history.revert_entry")).clicked() {
                                    to_load = Some(idx);
                                }
                                if ui.button(t!("history.revert_changes")).clicked() {
                                    to_restore = Some(idx);
                                }
                            });
                        });
                    });
                    ui.add_space(4.0);
                }

                if let Some(idx) = to_load {
                    self.document.load_version(idx);
                    self.loaded_history_index = Some(idx);
                    self.show_history = false;
                    self.notify(ctx, t!("actions.status_ver_loaded"));
                }
                if let Some(idx) = to_restore {
                    self.document.revert_to_version(idx);
                    self.loaded_history_index = None;
                    self.is_modified = true;
                    self.show_history = false;
                    self.notify(ctx, t!("actions.status_revert_success"));
                }
            });
        }
    }

    fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            if Self::render_panel_header(self.icons.as_ref(), ui, &t!("settings.workspace")) {
                self.show_file_tree = false;
            }

            ui.add_space(4.0);
            if ui.button("⟳").clicked() {
                self.refresh_file_tree();
            }
            ui.separator();

            if self.file_tree_entries.is_empty() {
                ui.add_space(20.0);
                ui.vertical_centered(|ui| {
                    ui.label(t!("settings.no_dir_opened"));
                    if ui.button(t!("settings.open_dir")).clicked() {
                        self.picker_active = true;
                        self.call_java_open_directory();
                    }
                });
            } else if let Some(ref icons) = self.icons {
                let folder_icon = icons.folder_filled.clone();
                let file_icon = icons.unknown_file.clone();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let entries = self.file_tree_entries.clone();
                    for entry in entries {
                        let icon = if entry.is_dir {
                            folder_icon.clone()
                        } else {
                            file_icon.clone()
                        };

                        let tint = self.current_theme.colors.icon_color();
                        let icon_size = ui.text_style_height(&egui::TextStyle::Button);

                        ui.horizontal(|ui| {
                            if ui
                                .add(egui::Button::image_and_text(
                                    egui::Image::new(&icon)
                                        .fit_to_exact_size(egui::vec2(icon_size, icon_size))
                                        .tint(tint),
                                    &entry.name,
                                ))
                                .clicked()
                            {
                                if !entry.is_dir {
                                    self.action_load_from_tree(&entry.uri, &entry.name, ui.ctx());
                                }
                            }
                        });
                    }
                });
            }
        });
    }

    fn action_load_from_tree(&mut self, uri: &str, name: &str, ctx: &egui::Context) {
        if let Some(data) = self.call_java_read_bytes(uri) {
            // Attempt decryption
            if !self.keyfile_content.is_empty() {
                // Try standard decryption
                let standard_result =
                    sen_core::crypto::decrypt_content_bytes(&data, &self.keyfile_content);

                // Try stealth if standard fails or if enabled
                let result = standard_result.or_else(|_| {
                    sen_core::crypto::decrypt_stealth_bytes(&data, &self.keyfile_content)
                });

                if let Ok(decrypted) = result {
                    let content_str = String::from_utf8_lossy(&decrypted).to_string();
                    self.document =
                        sen_core::history::DocumentWithHistory::from_file_content(&content_str);
                    self.current_file_name = Some(name.to_string());
                    self.current_file_uri = Some(uri.to_string());
                    self.loaded_history_index = None;
                    self.is_modified = false;
                    self.notify(
                        ctx,
                        t!(
                            "actions.status_opened_file_history",
                            file = name,
                            count = self.document.history.len()
                        ),
                    );
                    return;
                }
            }
            // Plain text fallback
            let content_str = String::from_utf8_lossy(&data).to_string();
            self.document = sen_core::history::DocumentWithHistory::from_file_content(&content_str);
            self.current_file_name = Some(name.to_string());
            self.current_file_uri = Some(uri.to_string());
            self.loaded_history_index = None;
            self.is_modified = false;
            self.notify(ctx, t!("actions.status_opened_plaintext"));
        } else {
            self.notify(ctx, t!("actions.status_key_read_err", e = "SAF Error"));
        }
    }
}

fn settings_set_language(code: &str) {
    sen_translations::set_locale(code);
}
