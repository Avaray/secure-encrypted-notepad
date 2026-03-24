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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background: [u8; 3],
    pub foreground: [u8; 3],
    #[serde(default)]
    pub editor_foreground: Option<[u8; 3]>,
    #[serde(default)]
    pub button_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub button_fg: Option<[u8; 3]>,
    #[serde(default)]
    pub separator: Option<[u8; 3]>,
    #[serde(default)]
    pub button_hover_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub button_active_bg: Option<[u8; 3]>,
    pub selection_background: [u8; 3],
    pub cursor: [u8; 3],
    pub line_number: [u8; 3],
    pub comment: [u8; 3],
    pub icon_hover: [u8; 3],
    /// Default icon tint color (non-hovered state)
    #[serde(default)]
    pub icon_color: Option<[u8; 3]>,
    /// Highlight color for search matches and history loaded indicator
    #[serde(default)]
    pub highlight: Option<[u8; 3]>,
    pub success: [u8; 3],
    pub info: [u8; 3],
    pub warning: [u8; 3],
    pub error: [u8; 3],
    /// Color for whitespace symbols (spaces, tabs, returns)
    #[serde(default)]
    pub whitespace_symbols: Option<[u8; 3]>,
    
    // --- Typography ---
    #[serde(default)]
    pub heading_text: Option<[u8; 3]>,
    #[serde(default)]
    pub label_text: Option<[u8; 3]>,
    #[serde(default)]
    pub weak_text: Option<[u8; 3]>,
    #[serde(default)]
    pub strong_text: Option<[u8; 3]>,
    #[serde(default)]
    pub hyperlink: Option<[u8; 3]>,

    // --- Interactive Widgets ---
    #[serde(default)]
    pub checkbox_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub checkbox_check: Option<[u8; 3]>,
    #[serde(default)]
    pub slider_rail: Option<[u8; 3]>,
    #[serde(default)]
    pub slider_thumb: Option<[u8; 3]>,
    #[serde(default)]
    pub scrollbar_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub scrollbar_thumb: Option<[u8; 3]>,
    #[serde(default)]
    pub tooltip_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub tooltip_text: Option<[u8; 3]>,

    // --- Editor Additions ---
    #[serde(default)]
    pub editor_background: Option<[u8; 3]>,
    #[serde(default)]
    pub text_edit_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub focus_outline: Option<[u8; 3]>,
    #[serde(default)]
    pub selection_text: Option<[u8; 3]>,

    // --- Geometry & Borders ---
    #[serde(default)]
    pub window_rounding: Option<f32>,
    #[serde(default)]
    pub button_rounding: Option<f32>,
    #[serde(default)]
    pub button_border_width: Option<f32>,
    #[serde(default)]
    pub button_border_color: Option<[u8; 3]>,
    #[serde(default)]
    pub button_padding_x: Option<f32>,
    #[serde(default)]
    pub button_padding_y: Option<f32>,
    #[serde(default)]
    pub separator_width: Option<f32>,
    #[serde(default)]
    pub shadow_color: Option<[u8; 3]>,
    #[serde(default)]
    pub shadow_blur: Option<f32>,
    #[serde(default)]
    pub shadow_spread: Option<f32>,
    #[serde(default)]
    pub shadow_offset_x: Option<f32>,
    #[serde(default)]
    pub shadow_offset_y: Option<f32>,

    // --- Granular Button States ---
    #[serde(default)]
    pub button_hover_fg: Option<[u8; 3]>,
    #[serde(default)]
    pub button_active_fg: Option<[u8; 3]>,
    #[serde(default)]
    pub button_hover_border_color: Option<[u8; 3]>,
    #[serde(default)]
    pub button_active_border_color: Option<[u8; 3]>,

    // --- Granular Input Fields ---
    #[serde(default)]
    pub input_bg: Option<[u8; 3]>,
    #[serde(default)]
    pub input_fg: Option<[u8; 3]>,
    #[serde(default)]
    pub input_border_color: Option<[u8; 3]>,
    #[serde(default)]
    pub input_focus_border_color: Option<[u8; 3]>,
    #[serde(default)]
    pub input_rounding: Option<f32>,
    /// Color for tree view lines (indentation guides)
    #[serde(default)]
    pub tree_line: Option<[u8; 3]>,
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
            editor_foreground: None,
            button_bg: None, // Use egui default or derived
            button_fg: None,
            separator: None,
            button_hover_bg: None, // Derived usually which is good
            button_active_bg: None,
            selection_background: [51, 51, 51],
            cursor: [255, 255, 255],
            line_number: [128, 128, 128],
            comment: [106, 153, 85],
            icon_hover: [100, 150, 255],
            icon_color: None, // defaults to [200, 200, 200]
            highlight: None,  // defaults to cursor color at 35% opacity
            success: [76, 175, 80],
            info: [33, 150, 243],
            warning: [255, 152, 0],
            error: [244, 67, 54],
            whitespace_symbols: None,

            heading_text: None,
            label_text: None,
            weak_text: None,
            strong_text: None,
            hyperlink: None,
            checkbox_bg: None,
            checkbox_check: None,
            slider_rail: None,
            slider_thumb: None,
            scrollbar_bg: None,
            scrollbar_thumb: None,
            tooltip_bg: None,
            tooltip_text: None,
            editor_background: None,
            text_edit_bg: None,
            focus_outline: None,
            selection_text: None,
            window_rounding: None,
            button_rounding: None,
            button_border_width: None,
            button_border_color: None,
            button_padding_x: None,
            button_padding_y: None,
            separator_width: None,
            shadow_color: None,
            shadow_blur: None,
            shadow_spread: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            button_hover_fg: None,
            button_active_fg: None,
            button_hover_border_color: None,
            button_active_border_color: None,
            input_bg: None,
            input_fg: None,
            input_border_color: None,
            input_focus_border_color: None,
            input_rounding: None,
            tree_line: None,
        }
    }

    pub fn light() -> Self {
        Self {
            background: [255, 255, 255],
            foreground: [0, 0, 0],
            editor_foreground: None,
            button_bg: None,
            button_fg: None,
            separator: None,
            button_hover_bg: None,
            button_active_bg: None,
            selection_background: [173, 214, 255],
            cursor: [0, 0, 0],
            line_number: [128, 128, 128],
            comment: [0, 128, 0],
            icon_hover: [0, 100, 255],
            icon_color: None, // defaults to [80, 80, 80]
            highlight: None,  // defaults to cursor color at 35% opacity
            success: [46, 125, 50],
            info: [13, 71, 161],
            warning: [230, 81, 0],
            error: [198, 40, 40],
            whitespace_symbols: None,

            heading_text: None,
            label_text: None,
            weak_text: None,
            strong_text: None,
            hyperlink: None,
            checkbox_bg: None,
            checkbox_check: None,
            slider_rail: None,
            slider_thumb: None,
            scrollbar_bg: None,
            scrollbar_thumb: None,
            tooltip_bg: None,
            tooltip_text: None,
            editor_background: None,
            text_edit_bg: None,
            focus_outline: None,
            selection_text: None,
            window_rounding: None,
            button_rounding: None,
            button_border_width: None,
            button_border_color: None,
            button_padding_x: None,
            button_padding_y: None,
            separator_width: None,
            shadow_color: None,
            shadow_blur: None,
            shadow_spread: None,
            shadow_offset_x: None,
            shadow_offset_y: None,
            button_hover_fg: None,
            button_active_fg: None,
            button_hover_border_color: None,
            button_active_border_color: None,
            input_bg: None,
            input_fg: None,
            input_border_color: None,
            input_focus_border_color: None,
            input_rounding: None,
            tree_line: None,
        }
    }

    pub fn editor_foreground_color(&self) -> egui::Color32 {
        let c = self.editor_foreground.unwrap_or(self.foreground);
        egui::Color32::from_rgb(c[0], c[1], c[2])
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

    pub fn icon_color(&self) -> egui::Color32 {
        let c = self.icon_color.unwrap_or(if self.background[0] > 128 {
            [80, 80, 80] // light theme
        } else {
            [200, 200, 200] // dark theme
        });
        egui::Color32::from_rgb(c[0], c[1], c[2])
    }

    pub fn highlight_color(&self) -> egui::Color32 {
        if let Some(h) = self.highlight {
            egui::Color32::from_rgb(h[0], h[1], h[2])
        } else {
            self.cursor_color().linear_multiply(0.35)
        }
    }
    
    pub fn hyperlink_color(&self) -> egui::Color32 {
        if let Some(c) = self.hyperlink {
            egui::Color32::from_rgb(c[0], c[1], c[2])
        } else {
            egui::Color32::from_rgb(90, 170, 255) // Default fallback generic blue
        }
    }

    pub fn comment_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(self.comment[0], self.comment[1], self.comment[2])
    }

    pub fn whitespace_symbols_color(&self) -> egui::Color32 {
        if let Some(c) = self.whitespace_symbols {
            egui::Color32::from_rgb(c[0], c[1], c[2])
        } else {
            self.comment_color().linear_multiply(0.4)
        }
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

    pub fn tree_line_color(&self, ui_visuals: &egui::Visuals) -> egui::Color32 {
        if let Some(c) = self.tree_line {
            egui::Color32::from_rgb(c[0], c[1], c[2])
        } else {
            ui_visuals.weak_text_color()
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

    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if self.color_scheme == ColorScheme::Light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        // --- Apply Global Background ---
        let bg_color = self.colors.to_egui_color32(self.colors.background);
        visuals.window_fill = bg_color;
        visuals.panel_fill = bg_color;
        visuals.extreme_bg_color = bg_color;

        // Custom text edit background if defined
        if let Some(c) = self.colors.text_edit_bg {
            visuals.extreme_bg_color = self.colors.to_egui_color32(c);
        }

        visuals.selection.bg_fill = self.colors.selection_color();
        
        // Custom selection text color
        if let Some(c) = self.colors.selection_text {
            visuals.selection.stroke.color = self.colors.to_egui_color32(c);
        } else {
            visuals.selection.stroke.color = self.colors.cursor_color();
        }

        visuals.text_cursor.stroke.color = self.colors.cursor_color();

        // Apply foreground (text) color
        let foreground = self.colors.to_egui_color32(self.colors.foreground);
        visuals.widgets.noninteractive.fg_stroke.color = foreground;
        visuals.widgets.inactive.fg_stroke.color = foreground;
        visuals.widgets.hovered.fg_stroke.color = foreground;
        visuals.widgets.active.fg_stroke.color = foreground;
        
        // Remove override_text_color to allow selective coloring
        visuals.override_text_color = None;

        // Apply custom label color
        if let Some(c) = self.colors.label_text {
            visuals.widgets.noninteractive.fg_stroke.color = self.colors.to_egui_color32(c);
        }

        // Apply custom text colors
        // Note: weak text is derived automatically by egui's alpha multiplier. Overriding `inactive` broke checkboxes.
        if let Some(_c) = self.colors.strong_text {
            // Strong text is usually just a font change, but we can set a main color override if desired.
        }
        if let Some(c) = self.colors.hyperlink {
            visuals.hyperlink_color = self.colors.to_egui_color32(c);
        }

        // Apply Button Colors
        if let Some(bg) = self.colors.button_bg {
            let bg_color = self.colors.to_egui_color32(bg);
            visuals.widgets.inactive.weak_bg_fill = bg_color;
            visuals.widgets.inactive.bg_fill = bg_color;

            // If hover/active not explicitly set, they will use egui defaults or be derived.
            // But we can set them to stay consistent if desired.
        }

        if let Some(hover_bg) = self.colors.button_hover_bg {
            let color = self.colors.to_egui_color32(hover_bg);
            visuals.widgets.hovered.weak_bg_fill = color;
            visuals.widgets.hovered.bg_fill = color;
        }

        if let Some(active_bg) = self.colors.button_active_bg {
            let color = self.colors.to_egui_color32(active_bg);
            visuals.widgets.active.weak_bg_fill = color;
            visuals.widgets.active.bg_fill = color;
        }
        if let Some(fg) = self.colors.button_fg {
            let fg_color = self.colors.to_egui_color32(fg);
            visuals.widgets.inactive.fg_stroke.color = fg_color;
            visuals.widgets.hovered.fg_stroke.color = fg_color;
            visuals.widgets.active.fg_stroke.color = fg_color;
        }

        // Apply Separator Color
        if let Some(sep) = self.colors.separator {
            let sep_color = self.colors.to_egui_color32(sep);
            visuals.widgets.noninteractive.bg_stroke.color = sep_color; // Used for separators
        }

        // Apply Focus Outline
        if let Some(c) = self.colors.focus_outline {
            visuals.selection.stroke = egui::Stroke::new(1.0, self.colors.to_egui_color32(c));
        }

        // Apply Shadow Color
        if let Some(c) = self.colors.shadow_color {
            visuals.window_shadow.color = self.colors.to_egui_color32(c);
            visuals.popup_shadow.color = self.colors.to_egui_color32(c);
        }
        
        if let Some(b) = self.colors.shadow_blur {
            visuals.window_shadow.blur = b as u8;
            visuals.popup_shadow.blur = b as u8;
        }
        
        if let Some(s) = self.colors.shadow_spread {
            visuals.window_shadow.spread = s as u8;
            visuals.popup_shadow.spread = s as u8;
        }

        if let Some(x) = self.colors.shadow_offset_x {
            visuals.window_shadow.offset[0] = x as i8;
            visuals.popup_shadow.offset[0] = x as i8;
        }
        
        if let Some(y) = self.colors.shadow_offset_y {
            visuals.window_shadow.offset[1] = y as i8;
            visuals.popup_shadow.offset[1] = y as i8;
        }

        // --- Apply Button States (Foreground & Border) ---
        if let Some(fg) = self.colors.button_hover_fg {
            visuals.widgets.hovered.fg_stroke.color = self.colors.to_egui_color32(fg);
        }
        if let Some(fg) = self.colors.button_active_fg {
            visuals.widgets.active.fg_stroke.color = self.colors.to_egui_color32(fg);
        }
        if let Some(c) = self.colors.button_hover_border_color {
            visuals.widgets.hovered.bg_stroke.color = self.colors.to_egui_color32(c);
        }
        if let Some(c) = self.colors.button_active_border_color {
            visuals.widgets.active.bg_stroke.color = self.colors.to_egui_color32(c);
        }

        // --- Apply Input Field Styles ---
        if let Some(bg) = self.colors.input_bg {
            visuals.extreme_bg_color = self.colors.to_egui_color32(bg);
            // Also apply to text_edit_bg if explicitly requested as input_bg
            visuals.widgets.active.bg_fill = self.colors.to_egui_color32(bg); 
            visuals.widgets.hovered.bg_fill = self.colors.to_egui_color32(bg);
        }
        
        if let Some(fg) = self.colors.input_fg {
            let color = self.colors.to_egui_color32(fg);
            // extreme_bg has no fg, it uses widget fg usually.
            // We set it on active/hovered to affect focused inputs.
            visuals.widgets.active.fg_stroke.color = color;
            visuals.widgets.hovered.fg_stroke.color = color;
        }

        if let Some(c) = self.colors.input_border_color {
            let color = self.colors.to_egui_color32(c);
            let stroke = egui::Stroke::new(1.0, color);
            visuals.widgets.inactive.bg_stroke = stroke;
            visuals.widgets.hovered.bg_stroke = stroke;
        }

        if let Some(c) = self.colors.input_focus_border_color {
             visuals.selection.stroke.color = self.colors.to_egui_color32(c);
        }



        ctx.set_visuals(visuals);

        // --- Apply Global Style Additions ---
        let mut style = (*ctx.style()).clone();
        
        if let Some(r) = self.colors.window_rounding {
            style.visuals.window_corner_radius = egui::CornerRadius::same(r as u8);
        }
        
        if let Some(r) = self.colors.button_rounding {
            let radius = egui::CornerRadius::same(r as u8);
            style.visuals.widgets.noninteractive.corner_radius = radius;
            style.visuals.widgets.inactive.corner_radius = radius;
            style.visuals.widgets.hovered.corner_radius = radius;
            style.visuals.widgets.active.corner_radius = radius;
            style.visuals.widgets.open.corner_radius = radius;
        }

        if let Some(r) = self.colors.input_rounding {
            let radius = egui::CornerRadius::same(r as u8);
            style.visuals.widgets.inactive.corner_radius = radius;
            style.visuals.widgets.hovered.corner_radius = radius;
            style.visuals.widgets.active.corner_radius = radius;
        }

        if let Some(w) = self.colors.button_border_width {
            style.visuals.widgets.inactive.bg_stroke.width = w;
            style.visuals.widgets.hovered.bg_stroke.width = w;
            style.visuals.widgets.active.bg_stroke.width = w;
        }

        if let Some(c) = self.colors.button_border_color {
            let stroke_color = self.colors.to_egui_color32(c);
            style.visuals.widgets.inactive.bg_stroke.color = stroke_color;
        }

        if let Some(x) = self.colors.button_padding_x {
            style.spacing.button_padding.x = x;
        }
        if let Some(y) = self.colors.button_padding_y {
            style.spacing.button_padding.y = y;
        }

        if let Some(w) = self.colors.separator_width {
            style.visuals.widgets.noninteractive.bg_stroke.width = w;
        }

        if let Some(c) = self.colors.scrollbar_bg {
            style.visuals.extreme_bg_color = self.colors.to_egui_color32(c); // Often used as scrollbar rail fallback
        }

        // Increase checkbox (icon) size
        style.spacing.icon_width = 26.0; 
        style.spacing.icon_spacing = 10.0;

        // Thicker checkmark (tick) icon
        style.visuals.widgets.inactive.fg_stroke.width = 2.0;
        style.visuals.widgets.hovered.fg_stroke.width = 2.2;
        style.visuals.widgets.active.fg_stroke.width = 2.5;

        ctx.set_style(style);
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
    fs::write(path, toml_string)?;
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
        assert_eq!(dark.colors.background, [27, 27, 27]);

        let light = Theme::light();
        assert_eq!(light.name, "Light");
        assert_eq!(light.color_scheme, ColorScheme::Light);
        assert_eq!(light.colors.background, [255, 255, 255]);
    }
}
