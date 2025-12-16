use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User preferences - persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// UI font size (8-32 px)
    pub ui_font_size: f32,

    /// Editor font size (8-32 px)
    pub editor_font_size: f32,

    /// Current theme name
    pub theme_name: String,

    /// Path to global default keyfile
    pub global_keyfile_path: Option<PathBuf>,

    /// Whether to use global keyfile automatically on startup
    pub use_global_keyfile: bool,

    /// Whether to auto-create snapshot on save (when content changes)
    pub auto_snapshot_on_save: bool,

    /// Show line numbers in editor
    pub show_line_numbers: bool,

    /// Show file tree panel
    pub show_file_tree: bool,

    /// Show debug panel
    pub show_debug_panel: bool,

    /// Last opened directory for file tree
    pub last_directory: Option<PathBuf>,

    /// File tree panel width
    pub file_tree_width: f32,

    /// Show subfolders in file tree
    pub show_subfolders: bool,

    // Max history length
    pub max_history_length: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_font_size: 20.0,
            editor_font_size: 20.0,
            theme_name: "Dark".to_string(),
            global_keyfile_path: None,
            use_global_keyfile: false,
            auto_snapshot_on_save: true,
            show_line_numbers: true,
            show_file_tree: false,
            show_debug_panel: false,
            last_directory: None,
            file_tree_width: 200.0,
            show_subfolders: true,
            max_history_length: 100,
        }
    }
}

/// Settings errors
#[derive(Debug)]
pub enum SettingsError {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
    SerializeError(toml::ser::Error),
}

impl From<std::io::Error> for SettingsError {
    fn from(err: std::io::Error) -> Self {
        SettingsError::IoError(err)
    }
}

impl From<toml::de::Error> for SettingsError {
    fn from(err: toml::de::Error) -> Self {
        SettingsError::TomlError(err)
    }
}

impl From<toml::ser::Error> for SettingsError {
    fn from(err: toml::ser::Error) -> Self {
        SettingsError::SerializeError(err)
    }
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SettingsError::IoError(e) => write!(f, "IO Error: {}", e),
            SettingsError::TomlError(e) => write!(f, "TOML Parse Error: {}", e),
            SettingsError::SerializeError(e) => write!(f, "TOML Serialize Error: {}", e),
        }
    }
}

impl std::error::Error for SettingsError {}

impl Settings {
    /// Get config file path
    fn config_path() -> Result<PathBuf, SettingsError> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            SettingsError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot find config directory",
            ))
        })?;

        let app_config_dir = config_dir.join("sed");

        if !app_config_dir.exists() {
            fs::create_dir_all(&app_config_dir)?;
        }

        Ok(app_config_dir.join("settings.toml"))
    }

    /// Load settings from file
    pub fn load() -> Self {
        if let Some(config_dir) = dirs::config_dir() {
            let config_path = config_dir.join("sed").join("config.toml");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(settings) = toml::from_str(&content) {
                        return settings;
                    }
                }
            }
        }
        Self::default()
    }

    fn load_internal() -> Result<Self, SettingsError> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let settings = Self::default();
            let _ = settings.save();
            return Ok(settings);
        }

        let content = fs::read_to_string(&config_path)?;
        let settings: Settings = toml::from_str(&content)?;

        Ok(settings)
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(config_dir) = dirs::config_dir() {
            let config_dir = config_dir.join("sed");
            fs::create_dir_all(&config_dir)?;
            let config_path = config_dir.join("config.toml");
            let toml_string = toml::to_string_pretty(self)?;
            fs::write(config_path, toml_string)?;
        }
        Ok(())
    }

    /// Validate font sizes
    pub fn validate_font_sizes(&mut self) {
        self.ui_font_size = self.ui_font_size.clamp(8.0, 32.0);
        self.editor_font_size = self.editor_font_size.clamp(8.0, 32.0);
    }

    // Validate history length
    pub fn validate_history_length(&mut self) {
        self.max_history_length = self.max_history_length.clamp(10, 1000);
    }
}
