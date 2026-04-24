use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

static DARK_THEME: OnceLock<Theme> = OnceLock::new();
static LIGHT_THEME: OnceLock<Theme> = OnceLock::new();

/// Color scheme type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    #[default]
    Dark,
    Light,
}

pub mod opt_alpha_color {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &Option<[u8; 4]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(c) = color {
            serializer.serialize_some(c)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 4]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt_vec = Option::<Vec<u8>>::deserialize(deserializer)?;
        match opt_vec {
            Some(vec) => match vec.len() {
                3 => Ok(Some([vec[0], vec[1], vec[2], 255])),
                4 => Ok(Some([vec[0], vec[1], vec[2], vec[3]])),
                _ => Err(serde::de::Error::custom("expected array of length 3 or 4")),
            },
            None => Ok(None),
        }
    }
}

/// Color scheme definition — all colors stored as UI-agnostic `[u8; 4]` RGBA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    #[serde(default, with = "opt_alpha_color")]
    pub background: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub foreground: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub editor_foreground: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_fg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub separator: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_hover_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_active_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub selection_background: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub cursor: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub line_number: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub comment: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub icon_hover: Option<[u8; 4]>,
    /// Default icon tint color (non-hovered state)
    #[serde(default, with = "opt_alpha_color")]
    pub icon_color: Option<[u8; 4]>,
    /// Highlight color for search matches and history loaded indicator
    #[serde(default, with = "opt_alpha_color")]
    pub highlight: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub find_match_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub find_current_match_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub success: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub info: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub warning: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub error: Option<[u8; 4]>,
    /// Color for whitespace symbols (spaces, tabs, returns)
    #[serde(default, with = "opt_alpha_color")]
    pub whitespace_symbols: Option<[u8; 4]>,

    // --- Typography ---
    #[serde(default, with = "opt_alpha_color")]
    pub heading_text: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub hyperlink: Option<[u8; 4]>,

    // --- Editor Additions ---
    #[serde(default, with = "opt_alpha_color")]
    pub editor_background: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub text_edit_bg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub selection_text: Option<[u8; 4]>,

    // --- Geometry & Borders (unified for all interactive widgets) ---
    #[serde(default)]
    pub window_rounding: Option<f32>,
    #[serde(default)]
    pub widget_rounding: Option<f32>,
    #[serde(default)]
    pub widget_border_width: Option<f32>,
    #[serde(default, with = "opt_alpha_color")]
    pub widget_border_color: Option<[u8; 4]>,
    #[serde(default)]
    pub widget_padding_x: Option<f32>,
    #[serde(default)]
    pub widget_padding_y: Option<f32>,
    /// Focus/selection border color for all interactive widgets
    #[serde(default, with = "opt_alpha_color")]
    pub widget_focus_border: Option<[u8; 4]>,

    #[serde(default, with = "opt_alpha_color")]
    pub shadow_color: Option<[u8; 4]>,
    #[serde(default)]
    pub shadow_blur: Option<f32>,
    #[serde(default)]
    pub shadow_spread: Option<f32>,
    #[serde(default)]
    pub shadow_offset_x: Option<f32>,
    #[serde(default)]
    pub shadow_offset_y: Option<f32>,

    // --- Scrollbars ---
    #[serde(default, with = "opt_alpha_color")]
    pub scrollbar_idle: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub scrollbar_hover: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub scrollbar_active: Option<[u8; 4]>,

    // --- Granular Button States ---
    #[serde(default, with = "opt_alpha_color")]
    pub button_hover_fg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_active_fg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_hover_border_color: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_active_border_color: Option<[u8; 4]>,

    // --- File Tree ---
    #[serde(default, with = "opt_alpha_color")]
    pub tree_file_stealth: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub tree_file_unlocked: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub tree_file_locked: Option<[u8; 4]>,
    /// Color for tree view lines (indentation guides)
    #[serde(default, with = "opt_alpha_color")]
    pub tree_line: Option<[u8; 4]>,

    // --- Miscellaneous ---
    #[serde(default, with = "opt_alpha_color")]
    pub modal_overlay_color: Option<[u8; 4]>,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Theme::dark().colors
    }

    pub fn light() -> Self {
        Theme::light().colors
    }

    /// Fill all None fields with defaults based on selected ColorScheme
    pub fn resolve(&mut self, scheme: ColorScheme) {
        let defaults = match scheme {
            ColorScheme::Dark => Self::dark(),
            ColorScheme::Light => Self::light(),
        };

        macro_rules! res {
            ($field:ident) => {
                if self.$field.is_none() {
                    self.$field = defaults.$field;
                }
            };
        }

        res!(background);
        res!(foreground);
        res!(editor_foreground);
        res!(button_bg);
        res!(button_fg);
        res!(separator);
        res!(button_hover_bg);
        res!(button_active_bg);
        res!(selection_background);
        res!(cursor);
        res!(line_number);
        res!(comment);
        res!(icon_hover);
        res!(icon_color);
        res!(highlight);
        res!(find_match_bg);
        res!(find_current_match_bg);
        res!(success);
        res!(info);
        res!(warning);
        res!(error);
        res!(whitespace_symbols);
        res!(heading_text);
        res!(hyperlink);
        res!(scrollbar_idle);
        res!(scrollbar_hover);
        res!(scrollbar_active);
        res!(widget_rounding);
        res!(widget_border_color);
        res!(widget_border_width);
        res!(widget_padding_x);
        res!(widget_padding_y);
        res!(widget_focus_border);
        res!(button_hover_fg);
        res!(button_active_fg);
        res!(button_hover_border_color);
        res!(button_active_border_color);
        res!(shadow_color);
        res!(shadow_blur);
        res!(shadow_spread);
        res!(shadow_offset_x);
        res!(shadow_offset_y);
        res!(tree_file_stealth);
        res!(tree_file_unlocked);
        res!(tree_file_locked);
        res!(tree_line);
        res!(modal_overlay_color);
    }

    /// Get the resolved RGBA value for a color field with a fallback.
    pub fn resolve_color(&self, rgba: Option<[u8; 4]>, fallback: [u8; 4]) -> [u8; 4] {
        rgba.unwrap_or(fallback)
    }

    /// Get the editor foreground color, falling back to main foreground.
    pub fn editor_foreground_rgba(&self) -> [u8; 4] {
        self.editor_foreground
            .or(self.foreground)
            .unwrap_or([255, 255, 255, 255])
    }

    /// Get icon color with intelligent dark/light default.
    pub fn icon_color_rgba(&self) -> [u8; 4] {
        if let Some(c) = self.icon_color {
            return c;
        }
        let bg = self.background.unwrap_or([18, 18, 18, 255]);
        if bg[0] > 128 {
            [80, 80, 80, 255] // light theme
        } else {
            [200, 200, 200, 255] // dark theme
        }
    }

    /// Get highlight color, defaulting to foreground color at 30% opacity.
    pub fn highlight_rgba(&self) -> [u8; 4] {
        if let Some(h) = self.highlight {
            h
        } else {
            let fg = self.foreground.unwrap_or([255, 255, 255, 255]);
            // Approximate linear_multiply(0.30) by scaling alpha
            [fg[0], fg[1], fg[2], (fg[3] as f32 * 0.30) as u8]
        }
    }

    /// Get modal overlay color, defaulting to halfway between background and pure black.
    pub fn modal_overlay_color_rgba(&self) -> [u8; 4] {
        if let Some(c) = self.modal_overlay_color {
            c
        } else {
            let bg = self.background.unwrap_or([18, 18, 18, 255]);
            // Calculate halfway point to black
            [bg[0] / 2, bg[1] / 2, bg[2] / 2, bg[3]]
        }
    }

    /// Get hyperlink color, defaulting to info color.
    pub fn hyperlink_rgba(&self) -> [u8; 4] {
        self.hyperlink
            .unwrap_or_else(|| self.info.unwrap_or([33, 150, 243, 255]))
    }

    /// Get heading color, falling back to foreground.
    pub fn heading_rgba(&self) -> [u8; 4] {
        self.heading_text
            .or(self.foreground)
            .unwrap_or([255, 255, 255, 255])
    }

    /// Get whitespace symbols color, defaulting to foreground text color at 30% opacity.
    pub fn whitespace_symbols_rgba(&self) -> [u8; 4] {
        if let Some(c) = self.whitespace_symbols {
            c
        } else {
            let fg = self.foreground.unwrap_or([255, 255, 255, 255]);
            [fg[0], fg[1], fg[2], (fg[3] as f32 * 0.30) as u8]
        }
    }
}

/// Complete theme definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    #[serde(default)]
    pub color_scheme: ColorScheme,
}

impl Theme {
    pub fn dark() -> Self {
        DARK_THEME
            .get_or_init(|| {
                let content = include_str!("../themes/dark.toml");
                toml::from_str::<Theme>(content).expect("Failed to parse embedded dark.toml")
            })
            .clone()
    }

    pub fn light() -> Self {
        LIGHT_THEME
            .get_or_init(|| {
                let content = include_str!("../themes/light.toml");
                toml::from_str::<Theme>(content).expect("Failed to parse embedded light.toml")
            })
            .clone()
    }
}

/// Load all available themes from themes directory
pub fn load_themes() -> Vec<Theme> {
    let mut themes = vec![Theme::dark(), Theme::light()];

    if let Some(themes_dir) = get_themes_dir() {
        if themes_dir.exists() {
            if let Ok(entries) = fs::read_dir(&themes_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(theme) = toml::from_str::<Theme>(&content) {
                                // Only add if a theme with this name doesn't exist yet
                                if !themes.iter().any(|t| t.name == theme.name) {
                                    themes.push(theme);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    themes
}

/// Get themes directory path
pub fn get_themes_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sen").join("themes"))
}

/// Ensure themes directory exists
pub fn ensure_themes_dir() -> Result<PathBuf, std::io::Error> {
    let themes_dir = get_themes_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot find config directory")
    })?;

    if !themes_dir.exists() {
        fs::create_dir_all(&themes_dir)?;
    }

    Ok(themes_dir)
}

/// Save theme to file
pub fn save_theme(theme: &Theme) -> Result<(), Box<dyn std::error::Error>> {
    let themes_dir = ensure_themes_dir()?;
    let filename = format!("{}.toml", theme.name.to_lowercase().replace(' ', "_"));
    let path = themes_dir.join(filename);
    let toml_string = toml::to_string_pretty(theme)?;
    crate::fs::atomic_write(path, toml_string)?;
    Ok(())
}

/// Delete theme file
pub fn delete_theme(theme_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let themes_dir = ensure_themes_dir()?;

    // First try the exact filename heuristic
    let filename = format!("{}.toml", theme_name.to_lowercase().replace(' ', "_"));
    let path = themes_dir.join(filename);
    if path.exists() {
        fs::remove_file(path)?;
        return Ok(());
    }

    // If not found, scan all TOML files and parse them to find the matching theme name
    if let Ok(entries) = fs::read_dir(&themes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(theme) = toml::from_str::<Theme>(&content) {
                        if theme.name == theme_name {
                            fs::remove_file(path)?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_themes() {
        let dark = Theme::dark();
        assert_eq!(dark.name, "Dark");
        assert_eq!(dark.color_scheme, ColorScheme::Dark);
        assert_eq!(dark.colors.background, Some([18, 18, 18, 255]));

        let light = Theme::light();
        assert_eq!(light.name, "Light");
        assert_eq!(light.color_scheme, ColorScheme::Light);
        assert_eq!(light.colors.background, Some([245, 245, 245, 255]));
    }
}
