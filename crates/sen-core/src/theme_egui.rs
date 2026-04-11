//! Shared egui extensions for the core Theme system.
//!
//! Provides `ThemeColorsExt` and `ThemeExt` traits that add
//! `egui::Color32` conversion methods and `Theme::apply()` to the
//! UI-agnostic data models.

use egui;
use crate::theme::{ColorScheme, Theme, ThemeColors};

// ─── ThemeColors egui helpers ───────────────────────────────────────────────

/// Extension trait adding egui-specific color conversion methods to `ThemeColors`.
pub trait ThemeColorsExt {
    fn to_egui_color32(&self, rgba: [u8; 4]) -> egui::Color32;
    fn to_color32_opt(&self, rgba: Option<[u8; 4]>, fallback: [u8; 4]) -> egui::Color32;

    fn editor_foreground_color(&self) -> egui::Color32;
    fn line_number_color(&self) -> egui::Color32;
    fn cursor_color(&self) -> egui::Color32;
    fn selection_color(&self) -> egui::Color32;
    fn icon_hover_color(&self) -> egui::Color32;
    fn icon_color(&self) -> egui::Color32;
    fn highlight_color(&self) -> egui::Color32;
    fn hyperlink_color(&self) -> egui::Color32;
    fn comment_color(&self) -> egui::Color32;
    fn whitespace_symbols_color(&self) -> egui::Color32;
    fn success_color(&self) -> egui::Color32;
    fn info_color(&self) -> egui::Color32;
    fn warning_color(&self) -> egui::Color32;
    fn error_color(&self) -> egui::Color32;
    fn heading_color(&self) -> egui::Color32;
    fn background_color(&self) -> egui::Color32;
    fn primary_color(&self) -> egui::Color32;
    fn tree_line_color(&self, ui_visuals: &egui::Visuals) -> egui::Color32;
}

impl ThemeColorsExt for ThemeColors {
    fn to_egui_color32(&self, rgba: [u8; 4]) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3])
    }

    fn to_color32_opt(&self, rgba: Option<[u8; 4]>, fallback: [u8; 4]) -> egui::Color32 {
        let c = rgba.unwrap_or(fallback);
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    fn editor_foreground_color(&self) -> egui::Color32 {
        let c = self
            .editor_foreground
            .or(self.foreground)
            .unwrap_or([255, 255, 255, 255]);
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    fn line_number_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.line_number, [128, 128, 128, 255])
    }

    fn cursor_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.cursor, [255, 255, 255, 255])
    }

    fn selection_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.selection_background, [51, 51, 51, 255])
    }

    fn icon_hover_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.icon_hover, [100, 150, 255, 255])
    }

    fn icon_color(&self) -> egui::Color32 {
        if let Some(c) = self.icon_color {
            return egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
        }
        let bg = self.background.unwrap_or([18, 18, 18, 255]);
        let c = if bg[0] > 128 {
            [80, 80, 80, 255]
        } else {
            [200, 200, 200, 255]
        };
        egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }

    fn highlight_color(&self) -> egui::Color32 {
        if let Some(h) = self.highlight {
            egui::Color32::from_rgba_unmultiplied(h[0], h[1], h[2], h[3])
        } else {
            self.cursor_color().linear_multiply(0.35)
        }
    }

    fn hyperlink_color(&self) -> egui::Color32 {
        if let Some(c) = self.hyperlink {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            egui::Color32::from_rgba_unmultiplied(90, 170, 255, 255)
        }
    }

    fn comment_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.comment, [106, 153, 85, 255])
    }

    fn whitespace_symbols_color(&self) -> egui::Color32 {
        if let Some(c) = self.whitespace_symbols {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            self.comment_color().linear_multiply(0.4)
        }
    }

    fn success_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.success, [76, 175, 80, 255])
    }

    fn info_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.info, [33, 150, 243, 255])
    }

    fn warning_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.warning, [255, 152, 0, 255])
    }

    fn error_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.error, [244, 67, 54, 255])
    }

    fn heading_color(&self) -> egui::Color32 {
        if let Some(c) = self.heading_text {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            let fg = self.foreground.unwrap_or([255, 255, 255, 255]);
            self.to_egui_color32(fg)
        }
    }

    fn background_color(&self) -> egui::Color32 {
        self.to_color32_opt(self.background, [18, 18, 18, 255])
    }

    fn primary_color(&self) -> egui::Color32 {
        // Use icon hover color as a proxy for "primary" if not defined, otherwise info color
        self.to_color32_opt(self.icon_hover, [100, 150, 255, 255])
    }

    fn tree_line_color(&self, ui_visuals: &egui::Visuals) -> egui::Color32 {
        if let Some(c) = self.tree_line {
            egui::Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        } else {
            ui_visuals.weak_text_color()
        }
    }
}

// ─── Theme egui application ─────────────────────────────────────────────────

pub trait ThemeExt {
    fn apply(&self, ctx: &egui::Context);
}

impl ThemeExt for Theme {
    fn apply(&self, ctx: &egui::Context) {
        let mut visuals = if self.color_scheme == ColorScheme::Light {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };

        let mut colors = self.colors.clone();
        colors.resolve(self.color_scheme);

        let bg_color = colors.to_egui_color32(colors.background.unwrap_or([18, 18, 18, 255]));
        visuals.window_fill = bg_color;
        visuals.panel_fill = bg_color;
        visuals.extreme_bg_color = bg_color;

        if let Some(c) = colors.text_edit_bg {
            visuals.extreme_bg_color = colors.to_egui_color32(c);
        }

        visuals.selection.bg_fill = colors.selection_color();

        if let Some(c) = colors.selection_text {
            visuals.selection.stroke.color = colors.to_egui_color32(c);
        } else {
            visuals.selection.stroke.color = colors.cursor_color();
        }

        visuals.text_cursor.stroke.color = colors.cursor_color();

        let foreground = colors.to_egui_color32(colors.foreground.unwrap_or([255, 255, 255, 255]));
        visuals.widgets.noninteractive.fg_stroke.color = foreground;
        visuals.widgets.inactive.fg_stroke.color = foreground;
        visuals.widgets.hovered.fg_stroke.color = foreground;
        visuals.widgets.active.fg_stroke.color = foreground;

        visuals.override_text_color = None;

        if let Some(c) = colors.hyperlink {
            visuals.hyperlink_color = colors.to_egui_color32(c);
        }

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
            visuals.widgets.inactive.fg_stroke.color = fg_color;
            if colors.button_hover_fg.is_none() {
                visuals.widgets.hovered.fg_stroke.color = fg_color;
            }
            if colors.button_active_fg.is_none() {
                visuals.widgets.active.fg_stroke.color = fg_color;
            }
        }

        if let Some(sep) = colors.separator {
            let sep_color = colors.to_egui_color32(sep);
            visuals.widgets.noninteractive.bg_stroke.color = sep_color;
        }

        if let Some(c) = colors.widget_focus_border {
            visuals.selection.stroke = egui::Stroke::new(1.0, colors.to_egui_color32(c));
        }

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

        {
            let spread = colors.shadow_spread.unwrap_or(0.0);
            let blur = colors.shadow_blur.unwrap_or(0.0);
            if spread == 0.0 && blur == 0.0 {
                visuals.window_shadow.color = egui::Color32::TRANSPARENT;
                visuals.popup_shadow.color = egui::Color32::TRANSPARENT;
            }
        }

        let offset_x = colors.shadow_offset_x.unwrap_or(0.0) as i8;
        let offset_y = colors.shadow_offset_y.unwrap_or(0.0) as i8;
        visuals.window_shadow.offset = [offset_x, offset_y];
        visuals.popup_shadow.offset = [offset_x, offset_y];

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

        if let Some(r) = colors.widget_rounding {
            let radius = egui::CornerRadius::same(r as u8);
            style.visuals.widgets.noninteractive.corner_radius = radius;
            style.visuals.widgets.inactive.corner_radius = radius;
            style.visuals.widgets.hovered.corner_radius = radius;
            style.visuals.widgets.active.corner_radius = radius;
            style.visuals.widgets.open.corner_radius = radius;
        } else {
            style.visuals.widgets.noninteractive.corner_radius =
                default_style.visuals.widgets.noninteractive.corner_radius;
            style.visuals.widgets.inactive.corner_radius =
                default_style.visuals.widgets.inactive.corner_radius;
            style.visuals.widgets.hovered.corner_radius =
                default_style.visuals.widgets.hovered.corner_radius;
            style.visuals.widgets.active.corner_radius =
                default_style.visuals.widgets.active.corner_radius;
            style.visuals.widgets.open.corner_radius =
                default_style.visuals.widgets.open.corner_radius;
        }

        if let Some(w) = colors.widget_border_width {
            style.visuals.widgets.inactive.bg_stroke.width = w;
            style.visuals.widgets.hovered.bg_stroke.width = w;
            style.visuals.widgets.active.bg_stroke.width = w;
        } else {
            style.visuals.widgets.inactive.bg_stroke.width =
                default_style.visuals.widgets.inactive.bg_stroke.width;
            style.visuals.widgets.hovered.bg_stroke.width =
                default_style.visuals.widgets.hovered.bg_stroke.width;
            style.visuals.widgets.active.bg_stroke.width =
                default_style.visuals.widgets.active.bg_stroke.width;
        }

        if let Some(c) = colors.widget_border_color {
            let stroke_color = colors.to_egui_color32(c);
            style.visuals.widgets.inactive.bg_stroke.color = stroke_color;
        } else {
            let visuals_def = if self.color_scheme == ColorScheme::Light {
                egui::Visuals::light()
            } else {
                egui::Visuals::dark()
            };
            style.visuals.widgets.inactive.bg_stroke.color =
                visuals_def.widgets.inactive.bg_stroke.color;
        }

        if let Some(x) = colors.widget_padding_x {
            style.spacing.button_padding.x = x;
        } else {
            style.spacing.button_padding.x = default_style.spacing.button_padding.x;
        }
        if let Some(y) = colors.widget_padding_y {
            style.spacing.button_padding.y = y;
            style.spacing.interact_size.y = 0.0;
        } else {
            style.spacing.button_padding.y = default_style.spacing.button_padding.y;
            style.spacing.interact_size.y = default_style.spacing.interact_size.y;
        }

        if let Some(w) = colors.separator_width {
            style.visuals.widgets.noninteractive.bg_stroke.width = w;
        } else {
            style.visuals.widgets.noninteractive.bg_stroke.width =
                default_style.visuals.widgets.noninteractive.bg_stroke.width;
        }

        style.spacing.icon_width = if colors.widget_padding_y.is_some() {
            20.0
        } else {
            26.0
        };
        style.spacing.icon_spacing = if colors.widget_padding_y.is_some() {
            6.0
        } else {
            10.0
        };

        style.visuals.widgets.inactive.fg_stroke.width = 2.0;
        style.visuals.widgets.hovered.fg_stroke.width = 2.2;
        style.visuals.widgets.active.fg_stroke.width = 2.5;

        ctx.set_style(style);
    }
}
