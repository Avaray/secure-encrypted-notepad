use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// User preferences - persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Font size (8-32 px)
    pub font_size: f32,
    
    /// Font name (e.g. "Monospace", "Proportional")
    pub font_family: String,
    
    /// Theme: true = dark, false = light
    pub dark_theme: bool,
    
    /// Path to last used keyfile (optional)
    pub last_keyfile_path: Option<PathBuf>,
    
    /// Whether to remember keyfile path
    pub remember_keyfile_path: bool,
    
    /// Whether to automatically create snapshot on Save
    pub auto_snapshot_on_save: bool,
    
    /// Number of days to keep history (0 = no limit)
    pub snapshot_retention_days: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 14.0,
            font_family: "Monospace".to_string(),
            dark_theme: true,
            last_keyfile_path: None,
            remember_keyfile_path: false,
            auto_snapshot_on_save: true,  // Enabled by default
            snapshot_retention_days: 30,  // 30 days by default
        }
    }
}

/// Settings-related errors
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
    /// Returns path to configuration file
    fn config_path() -> Result<PathBuf, SettingsError> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| {
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
        match Self::load_internal() {
            Ok(settings) => settings,
            Err(e) => {
                eprintln!("Failed to load settings: {}. Using defaults.", e);
                Self::default()
            }
        }
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
    pub fn save(&self) -> Result<(), SettingsError> {
        let config_path = Self::config_path()?;
        
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(&config_path, toml_string)?;
        
        Ok(())
    }
    
    /// Validate font size (8-32 px)
    pub fn validate_font_size(&mut self) {
        self.font_size = self.font_size.clamp(8.0, 32.0);
    }
    
    /// Validate retention days (0-365)
    pub fn validate_retention_days(&mut self) {
        self.snapshot_retention_days = self.snapshot_retention_days.clamp(0, 365);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.font_size, 14.0);
        assert!(settings.auto_snapshot_on_save);
        assert_eq!(settings.snapshot_retention_days, 30);
    }
}
