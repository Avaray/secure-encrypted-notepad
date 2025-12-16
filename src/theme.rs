use eframe::egui;
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

/// Color scheme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    pub panel_background: [u8; 3],
    pub selection_background: [u8; 3],
    pub cursor: [u8; 3],
    pub line_number: [u8; 3],
    pub comment: [u8; 3],
    pub icon_hover: [u8; 3],
    pub success: [u8; 3],
    pub info: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    pub fn dark() -> Self {
        Self {
            background: [27, 27, 27],
            foreground: [255, 255, 255],
            panel_background: [37, 37, 37],
            selection_background: [51, 51, 51],
            cursor: [255, 255, 255],
            line_number: [128, 128, 128],
            comment: [106, 153, 85],
            icon_hover: [100, 150, 255],
            success: [76, 175, 80], // Green
            info: [33, 150, 243],   // Blue
            warning: [255, 152, 0], // Orange
            error: [244, 67, 54],   // Red
        }
    }

    pub fn light() -> Self {
        Self {
            background: [255, 255, 255],
            foreground: [0, 0, 0],
            panel_background: [245, 245, 245],
            selection_background: [173, 214, 255],
            cursor: [0, 0, 0],
            line_number: [128, 128, 128],
            comment: [0, 128, 0],
            icon_hover: [0, 100, 255],
            success: [46, 125, 50], // Dark green
            info: [13, 71, 161],    // Dark blue
            warning: [230, 81, 0],  // Dark orange
            error: [198, 40, 40],   // Dark red
        }
    }

    pub fn foreground_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.foreground[0], self.foreground[1], self.foreground[2])
    }

    pub fn to_egui_color32(&self, rgb: [u8; 3]) -> egui::Color32 {
        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
    }

    pub fn line_number_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.line_number[0],
            self.line_number[1],
            self.line_number[2],
        )
    }

    pub fn cursor_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.cursor[0], self.cursor[1], self.cursor[2])
    }

    pub fn selection_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(
            self.selection_background[0],
            self.selection_background[1],
            self.selection_background[2],
        )
    }

    pub fn icon_hover_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.icon_hover[0], self.icon_hover[1], self.icon_hover[2])
    }

    pub fn comment_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.comment[0], self.comment[1], self.comment[2])
    }

    pub fn success_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.success[0], self.success[1], self.success[2])
    }

    #[allow(dead_code)]
    pub fn info_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.info[0], self.info[1], self.info[2])
    }

    pub fn warning_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.warning[0], self.warning[1], self.warning[2])
    }

    #[allow(dead_code)]
    pub fn error_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.error[0], self.error[1], self.error[2])
    }
}

/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if self.color_scheme == ColorScheme::Light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        // Apply custom colors
        visuals.window_fill = self.colors.to_egui_color32(self.colors.background);
        visuals.panel_fill = self.colors.to_egui_color32(self.colors.panel_background);
        visuals.extreme_bg_color = self.colors.to_egui_color32(self.colors.panel_background);
        visuals.selection.bg_fill = self.colors.selection_color();
        visuals.selection.stroke.color = self.colors.cursor_color();

        // Apply foreground (text) color
        let foreground = self.colors.to_egui_color32(self.colors.foreground);
        visuals.widgets.noninteractive.fg_stroke.color = foreground;
        visuals.widgets.inactive.fg_stroke.color = foreground;
        visuals.widgets.active.fg_stroke.color = foreground;
        visuals.override_text_color = Some(foreground);

        ctx.set_visuals(visuals);
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
                                themes.push(theme);
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
    dirs::config_dir().map(|d| d.join("sed").join("themes"))
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
    fs::write(path, toml_string)?;
    Ok(())
}
