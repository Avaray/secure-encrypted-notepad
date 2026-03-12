use orion::aead;
use rand::RngCore;
use serde::{Deserialize, Serialize};

use std::fs;
use std::path::PathBuf;

use keyring::Entry;

const CONFIG_MAGIC: &[u8; 4] = b"SENC";
const SERVICE_NAME: &str = "sen-notepad";
const USER_NAME: &str = "config-key";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ToolbarPosition {
    Top,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CursorShape {
    Bar,
    Block,
    Underscore,
}

impl Default for CursorShape {
    fn default() -> Self {
        Self::Bar
    }
}

impl Default for ToolbarPosition {
    fn default() -> Self {
        Self::Top
    }
}

/// User preferences - persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// UI font size (8-32 px)
    pub ui_font_size: f32,
    /// Editor font size (8-32 px)
    pub editor_font_size: f32,
    /// Editor line height multiplier (1.0 - 2.0)
    #[serde(default = "default_line_height")]
    pub line_height: f32,
    /// UI font family
    #[serde(default = "default_ui_font")]
    pub ui_font_family: String,
    /// Editor font family
    #[serde(default = "default_editor_font")]
    pub editor_font_family: String,
    /// Current theme name
    pub theme_name: String,
    /// Whether to use global keyfile automatically on startup
    pub use_global_keyfile: bool,
    /// Encrypted keyfile path (serialized to TOML as "<nonce>:<ciphertext>")
    #[serde(default)]
    pub keyfile_path_encrypted: Option<String>,
    /// Path to the global keyfile (memory-only, never serialized to disk)
    #[serde(skip)]
    pub global_keyfile_path: Option<PathBuf>,

    /// Encrypted file tree starting directory (serialized the same way as keyfile_path_encrypted)
    #[serde(default)]
    pub file_tree_dir_encrypted: Option<String>,
    /// File tree starting directory (memory-only, never serialized to disk)
    #[serde(skip)]
    pub file_tree_starting_dir: Option<PathBuf>,

    /// Encrypted auto-backup directory path
    #[serde(default)]
    pub auto_backup_dir_encrypted: Option<String>,
    /// Path to the auto-backup directory (memory-only, never serialized to disk)
    #[serde(skip)]
    pub auto_backup_dir: Option<PathBuf>,
    /// Auto-backup on save enabled
    #[serde(default)]
    pub auto_backup_enabled: bool,
    /// Screen capture protection (Windows only)
    #[serde(default)]
    pub screen_capture_protection: bool,
    /// Show line numbers in editor
    pub show_line_numbers: bool,
    /// Show file tree panel
    pub show_file_tree: bool,
    /// Show special symbols (spaces, tabs, returns)
    #[serde(default)]
    pub show_whitespace: bool,

    /// Editor settings
    pub tab_size: usize,
    pub use_spaces_for_tabs: bool,
    pub word_wrap: bool,

    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Auto-save debounce time in seconds (inactivity before saving)
    pub auto_save_debounce_secs: u64,
    /// Auto-save on focus loss
    #[serde(default = "default_true")]
    pub auto_save_on_focus_loss: bool,

    /// Show debug panel
    pub show_debug_panel: bool,
    /// Show settings panel
    #[serde(default)]
    pub show_settings_panel: bool,
    /// Show history panel
    #[serde(default)]
    pub show_history_panel: bool,
    /// Show theme editor panel
    #[serde(default)]
    pub show_theme_editor: bool,
    /// Show search panel
    #[serde(default)]
    pub show_search_panel: bool,
    /// Whether to restore all open panels from last session
    #[serde(default)]
    pub preserve_all_panels: bool,
    /// File tree panel width
    pub file_tree_width: f32,

    /// Theme editor panel width
    #[serde(default = "default_panel_width_270")]
    pub theme_editor_width: f32,

    /// Settings panel width
    #[serde(default = "default_panel_width_350")]
    pub settings_panel_width: f32,

    /// History panel width
    #[serde(default = "default_panel_width_250")]
    pub history_panel_width: f32,

    /// Debug panel width
    #[serde(default = "default_panel_width_250")]
    pub debug_panel_width: f32,

    /// Show subfolders in file tree
    pub show_subfolders: bool,
    /// Hide .sen extension in file tree
    #[serde(default)]
    pub hide_sen_extension: bool,
    /// Max history length
    pub max_history_length: usize,

    /// Editor line limit (0 means disabled)
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,

    /// Whether to show full keyfile paths
    #[serde(default)]
    pub show_keyfile_paths: bool,
    /// Whether to show full directory paths
    #[serde(default)]
    pub show_directory_paths: bool,
    /// Start window in maximized mode
    #[serde(default)]
    pub start_maximized: bool,
    /// Hide panel headers (Settings, History, etc.)
    #[serde(default)]
    pub hide_panel_headers: bool,

    /// Window dimensions and position
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default)]
    pub window_pos_x: f32,
    #[serde(default)]
    pub window_pos_y: f32,
    /// Toolbar icon size in pixels (e.g. 16 to 64)
    #[serde(default = "default_toolbar_icon_size")]
    pub toolbar_icon_size: f32,
    /// Toolbar position
    #[serde(default)]
    pub toolbar_position: ToolbarPosition,

    /// Text cursor shape
    #[serde(default)]
    pub cursor_shape: CursorShape,
    /// Whether the cursor blinks
    #[serde(default = "default_true")]
    pub cursor_blink: bool,

    /// Volatile flag to indicate if this is the first run (no config file existed)
    #[serde(skip)]
    pub is_first_run: bool,
}

fn default_window_width() -> f32 {
    1200.0
}

fn default_window_height() -> f32 {
    800.0
}

fn default_ui_font() -> String {
    "Proportional (Default)".to_string()
}

fn default_editor_font() -> String {
    "Monospace (Default)".to_string()
}

fn default_toolbar_icon_size() -> f32 {
    24.0
}

fn default_panel_width_250() -> f32 {
    250.0
}

fn default_panel_width_270() -> f32 {
    270.0
}

fn default_panel_width_350() -> f32 {
    350.0
}

fn default_line_height() -> f32 {
    1.2
}

fn default_max_lines() -> usize {
    1000
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_font_size: 16.0,
            editor_font_size: 14.0,
            line_height: 1.2,
            ui_font_family: "Proportional (Default)".to_string(),
            editor_font_family: "Monospace (Default)".to_string(),
            theme_name: "Dark".to_string(),
            use_global_keyfile: false,
            keyfile_path_encrypted: None,
            global_keyfile_path: None,
            file_tree_dir_encrypted: None,
            file_tree_starting_dir: None,
            auto_backup_dir_encrypted: None,
            auto_backup_dir: None,
            auto_backup_enabled: false,
            screen_capture_protection: false,
            show_line_numbers: true,
            show_file_tree: true,
            show_whitespace: false,
            tab_size: 4,
            use_spaces_for_tabs: true,
            word_wrap: false,
            auto_save_enabled: true,
            auto_save_debounce_secs: 15,
            auto_save_on_focus_loss: true,

            show_debug_panel: false,
            show_settings_panel: false,
            show_history_panel: false,
            show_theme_editor: false,
            show_search_panel: false,
            preserve_all_panels: false,
            file_tree_width: 200.0,
            theme_editor_width: default_panel_width_270(),
            settings_panel_width: default_panel_width_350(),
            history_panel_width: default_panel_width_250(),
            debug_panel_width: default_panel_width_250(),
            show_subfolders: true,
            hide_sen_extension: false,
            hide_panel_headers: false,
            max_history_length: 100,
            max_lines: default_max_lines(),

            show_keyfile_paths: false,
            show_directory_paths: false,
            start_maximized: false,
            window_width: 1200.0,
            window_height: 800.0,
            window_pos_x: -1.0, // -1 means "let OS decide" or "center"
            window_pos_y: -1.0,
            toolbar_icon_size: 24.0, // Default 24px icon
            toolbar_position: ToolbarPosition::Top,
            cursor_shape: crate::settings::CursorShape::Bar,
            cursor_blink: true,
            is_first_run: false,
        }
    }
}

fn default_true() -> bool {
    true
}

impl Settings {
    /// Load settings from file
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("sen").join("config.toml");
            if config_path.exists() {
                match fs::read(&config_path) {
                    Ok(content_bytes) => {
                        // Check for encryption magic header
                        if content_bytes.len() > 4 && &content_bytes[0..4] == CONFIG_MAGIC {
                            // Encrypted content
                            match Self::get_or_create_config_key() {
                                Ok(key_bytes) => match aead::SecretKey::from_slice(&key_bytes) {
                                    Ok(secret_key) => {
                                        let encrypted_data = &content_bytes[4..];
                                        match aead::open(&secret_key, encrypted_data) {
                                                Ok(plaintext) => {
                                                    match toml::from_str::<Settings>(&String::from_utf8_lossy(&plaintext)) {
                                                        Ok(mut settings) => {
                                                            Self::decrypt_keyfile_path_field(&mut settings);
                                                            Self::decrypt_file_tree_dir_field(&mut settings);
                                                            Self::decrypt_auto_backup_dir_field(&mut settings);
                                                            Self::migrate_legacy_fonts(&mut settings);
                                                            sen_debug!("Settings loaded OK: use_global_keyfile={}, global_keyfile={:?}, start_maximized={}, theme={}",
                                                                settings.use_global_keyfile, settings.global_keyfile_path, settings.start_maximized, settings.theme_name);
                                                            return settings;
                                                        }
                                                        Err(e) => sen_debug!("Config TOML parse error: {}", e),
                                                    }
                                                }
                                                Err(e) => sen_debug!("Config decryption failed (key may have changed): {}", e),
                                            }
                                    }
                                    Err(e) => sen_debug!("Invalid config key format: {}", e),
                                },
                                Err(e) => {
                                    sen_debug!("Failed to get config key from keyring: {}", e)
                                }
                            }
                        } else {
                            // Plaintext fallback (legacy)
                            if let Ok(content_str) = String::from_utf8(content_bytes) {
                                match toml::from_str::<Settings>(&content_str) {
                                    Ok(mut settings) => {
                                        Self::decrypt_keyfile_path_field(&mut settings);
                                        Self::decrypt_file_tree_dir_field(&mut settings);
                                        Self::decrypt_auto_backup_dir_field(&mut settings);
                                        Self::migrate_legacy_fonts(&mut settings);
                                        sen_debug!("Settings loaded OK (plaintext): use_global_keyfile={}, global_keyfile={:?}, start_maximized={}, theme={}",
                                            settings.use_global_keyfile, settings.global_keyfile_path, settings.start_maximized, settings.theme_name);
                                        return settings;
                                    }
                                    Err(e) => {
                                        sen_debug!("Config TOML parse error (plaintext): {}", e)
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => sen_debug!("Failed to read config file: {}", e),
                }
            }
        }
        sen_debug!("Using default settings (possible first run)");
        let mut settings = Self::default();
        settings.is_first_run = true;
        settings
    }

    /// Migrates legacy font names without the "(Default)" suffix.
    fn migrate_legacy_fonts(settings: &mut Settings) {
        if settings.ui_font_family == "Proportional" {
            settings.ui_font_family = "Proportional (Default)".to_string();
        }
        if settings.editor_font_family == "Monospace" {
            settings.editor_font_family = "Monospace (Default)".to_string();
        }
    }

    /// Save settings to file (plaintext TOML).
    ///
    /// The global keyfile path is encrypted before serialization.
    /// The `#[serde(skip)]` on `global_keyfile_path` ensures only the
    /// encrypted form (`keyfile_path_encrypted`) hits disk.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("sen");
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("config.toml");

            // Clone to mutate the encrypted field before serialization
            let mut to_save = self.clone();
            to_save.encrypt_keyfile_path_field();
            to_save.encrypt_file_tree_dir_field();
            to_save.encrypt_auto_backup_dir_field();

            let toml_string = toml::to_string_pretty(&to_save)?;
            fs::write(&config_path, toml_string)?;
            sen_debug!("Settings saved OK: use_global_keyfile={}, keyfile_encrypted={}, start_maximized={}, theme={}",
                self.use_global_keyfile, to_save.keyfile_path_encrypted.is_some(), self.start_maximized, self.theme_name);
        }
        Ok(())
    }

    /// Get or create a random 32-byte encryption key stored in OS keyring
    fn get_or_create_config_key() -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let entry = Entry::new(SERVICE_NAME, USER_NAME)?;

        match entry.get_password() {
            Ok(password) => {
                // Password is hex encoded key
                if let Ok(key_bytes) = hex::decode(password) {
                    if key_bytes.len() == 32 {
                        return Ok(key_bytes);
                    }
                }
            }
            Err(_) => {
                // Key not found or error, create new
            }
        }

        // Generate new key
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        let key_hex = hex::encode(key);

        // Store in keyring
        entry.set_password(&key_hex)?;

        Ok(key.to_vec())
    }

    /// Validate font sizes
    pub fn validate_font_sizes(&mut self) {
        self.ui_font_size = self.ui_font_size.clamp(8.0, 32.0);
        self.editor_font_size = self.editor_font_size.clamp(8.0, 128.0);
    }

    /// Decrypt `keyfile_path_encrypted` into `global_keyfile_path`.
    /// On any error (keychain unavailable, decryption failure) the path
    /// is set to None and a warning is logged — never crashes.
    fn decrypt_keyfile_path_field(settings: &mut Self) {
        if let Some(ref encrypted) = settings.keyfile_path_encrypted {
            sen_debug!(
                "Attempting to decrypt keyfile path ({} chars encrypted)",
                encrypted.len()
            );
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => {
                    sen_debug!("Config crypto key retrieved from keychain OK");
                    match crate::config_crypto::decrypt_keyfile_path(&key, encrypted) {
                        Ok(path_str) => {
                            sen_debug!("Keyfile path decrypted OK: {:?}", path_str);
                            settings.global_keyfile_path = Some(PathBuf::from(path_str));
                        }
                        Err(e) => {
                            sen_debug!("Warning: failed to decrypt keyfile path: {}", e);
                            settings.global_keyfile_path = None;
                        }
                    }
                }
                Err(e) => {
                    sen_debug!(
                        "Warning: keychain unavailable, cannot decrypt keyfile path: {}",
                        e
                    );
                    settings.global_keyfile_path = None;
                }
            }
        } else {
            sen_debug!("No encrypted keyfile path in config");
        }
    }

    /// Encrypt `global_keyfile_path` into `keyfile_path_encrypted`.
    /// On keychain error the encrypted field is left UNCHANGED (preserves
    /// existing encrypted data) and a warning is logged.
    /// When `global_keyfile_path` is None, `keyfile_path_encrypted` is NOT
    /// cleared here — the caller must explicitly set it to None if the
    /// user intentionally removed the keyfile path.
    fn encrypt_keyfile_path_field(&mut self) {
        if let Some(ref path) = self.global_keyfile_path {
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => {
                    let path_str = path.to_string_lossy();
                    match crate::config_crypto::encrypt_keyfile_path(&key, &path_str) {
                        Ok(encrypted) => {
                            self.keyfile_path_encrypted = Some(encrypted);
                        }
                        Err(e) => {
                            // Don't erase existing encrypted data on failure
                            sen_debug!("Warning: failed to encrypt keyfile path: {}", e);
                        }
                    }
                }
                Err(e) => {
                    // Don't erase existing encrypted data on keychain failure
                    sen_debug!(
                        "Warning: keychain unavailable, keyfile path will not be saved: {}",
                        e
                    );
                }
            }
        }
        // When global_keyfile_path is None, leave keyfile_path_encrypted as-is.
        // This preserves the encrypted value if decryption failed on load,
        // so a subsequent save() doesn't destroy the data.
    }

    /// Decrypt `file_tree_dir_encrypted` into `file_tree_starting_dir`.
    /// Mirrors the keyfile path decryption pattern.
    fn decrypt_file_tree_dir_field(settings: &mut Self) {
        if let Some(ref encrypted) = settings.file_tree_dir_encrypted {
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => match crate::config_crypto::decrypt_keyfile_path(&key, encrypted) {
                    Ok(path_str) => {
                        sen_debug!("File tree dir decrypted OK: {:?}", path_str);
                        settings.file_tree_starting_dir = Some(PathBuf::from(path_str));
                    }
                    Err(e) => {
                        sen_debug!("Warning: failed to decrypt file tree dir: {}", e);
                        settings.file_tree_starting_dir = None;
                    }
                },
                Err(e) => {
                    sen_debug!("Warning: keychain unavailable for file tree dir: {}", e);
                    settings.file_tree_starting_dir = None;
                }
            }
        }
    }

    /// Encrypt `file_tree_starting_dir` into `file_tree_dir_encrypted`.
    /// Mirrors the keyfile path encryption pattern.
    fn encrypt_file_tree_dir_field(&mut self) {
        if let Some(ref path) = self.file_tree_starting_dir {
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => {
                    let path_str = path.to_string_lossy();
                    match crate::config_crypto::encrypt_keyfile_path(&key, &path_str) {
                        Ok(encrypted) => {
                            self.file_tree_dir_encrypted = Some(encrypted);
                        }
                        Err(e) => {
                            sen_debug!("Warning: failed to encrypt file tree dir: {}", e);
                        }
                    }
                }
                Err(e) => {
                    sen_debug!("Warning: keychain unavailable for file tree dir: {}", e);
                }
            }
        }
    }

    /// Decrypt `auto_backup_dir_encrypted` into `auto_backup_dir`.
    fn decrypt_auto_backup_dir_field(settings: &mut Self) {
        if let Some(ref encrypted) = settings.auto_backup_dir_encrypted {
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => match crate::config_crypto::decrypt_keyfile_path(&key, encrypted) {
                    Ok(path_str) => {
                        sen_debug!("Auto-backup dir decrypted OK: {:?}", path_str);
                        settings.auto_backup_dir = Some(PathBuf::from(path_str));
                    }
                    Err(e) => {
                        sen_debug!("Warning: failed to decrypt auto-backup dir: {}", e);
                        settings.auto_backup_dir = None;
                    }
                },
                Err(e) => {
                    sen_debug!("Warning: keychain unavailable for auto-backup dir: {}", e);
                    settings.auto_backup_dir = None;
                }
            }
        }
    }

    /// Encrypt `auto_backup_dir` into `auto_backup_dir_encrypted`.
    fn encrypt_auto_backup_dir_field(&mut self) {
        if let Some(ref path) = self.auto_backup_dir {
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => {
                    let path_str = path.to_string_lossy();
                    match crate::config_crypto::encrypt_keyfile_path(&key, &path_str) {
                        Ok(encrypted) => {
                            self.auto_backup_dir_encrypted = Some(encrypted);
                        }
                        Err(e) => {
                            sen_debug!("Warning: failed to encrypt auto-backup dir: {}", e);
                        }
                    }
                }
                Err(e) => {
                    sen_debug!("Warning: keychain unavailable for auto-backup dir: {}", e);
                }
            }
        }
    }

    pub fn get_config_dir() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|d| d.join("sen"))
    }
}
