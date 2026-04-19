use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Color scheme type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorScheme {
    Dark,
    Light,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::Dark
    }
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

    // --- Interactive Widgets ---
    #[serde(default, with = "opt_alpha_color")]
    pub checkbox_check: Option<[u8; 4]>,

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

    // --- Granular Button States ---
    #[serde(default, with = "opt_alpha_color")]
    pub button_hover_fg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_active_fg: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_hover_border_color: Option<[u8; 4]>,
    #[serde(default, with = "opt_alpha_color")]
    pub button_active_border_color: Option<[u8; 4]>,

    /// Color for tree view lines (indentation guides)
    #[serde(default, with = "opt_alpha_color")]
    pub tree_line: Option<[u8; 4]>,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: Some([18, 18, 18, 255]),
            foreground: Some([255, 255, 255, 255]),
            editor_foreground: None,
            button_bg: None, // Use egui default or derived
            button_fg: None,
            separator: None,
            button_hover_bg: None, // Derived usually which is good
            button_active_bg: None,
            selection_background: Some([40, 40, 40, 255]),
            cursor: Some([255, 255, 255, 255]),
            line_number: Some([128, 128, 128, 255]),
            comment: Some([106, 153, 85, 255]),
            icon_hover: Some([100, 150, 255, 255]),
            icon_color: None, // defaults to [200, 200, 200, 255]
            highlight: None,  // defaults to cursor color at 35% opacity
            success: Some([76, 175, 80, 255]),
            info: Some([33, 150, 243, 255]),
            warning: Some([255, 152, 0, 255]),
            error: Some([244, 67, 54, 255]),
            whitespace_symbols: None,

            heading_text: None,
            hyperlink: None,
            checkbox_check: None,
            editor_background: None,
            text_edit_bg: None,
            selection_text: None,
            window_rounding: None,
            widget_rounding: None,
            widget_border_width: None,
            widget_border_color: None,
            widget_padding_x: None,
            widget_padding_y: None,
            widget_focus_border: None,

            shadow_color: None,
            shadow_blur: None,
            shadow_spread: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            button_hover_fg: None,
            button_active_fg: None,
            button_hover_border_color: None,
            button_active_border_color: None,
            tree_line: None,
        }
    }

    pub fn light() -> Self {
        Self {
            background: Some([245, 245, 245, 255]),
            foreground: Some([0, 0, 0, 255]),
            editor_foreground: None,
            button_bg: None,
            button_fg: None,
            separator: None,
            button_hover_bg: None,
            button_active_bg: None,
            selection_background: Some([210, 230, 255, 255]),
            cursor: Some([0, 0, 0, 255]),
            line_number: Some([128, 128, 128, 255]),
            comment: Some([0, 128, 0, 255]),
            icon_hover: Some([0, 100, 255, 255]),
            icon_color: None, // defaults to [80, 80, 80, 255]
            highlight: None,  // defaults to cursor color at 35% opacity
            success: Some([46, 125, 50, 255]),
            info: Some([13, 71, 161, 255]),
            warning: Some([230, 81, 0, 255]),
            error: Some([198, 40, 40, 255]),
            whitespace_symbols: None,

            heading_text: None,
            hyperlink: None,
            checkbox_check: None,
            editor_background: None,
            text_edit_bg: None,
            selection_text: None,
            window_rounding: None,
            widget_rounding: None,
            widget_border_width: None,
            widget_border_color: None,
            widget_padding_x: None,
            widget_padding_y: None,
            widget_focus_border: None,

            shadow_color: None,
            shadow_blur: None,
            shadow_spread: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            button_hover_fg: None,
            button_active_fg: None,
            button_hover_border_color: None,
            button_active_border_color: None,
            tree_line: None,
        }
    }

    /// Fill all None fields with defaults based on selected ColorScheme
    pub fn resolve(&mut self, scheme: ColorScheme) {
        let defaults = match scheme {
            ColorScheme::Dark => Self::dark(),
            ColorScheme::Light => Self::light(),
        };

        if self.background.is_none() {
            self.background = defaults.background;
        }
        if self.foreground.is_none() {
            self.foreground = defaults.foreground;
        }
        if self.selection_background.is_none() {
            self.selection_background = defaults.selection_background;
        }
        if self.cursor.is_none() {
            self.cursor = defaults.cursor;
        }
        if self.line_number.is_none() {
            self.line_number = defaults.line_number;
        }
        if self.comment.is_none() {
            self.comment = defaults.comment;
        }
        if self.icon_hover.is_none() {
            self.icon_hover = defaults.icon_hover;
        }
        if self.success.is_none() {
            self.success = defaults.success;
        }
        if self.info.is_none() {
            self.info = defaults.info;
        }
        if self.warning.is_none() {
            self.warning = defaults.warning;
        }
        if self.error.is_none() {
            self.error = defaults.error;
        }
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

    /// Get highlight color, defaulting to cursor color at 35% opacity.
    pub fn highlight_rgba(&self) -> [u8; 4] {
        if let Some(h) = self.highlight {
            h
        } else {
            let cursor = self.cursor.unwrap_or([255, 255, 255, 255]);
            // Approximate linear_multiply(0.35) by scaling alpha
            [
                cursor[0],
                cursor[1],
                cursor[2],
                (cursor[3] as f32 * 0.35) as u8,
            ]
        }
    }

    /// Get hyperlink color with default fallback.
    pub fn hyperlink_rgba(&self) -> [u8; 4] {
        self.hyperlink.unwrap_or([90, 170, 255, 255])
    }

    /// Get heading color, falling back to foreground.
    pub fn heading_rgba(&self) -> [u8; 4] {
        self.heading_text
            .or(self.foreground)
            .unwrap_or([255, 255, 255, 255])
    }

    /// Get whitespace symbols color, defaulting to comment color at 40% opacity.
    pub fn whitespace_symbols_rgba(&self) -> [u8; 4] {
        if let Some(c) = self.whitespace_symbols {
            c
        } else {
            let comment = self.comment.unwrap_or([106, 153, 85, 255]);
            [
                comment[0],
                comment[1],
                comment[2],
                (comment[3] as f32 * 0.4) as u8,
            ]
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
        Self {
            name: "Dark".to_string(),
            colors: ThemeColors::dark(),
            color_scheme: ColorScheme::Dark,
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            colors: ThemeColors::light(),
            color_scheme: ColorScheme::Light,
        }
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
    let filename = format!("{}.toml", theme_name.to_lowercase().replace(' ', "_"));
    let path = themes_dir.join(filename);

    if path.exists() {
        fs::remove_file(path)?;
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
