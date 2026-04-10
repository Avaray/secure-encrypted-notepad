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

/// Color scheme definition
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
    #[serde(default)]
    pub separator_width: Option<f32>,
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

    pub fn editor_foreground_color(&self) -> egui::Color32 {
        let c = self.editor_foreground.or(self.foreground).unwrap_or([255, 255, 255, 255]);
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    pub fn to_egui_color32(&self, rgba: [u8; 4]) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    fn to_color32_opt(&self, rgba: Option<[u8; 4]>, fallback: [u8; 4]) -> egui::Color32 {
        let c = rgba.unwrap_or(fallback);
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    pub fn line_number_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.line_number, [128, 128, 128, 255])
    }

    pub fn cursor_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.cursor, [255, 255, 255, 255])
    }

    pub fn selection_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.selection_background, [51, 51, 51, 255])
    }

    pub fn icon_hover_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.icon_hover, [100, 150, 255, 255])
    }

    pub fn icon_color(&self) -> egui::Color32 {
        if let Some(c) = self.icon_color {
            return egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
        }
        // Intelligent default if not defined
        let bg = self.background.unwrap_or([18, 18, 18, 255]);
        let c = if bg[0] > 128 {
            [80, 80, 80, 255] // light theme
        } else {
            [200, 200, 200, 255] // dark theme
        };
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    pub fn highlight_color(&self) -> egui::Color32 {
        if let Some(h) = self.highlight {
            egui::Color32::from_rgba_unmultiplied(h[0], h[1], h[2], h[3])
        } else {
            self.cursor_color().linear_multiply(0.35)
        }
    }

    pub fn hyperlink_color(&self) -> egui::Color32 {
        if let Some(c) = self.hyperlink {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            egui::Color32::from_rgba_unmultiplied(90, 170, 255, 255) // Default fallback generic blue
        }
    }

    pub fn comment_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.comment, [106, 153, 85, 255])
    }

    pub fn whitespace_symbols_color(&self) -> egui::Color32 {
        if let Some(c) = self.whitespace_symbols {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            self.comment_color().linear_multiply(0.4)
        }
    }

    pub fn success_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.success, [76, 175, 80, 255])
    }

    #[allow(dead_code)]
    pub fn info_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.info, [33, 150, 243, 255])
    }

    pub fn warning_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.warning, [255, 152, 0, 255])
    }

    #[allow(dead_code)]
    pub fn error_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.error, [244, 67, 54, 255])
    }

    pub fn heading_color(&self) -> egui::Color32 {
        if let Some(c) = self.heading_text {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            let fg = self.foreground.unwrap_or([255, 255, 255, 255]);
            self.to_egui_color32(fg)
        }
    }

    pub fn tree_line_color(&self, ui_visuals: &egui::Visuals) -> egui::Color32 {
        if let Some(c) = self.tree_line {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
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

        // Create a resolved copy of colors for application
        let mut colors = self.colors.clone();
        colors.resolve(self.color_scheme);

        // --- Apply Global Background ---
        let bg_color = colors.to_egui_color32(colors.background.unwrap_or([18, 18, 18, 255]));
        visuals.window_fill = bg_color;
        visuals.panel_fill = bg_color;
        visuals.extreme_bg_color = bg_color;

        // Custom text edit background if defined
        if let Some(c) = colors.text_edit_bg {
            visuals.extreme_bg_color = colors.to_egui_color32(c);
        }

        visuals.selection.bg_fill = colors.selection_color();

        // Custom selection text color
        if let Some(c) = colors.selection_text {
            visuals.selection.stroke.color = colors.to_egui_color32(c);
        } else {
            visuals.selection.stroke.color = colors.cursor_color();
        }

        visuals.text_cursor.stroke.color = colors.cursor_color();

        // Apply foreground (text) color
        let foreground = colors.to_egui_color32(colors.foreground.unwrap_or([255, 255, 255, 255]));
        visuals.widgets.noninteractive.fg_stroke.color = foreground;
        visuals.widgets.inactive.fg_stroke.color = foreground;
        visuals.widgets.hovered.fg_stroke.color = foreground;
        visuals.widgets.active.fg_stroke.color = foreground;

        // Remove override_text_color to allow selective coloring
        visuals.override_text_color = None;

        if let Some(c) = colors.hyperlink {
            visuals.hyperlink_color = colors.to_egui_color32(c);
        }

        // Apply Button Colors
        if let Some(bg) = colors.button_bg {
            let bg_color = colors.to_egui_color32(bg);
            visuals.widgets.inactive.weak_bg_fill = bg_color;
            visuals.widgets.inactive.bg_fill = bg_color;
        }

        if let Some(hover_bg) = colors.button_hover_bg {
            let color = colors.to_egui_color32(hover_bg);
            visuals.widgets.hovered.weak_bg_fill = color;
            visuals.widgets.hovered.bg_fill = color;
        }

        if let Some(active_bg) = colors.button_active_bg {
            let color = colors.to_egui_color32(active_bg);
            visuals.widgets.active.weak_bg_fill = color;
            visuals.widgets.active.bg_fill = color;
        }
        if let Some(fg) = colors.button_fg {
            let fg_color = colors.to_egui_color32(fg);
            // Only set inactive; hover/active have their own explicit fields
            visuals.widgets.inactive.fg_stroke.color = fg_color;
            // Also set hover/active as fallback, but specific overrides below will take priority
            if colors.button_hover_fg.is_none() {
                visuals.widgets.hovered.fg_stroke.color = fg_color;
            }
            if colors.button_active_fg.is_none() {
                visuals.widgets.active.fg_stroke.color = fg_color;
            }
        }

        // Apply Separator Color
        if let Some(sep) = colors.separator {
            let sep_color = colors.to_egui_color32(sep);
            visuals.widgets.noninteractive.bg_stroke.color = sep_color; // Used for separators
        }

        // Apply Focus/Selection Border (unified)
        if let Some(c) = colors.widget_focus_border {
            visuals.selection.stroke = egui::Stroke::new(1.0, colors.to_egui_color32(c));
        }

        // Apply Shadow Color
        if let Some(c) = colors.shadow_color {
            visuals.window_shadow.color = colors.to_egui_color32(c);
            visuals.popup_shadow.color = colors.to_egui_color32(c);
        }

        if let Some(b) = colors.shadow_blur {
            visuals.window_shadow.blur = b as u8;
            visuals.popup_shadow.blur = b as u8;
        }

        if let Some(s) = colors.shadow_spread {
            visuals.window_shadow.spread = s as u8;
            visuals.popup_shadow.spread = s as u8;
        }

        // When both spread and blur are zero, force shadow to be invisible
        {
            let spread = colors.shadow_spread.unwrap_or(0.0);
            let blur = colors.shadow_blur.unwrap_or(0.0);
            if spread == 0.0 && blur == 0.0 {
                visuals.window_shadow.color = egui::Color32::TRANSPARENT;
                visuals.popup_shadow.color = egui::Color32::TRANSPARENT;
            }
        }

        // Always apply shadow offset (default to 0 when not set,
        // otherwise egui's built-in non-zero defaults take over)
        let offset_x = colors.shadow_offset_x.unwrap_or(0.0) as i8;
        let offset_y = colors.shadow_offset_y.unwrap_or(0.0) as i8;
        visuals.window_shadow.offset = [offset_x, offset_y];
        visuals.popup_shadow.offset = [offset_x, offset_y];

        // --- Apply Button States (Foreground & Border) ---
        if let Some(fg) = colors.button_hover_fg {
            visuals.widgets.hovered.fg_stroke.color = colors.to_egui_color32(fg);
        }
        if let Some(fg) = colors.button_active_fg {
            visuals.widgets.active.fg_stroke.color = colors.to_egui_color32(fg);
        }
        if let Some(c) = colors.button_hover_border_color {
            visuals.widgets.hovered.bg_stroke.color = colors.to_egui_color32(c);
        }
        if let Some(c) = colors.button_active_border_color {
            visuals.widgets.active.bg_stroke.color = colors.to_egui_color32(c);
        }

        ctx.set_visuals(visuals);

        // --- Apply Global Style Additions ---
        let mut style = (*ctx.style()).clone();
        let default_style = egui::Style::default();

        if let Some(r) = colors.window_rounding {
            let radius = egui::CornerRadius::same(r as u8);
            style.visuals.window_corner_radius = radius;
            style.visuals.menu_corner_radius = radius;
        } else {
            style.visuals.window_corner_radius = default_style.visuals.window_corner_radius;
            style.visuals.menu_corner_radius = default_style.visuals.menu_corner_radius;
        }

        // Unified widget rounding (applies to buttons, inputs, checkboxes, etc.)
        if let Some(r) = colors.widget_rounding {
            let radius = egui::CornerRadius::same(r as u8);
            style.visuals.widgets.noninteractive.corner_radius = radius;
            style.visuals.widgets.inactive.corner_radius = radius;
            style.visuals.widgets.hovered.corner_radius = radius;
            style.visuals.widgets.active.corner_radius = radius;
            style.visuals.widgets.open.corner_radius = radius;
        } else {
            style.visuals.widgets.noninteractive.corner_radius = default_style.visuals.widgets.noninteractive.corner_radius;
            style.visuals.widgets.inactive.corner_radius = default_style.visuals.widgets.inactive.corner_radius;
            style.visuals.widgets.hovered.corner_radius = default_style.visuals.widgets.hovered.corner_radius;
            style.visuals.widgets.active.corner_radius = default_style.visuals.widgets.active.corner_radius;
            style.visuals.widgets.open.corner_radius = default_style.visuals.widgets.open.corner_radius;
        }

        // Unified widget border width
        if let Some(w) = colors.widget_border_width {
            style.visuals.widgets.inactive.bg_stroke.width = w;
            style.visuals.widgets.hovered.bg_stroke.width = w;
            style.visuals.widgets.active.bg_stroke.width = w;
        } else {
            style.visuals.widgets.inactive.bg_stroke.width = default_style.visuals.widgets.inactive.bg_stroke.width;
            style.visuals.widgets.hovered.bg_stroke.width = default_style.visuals.widgets.hovered.bg_stroke.width;
            style.visuals.widgets.active.bg_stroke.width = default_style.visuals.widgets.active.bg_stroke.width;
        }

        // Unified widget border color (idle state)
        if let Some(c) = colors.widget_border_color {
            let stroke_color = colors.to_egui_color32(c);
            style.visuals.widgets.inactive.bg_stroke.color = stroke_color;
        } else {
            let visuals_def = if self.color_scheme == ColorScheme::Light { egui::Visuals::light() } else { egui::Visuals::dark() };
            style.visuals.widgets.inactive.bg_stroke.color = visuals_def.widgets.inactive.bg_stroke.color;
        }

        // Unified widget padding
        if let Some(x) = colors.widget_padding_x {
            style.spacing.button_padding.x = x;
        } else {
            style.spacing.button_padding.x = default_style.spacing.button_padding.x;
        }
        if let Some(y) = colors.widget_padding_y {
            style.spacing.button_padding.y = y;
            // ComboBox and other widgets use interact_size.y as their default height.
            // Setting it to 0.0 forces them to use (text_height + 2 * padding.y),
            // making them consistent with buttons.
            style.spacing.interact_size.y = 0.0;
        } else {
            style.spacing.button_padding.y = default_style.spacing.button_padding.y;
            style.spacing.interact_size.y = default_style.spacing.interact_size.y;
        }

        // Separator width — only affects noninteractive bg_stroke width
        if let Some(w) = colors.separator_width {
            style.visuals.widgets.noninteractive.bg_stroke.width = w;
        } else {
            style.visuals.widgets.noninteractive.bg_stroke.width = default_style.visuals.widgets.noninteractive.bg_stroke.width;
        }

        // Increase checkbox (icon) size - adjusted to stay consistent with widget height
        style.spacing.icon_width = if colors.widget_padding_y.is_some() { 20.0 } else { 26.0 };
        style.spacing.icon_spacing = if colors.widget_padding_y.is_some() { 6.0 } else { 10.0 };

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
        assert_eq!(dark.colors.background, Some([18, 18, 18, 255]));

        let light = Theme::light();
        assert_eq!(light.name, "Light");
        assert_eq!(light.color_scheme, ColorScheme::Light);
        assert_eq!(light.colors.background, Some([245, 245, 245, 255]));
    }
}
