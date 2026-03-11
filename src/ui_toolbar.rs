use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Render icon toolbar.
    ///
    /// Layout behaviour:
    ///  • Vertical   – file + keyfile groups at the **top** (scrollable), settings group
    ///                 pinned to the **bottom** with a flexible spacer between them.
    ///  • Horizontal – file + keyfile groups on the **left**, settings group pushed to
    ///                 the **right** via a right-to-left sub-layout.
    pub(crate) fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().button_padding = egui::vec2(2.0, 2.0);
        ui.spacing_mut().interact_size.y = 0.0;
        ui.spacing_mut().item_spacing = egui::vec2(4.0, 4.0);

        let is_vertical =
            self.settings.toolbar_position == crate::settings::ToolbarPosition::Left
                || self.settings.toolbar_position == crate::settings::ToolbarPosition::Right;

        if is_vertical {
            self.render_toolbar_vertical(ui);
        } else {
            self.render_toolbar_horizontal(ui);
        }
    }

    // ─── Layout orchestrators ────────────────────────────────────────────────────

    fn render_toolbar_vertical(&mut self, ui: &mut egui::Ui) {
        // Capture the panel height *before* entering the ScrollArea (inside a
        // ScrollArea, available_height() reports the virtual/infinite canvas size).
        let panel_h = ui.available_height();

        let ico_s = self.settings.toolbar_icon_size;
        let btn_h = ico_s + 4.0 + 4.0; // button height + item_spacing
        let sep_h = 10.0; // approximate separator height

        // Rough content-height estimate for all three groups + their separators.
        //   File group:     6 buttons
        //   Keyfile group:  4 buttons
        //   Settings group: separator + 2 buttons + separator + 2 buttons + 1 button (total 5)
        let file_h     = 6.0 * btn_h;
        let keyfile_h  = 4.0 * btn_h; // used same size as file_h
        let settings_h = 5.0 * btn_h + 2.0 * sep_h;
        let dividers_h = 2.0 * sep_h; // the two separators between groups
        let total_content_h = file_h + keyfile_h + settings_h + dividers_h;

        // The spacer fills whatever is left between groups 2 and 3.
        // When the window shrinks below total_content_h it collapses to a
        // small minimum gap and the ScrollArea scrolls normally.
        let min_gap = 16.0;
        let spacer = (panel_h - total_content_h).max(min_gap);

        egui::ScrollArea::vertical()
            .id_salt("tb_scroll")
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    self.render_toolbar_file_group(ui);
                    ui.separator();
                    self.render_toolbar_keyfile_group(ui);

                    // Flexible gap — shrinks to min_gap when space is tight.
                    ui.add_space(spacer);

                    self.render_toolbar_settings_group(ui);
                });
            });
    }

    fn render_toolbar_horizontal(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // ── Main groups (left-aligned) ────────────────────────────────────
            self.render_toolbar_file_group(ui);
            ui.separator();
            self.render_toolbar_keyfile_group(ui);

            // ── Settings group (right-aligned) ────────────────────────────────
            // `right_to_left` consumes all remaining width and renders items from
            // the right edge inwards, naturally pushing the group to the far right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                self.render_toolbar_settings_group_rtl(ui);
            });
        });
    }

    // ─── Shared icon-button primitive (no `self` borrow – avoids conflicts) ─────

    /// Allocate and draw one icon button, returning its `Response`.
    ///
    /// Being an associated function (no `self`) allows callers to hold an
    /// immutable borrow on e.g. `self.icons.foo` for the call duration, then
    /// immediately follow with a `&mut self` method call on the result.
    fn icon_btn(
        ui: &mut egui::Ui,
        icon: &egui::TextureHandle,
        tooltip: &str,
        selected: bool,
        btn_size: egui::Vec2,
        ico_size: egui::Vec2,
        hover_tint: egui::Color32,
        default_tint: egui::Color32,
    ) -> egui::Response {
        let (rect, mut response) = ui.allocate_exact_size(btn_size, egui::Sense::click());
        if response.clicked() {
            response.mark_changed();
        }

        if selected {
            ui.painter()
                .rect_filled(rect, 4.0, ui.visuals().widgets.active.bg_fill);
        } else if response.hovered() {
            ui.painter()
                .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
        }

        let tint = if response.hovered() || selected {
            hover_tint
        } else {
            default_tint
        };
        let icon_rect = egui::Rect::from_center_size(rect.center(), ico_size);
        ui.painter().image(
            icon.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );

        response.on_hover_text(tooltip)
    }

    // ─── Group renderers ─────────────────────────────────────────────────────────

    /// File operations: New, Open, Open Directory, Save, Save As, Export.
    fn render_toolbar_file_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.new_doc, "New (Ctrl+N)", false, bs, is, ht, dt).clicked() {
            self.new_document();
        }
        if Self::icon_btn(ui, &self.icons.open, "Open (Ctrl+O)", false, bs, is, ht, dt).clicked() {
            self.open_file_dialog();
        }
        if Self::icon_btn(ui, &self.icons.open_folder, "Open Directory", false, bs, is, ht, dt).clicked() {
            self.open_directory();
        }
        if Self::icon_btn(ui, &self.icons.save, "Save (Ctrl+S)", false, bs, is, ht, dt).clicked() {
            self.save_file();
        }
        if Self::icon_btn(ui, &self.icons.save_as, "Save As", false, bs, is, ht, dt).clicked() {
            self.save_file_as();
        }
        if Self::icon_btn(ui, &self.icons.export, "Export to Plaintext (.txt)", false, bs, is, ht, dt).clicked() {
            self.export_plaintext();
        }
    }

    /// Keyfile operations: Generate, Load, Rotate, Batch Convert.
    /// NOTE: the duplicate "Export as Plaintext" that was here has been removed.
    fn render_toolbar_keyfile_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.generate, "Generate Keyfile", false, bs, is, ht, dt).clicked() {
            self.generate_new_keyfile();
        }
        if Self::icon_btn(ui, &self.icons.key, "Load Keyfile", false, bs, is, ht, dt).clicked() {
            self.load_keyfile();
        }
        if Self::icon_btn(ui, &self.icons.rotate, "Rotate Keyfile", false, bs, is, ht, dt).clicked() {
            self.rotate_keyfile();
        }
        if Self::icon_btn(ui, &self.icons.batch_convert, "Batch Convert", false, bs, is, ht, dt).clicked() {
            self.show_batch_converter = !self.show_batch_converter;
        }
    }

    /// Panel-toggle group rendered in normal (top-down / left-to-right) order.
    /// Used by the **vertical** toolbar.
    fn render_toolbar_settings_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        ui.separator();

        if Self::icon_btn(ui, &self.icons.theme, "Toggle Theme Editor", self.show_theme_editor, bs, is, ht, dt).clicked() {
            self.show_theme_editor = !self.show_theme_editor;
            self.settings.show_theme_editor = self.show_theme_editor;
            let _ = self.settings.save();
            if self.show_theme_editor {
                self.editing_theme = Some(self.current_theme.clone());
            }
        }
        if Self::icon_btn(ui, &self.icons.settings, "Toggle Settings", self.show_settings_panel, bs, is, ht, dt).clicked() {
            self.show_settings_panel = !self.show_settings_panel;
            self.settings.show_settings_panel = self.show_settings_panel;
            let _ = self.settings.save();
        }

        ui.separator();

        if Self::icon_btn(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel, bs, is, ht, dt).clicked() {
            self.show_debug_panel = !self.show_debug_panel;
            self.settings.show_debug_panel = self.show_debug_panel;
            let _ = self.settings.save();
        }
        if Self::icon_btn(ui, &self.icons.history, "Toggle History", self.show_history_panel, bs, is, ht, dt).clicked() {
            self.show_history_panel = !self.show_history_panel;
            self.settings.show_history_panel = self.show_history_panel;
            let _ = self.settings.save();
        }
        if Self::icon_btn(ui, &self.icons.file_tree, "Toggle File Tree", self.show_file_tree, bs, is, ht, dt).clicked() {
            self.show_file_tree = !self.show_file_tree;
            self.settings.show_file_tree = self.show_file_tree;
            let _ = self.settings.save();
        }
    }

    /// Panel-toggle group rendered in **reverse** order for the right-to-left layout.
    /// Used by the **horizontal** toolbar.
    ///
    /// In RTL the first item rendered lands at the rightmost position.
    /// Desired visual order left→right:  | theme | settings || debug | history | file_tree |
    /// So the RTL render order must be:    file_tree, history, debug, sep, settings, theme, sep
    fn render_toolbar_settings_group_rtl(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.file_tree, "Toggle File Tree", self.show_file_tree, bs, is, ht, dt).clicked() {
            self.show_file_tree = !self.show_file_tree;
            self.settings.show_file_tree = self.show_file_tree;
            let _ = self.settings.save();
        }
        if Self::icon_btn(ui, &self.icons.history, "Toggle History", self.show_history_panel, bs, is, ht, dt).clicked() {
            self.show_history_panel = !self.show_history_panel;
            self.settings.show_history_panel = self.show_history_panel;
            let _ = self.settings.save();
        }
        if Self::icon_btn(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel, bs, is, ht, dt).clicked() {
            self.show_debug_panel = !self.show_debug_panel;
            self.settings.show_debug_panel = self.show_debug_panel;
            let _ = self.settings.save();
        }

        ui.separator();

        if Self::icon_btn(ui, &self.icons.settings, "Toggle Settings", self.show_settings_panel, bs, is, ht, dt).clicked() {
            self.show_settings_panel = !self.show_settings_panel;
            self.settings.show_settings_panel = self.show_settings_panel;
            let _ = self.settings.save();
        }
        if Self::icon_btn(ui, &self.icons.theme, "Toggle Theme Editor", self.show_theme_editor, bs, is, ht, dt).clicked() {
            self.show_theme_editor = !self.show_theme_editor;
            self.settings.show_theme_editor = self.show_theme_editor;
            let _ = self.settings.save();
            if self.show_theme_editor {
                self.editing_theme = Some(self.current_theme.clone());
            }
        }

        // Leftmost visual separator (rendered last in RTL = placed at the left edge)
        ui.separator();
    }
}