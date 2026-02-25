use orion::aead;
use orion::kdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use zeroize::Zeroizing;
use keyring::Entry;

const CONFIG_MAGIC: &[u8; 4] = b"SEDC";
const SERVICE_NAME: &str = "sed-editor";
const USER_NAME: &str = "config-key";

/// User preferences - persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// UI font size (8-32 px)
    pub ui_font_size: f32,
    /// Editor font size (8-32 px)
    pub editor_font_size: f32,
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
    /// Whether to auto-create snapshot on save when content changes
    pub auto_snapshot_on_save: bool,
    /// Show line numbers in editor
    pub show_line_numbers: bool,
    /// Show file tree panel
    pub show_file_tree: bool,

    /// Editor settings
    pub tab_size: usize,
    pub use_spaces_for_tabs: bool,
    pub word_wrap: bool,
    
    /// Auto-save enabled
    pub auto_save_enabled: bool,
    /// Auto-save interval in seconds
    pub auto_save_interval_secs: u64,

    /// Clipboard Security
    pub clipboard_security_enabled: bool,
    pub clipboard_clear_timeout_secs: u64,

    /// Show debug panel
    pub show_debug_panel: bool,
    /// File tree panel width
    pub file_tree_width: f32,
    /// Show subfolders in file tree
    pub show_subfolders: bool,
    /// Max history length
    pub max_history_length: usize,
    /// Whether master password protection is enabled for sensitive settings
    #[serde(default)]
    pub master_password_enabled: bool,
    /// Start window in maximized mode
    #[serde(default)]
    pub start_maximized: bool,
    
    /// Window dimensions and position
    #[serde(default = "default_window_width")]
    pub window_width: f32,
    #[serde(default = "default_window_height")]
    pub window_height: f32,
    #[serde(default)]
    pub window_pos_x: f32,
    #[serde(default)]
    pub window_pos_y: f32,
}

fn default_window_width() -> f32 {
    1200.0
}

fn default_window_height() -> f32 {
    800.0
}

fn default_ui_font() -> String {
    "Proportional".to_string()
}

fn default_editor_font() -> String {
    "Monospace".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_font_size: 16.0,
            editor_font_size: 14.0,
            ui_font_family: "Proportional".to_string(),
            editor_font_family: "Monospace".to_string(),
            theme_name: "Dark".to_string(),
            use_global_keyfile: false,
            keyfile_path_encrypted: None,
            global_keyfile_path: None,
            auto_snapshot_on_save: true,
            show_line_numbers: true,
            show_file_tree: true,
            tab_size: 4,
            use_spaces_for_tabs: true,
            word_wrap: false,
            auto_save_enabled: true,
            auto_save_interval_secs: 60,
            clipboard_security_enabled: true,
            clipboard_clear_timeout_secs: 30,
            show_debug_panel: false,
            file_tree_width: 200.0,
            show_subfolders: true,
            max_history_length: 100,
            master_password_enabled: false,
            start_maximized: false,
            window_width: 1200.0,
            window_height: 800.0,
            window_pos_x: -1.0, // -1 means "let OS decide" or "center"
            window_pos_y: -1.0,
        }
    }
}


impl Settings {
    /// Load settings from file
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("sed").join("config.toml");
            if config_path.exists() {
                match fs::read(&config_path) {
                    Ok(content_bytes) => {
                        // Check for encryption magic header
                        if content_bytes.len() > 4 && &content_bytes[0..4] == CONFIG_MAGIC {
                            // Encrypted content
                            match Self::get_or_create_config_key() {
                                Ok(key_bytes) => {
                                    match aead::SecretKey::from_slice(&key_bytes) {
                                        Ok(secret_key) => {
                                            let encrypted_data = &content_bytes[4..];
                                            match aead::open(&secret_key, encrypted_data) {
                                                Ok(plaintext) => {
                                                    match toml::from_str::<Settings>(&String::from_utf8_lossy(&plaintext)) {
                                                        Ok(mut settings) => {
                                                            Self::decrypt_keyfile_path_field(&mut settings);
                                                            eprintln!("[SED] Settings loaded OK: use_global_keyfile={}, global_keyfile={:?}, start_maximized={}, theme={}",
                                                                settings.use_global_keyfile, settings.global_keyfile_path, settings.start_maximized, settings.theme_name);
                                                            return settings;
                                                        }
                                                        Err(e) => eprintln!("[SED] Config TOML parse error: {}", e),
                                                    }
                                                }
                                                Err(e) => eprintln!("[SED] Config decryption failed (key may have changed): {}", e),
                                            }
                                        }
                                        Err(e) => eprintln!("[SED] Invalid config key format: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("[SED] Failed to get config key from keyring: {}", e),
                            }
                        } else {
                            // Plaintext fallback (legacy)
                            if let Ok(content_str) = String::from_utf8(content_bytes) {
                                match toml::from_str::<Settings>(&content_str) {
                                    Ok(mut settings) => {
                                        Self::decrypt_keyfile_path_field(&mut settings);
                                        eprintln!("[SED] Settings loaded OK (plaintext): use_global_keyfile={}, global_keyfile={:?}, start_maximized={}, theme={}",
                                            settings.use_global_keyfile, settings.global_keyfile_path, settings.start_maximized, settings.theme_name);
                                        return settings;
                                    }
                                    Err(e) => eprintln!("[SED] Config TOML parse error (plaintext): {}", e),
                                }
                            }
                        }
                    }
                    Err(e) => eprintln!("[SED] Failed to read config file: {}", e),
                }
            }
        }
        eprintln!("[SED] Using default settings");
        Self::default()
    }

    /// Save settings to file (plaintext TOML).
    ///
    /// The global keyfile path is encrypted before serialization.
    /// The `#[serde(skip)]` on `global_keyfile_path` ensures only the
    /// encrypted form (`keyfile_path_encrypted`) hits disk.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("sed");
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("config.toml");

            // Clone to mutate the encrypted field before serialization
            let mut to_save = self.clone();
            to_save.encrypt_keyfile_path_field();

            let toml_string = toml::to_string_pretty(&to_save)?;
            fs::write(&config_path, toml_string)?;
            eprintln!("[SED] Settings saved OK: use_global_keyfile={}, keyfile_encrypted={}, start_maximized={}, theme={}",
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
        rand::thread_rng().fill_bytes(&mut key);
        let key_hex = hex::encode(key);
        
        // Store in keyring
        entry.set_password(&key_hex)?;
        
        Ok(key.to_vec())
    }

    /// Validate font sizes
    pub fn validate_font_sizes(&mut self) {
        self.ui_font_size = self.ui_font_size.clamp(8.0, 32.0);
        self.editor_font_size = self.editor_font_size.clamp(8.0, 32.0);
    }

    /// Decrypt `keyfile_path_encrypted` into `global_keyfile_path`.
    /// On any error (keychain unavailable, decryption failure) the path
    /// is set to None and a warning is logged — never crashes.
    fn decrypt_keyfile_path_field(settings: &mut Self) {
        if let Some(ref encrypted) = settings.keyfile_path_encrypted {
            eprintln!("[SED] Attempting to decrypt keyfile path ({} chars encrypted)", encrypted.len());
            match crate::config_crypto::get_or_create_config_key() {
                Ok(key) => {
                    eprintln!("[SED] Config crypto key retrieved from keychain OK");
                    match crate::config_crypto::decrypt_keyfile_path(&key, encrypted) {
                        Ok(path_str) => {
                            eprintln!("[SED] Keyfile path decrypted OK: {:?}", path_str);
                            settings.global_keyfile_path = Some(PathBuf::from(path_str));
                        }
                        Err(e) => {
                            eprintln!("[SED] Warning: failed to decrypt keyfile path: {}", e);
                            settings.global_keyfile_path = None;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[SED] Warning: keychain unavailable, cannot decrypt keyfile path: {}", e);
                    settings.global_keyfile_path = None;
                }
            }
        } else {
            eprintln!("[SED] No encrypted keyfile path in config");
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
                            eprintln!("[SED] Warning: failed to encrypt keyfile path: {}", e);
                        }
                    }
                }
                Err(e) => {
                    // Don't erase existing encrypted data on keychain failure
                    eprintln!("[SED] Warning: keychain unavailable, keyfile path will not be saved: {}", e);
                }
            }
        }
        // When global_keyfile_path is None, leave keyfile_path_encrypted as-is.
        // This preserves the encrypted value if decryption failed on load,
        // so a subsequent save() doesn't destroy the data.
    }
}

// =============================================================================
// SENSITIVE SETTINGS - stored encrypted or in memory only
// =============================================================================

/// Sensitive settings that should never be stored in plaintext.
/// When master_password is enabled, these are encrypted and persisted.
/// When disabled, these exist only in memory during the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitiveSettings {
    /// Path to global default keyfile
    pub global_keyfile_path: Option<PathBuf>,
    /// Last opened directory for file tree
    pub last_directory: Option<PathBuf>,
}

impl Default for SensitiveSettings {
    fn default() -> Self {
        Self {
            global_keyfile_path: None,
            last_directory: None,
        }
    }
}

/// Magic number for encrypted settings file
const SETTINGS_MAGIC: &[u8; 4] = b"SEDS";
const SETTINGS_SALT_SIZE: usize = 32;

#[allow(dead_code)]
impl SensitiveSettings {
    /// Derive encryption key from master password
    fn derive_key(password: &str, salt: &[u8]) -> Result<aead::SecretKey, Box<dyn std::error::Error>> {
        let password_bytes = Zeroizing::new(password.as_bytes().to_vec());
        let kdf_password = kdf::Password::from_slice(&password_bytes)?;
        let kdf_salt = kdf::Salt::from_slice(salt)?;

        // Derive 32-byte key using Argon2id
        let derived_key = kdf::derive_key(&kdf_password, &kdf_salt, 3, 19456, 32)?;
        let secret_key = aead::SecretKey::from_slice(derived_key.unprotected_as_bytes())?;
        Ok(secret_key)
    }

    /// Get encrypted settings file path
    fn encrypted_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("sed").join("sensitive.sed"))
    }

    /// Save sensitive settings encrypted with master password
    pub fn save_encrypted(&self, master_password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::encrypted_path()
            .ok_or_else(|| "Cannot find config directory".to_string())?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Generate salt
        let mut salt = [0u8; SETTINGS_SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);

        // Derive key
        let secret_key = Self::derive_key(master_password, &salt)?;

        // Serialize to JSON
        let json = Zeroizing::new(serde_json::to_string(self)?.into_bytes());

        // Encrypt
        let ciphertext = aead::seal(&secret_key, &json)?;

        // Password verification hash (SHA-256 of password + salt)
        let mut hasher = Sha256::new();
        hasher.update(master_password.as_bytes());
        hasher.update(&salt);
        let password_hash = hasher.finalize();

        // Assemble: [MAGIC 4B] [SALT 32B] [PASSWORD_HASH 32B] [ENCRYPTED DATA]
        let mut file_data = Vec::new();
        file_data.extend_from_slice(SETTINGS_MAGIC);
        file_data.extend_from_slice(&salt);
        file_data.extend_from_slice(&password_hash);
        file_data.extend_from_slice(&ciphertext);

        fs::write(&path, file_data)?;
        Ok(())
    }

    /// Load sensitive settings decrypted with master password
    pub fn load_encrypted(master_password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::encrypted_path()
            .ok_or_else(|| "Cannot find config directory".to_string())?;

        let file_data = fs::read(&path)?;

        // Validate minimum size: magic + salt + password_hash + some data
        if file_data.len() < 4 + SETTINGS_SALT_SIZE + 32 + 1 {
            return Err("Invalid encrypted settings file".into());
        }

        // Validate magic
        if &file_data[0..4] != SETTINGS_MAGIC {
            return Err("Not an encrypted settings file".into());
        }

        // Extract components
        let salt = &file_data[4..4 + SETTINGS_SALT_SIZE];
        let stored_hash = &file_data[4 + SETTINGS_SALT_SIZE..4 + SETTINGS_SALT_SIZE + 32];
        let encrypted_data = &file_data[4 + SETTINGS_SALT_SIZE + 32..];

        // Verify password before expensive decryption
        let mut hasher = Sha256::new();
        hasher.update(master_password.as_bytes());
        hasher.update(salt);
        let computed_hash = hasher.finalize();
        if computed_hash.as_slice() != stored_hash {
            return Err("Wrong master password".into());
        }

        // Derive key and decrypt
        let secret_key = Self::derive_key(master_password, salt)?;
        let plaintext = aead::open(&secret_key, encrypted_data)
            .map_err(|_| "Decryption failed")?;

        // Parse JSON
        let settings: SensitiveSettings = serde_json::from_slice(&plaintext)?;
        Ok(settings)
    }

    /// Check if encrypted settings file exists
    pub fn encrypted_file_exists() -> bool {
        Self::encrypted_path()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Delete encrypted settings file
    pub fn delete_encrypted() -> Result<(), std::io::Error> {
        if let Some(path) = Self::encrypted_path() {
            if path.exists() {
                fs::remove_file(&path)?;
            }
        }
        Ok(())
    }
}
