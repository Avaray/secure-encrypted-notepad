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

        let is_vertical = self.settings.toolbar_position == crate::settings::ToolbarPosition::Left
            || self.settings.toolbar_position == crate::settings::ToolbarPosition::Right;

        if is_vertical {
            self.render_toolbar_vertical(ui);
        } else {
            self.render_toolbar_horizontal(ui);
        }
    }

    // ─── Layout orchestrators ────────────────────────────────────────────────────

    fn render_toolbar_vertical(&mut self, ui: &mut egui::Ui) {
        // Capture the panel height *before* entering the ScrollArea
        let panel_h = ui.available_height();

        let ico_s = self.settings.toolbar_icon_size;
        let btn_h = ico_s + 4.0;
        let spacing = 4.0;
        let sep_h = 4.0;

        // Group 1: 5 btns (new, open, open_folder, save, save_as)
        // Group 2: 2 btns
        // Group 3: 3 btns (load, rotate, generate)
        // Group 4: 6 btns
        // Total: 16 buttons
        // Separators: 3
        // Padding/Spacing:
        //   5 (in G1) + 1 (sep) + 2 (in G2) + 1 (sep) + 3 (in G3) + 1 (sep) + 1 (spacer) + 6 (in G4) = 20 gaps
        let total_content_h = (16.0 * btn_h) + (3.0 * sep_h) + (20.0 * spacing);
        let min_gap = 16.0;
        let spacer = (panel_h - total_content_h - 1.0).max(min_gap);

        egui::ScrollArea::vertical()
            .id_salt("tb_scroll")
            .show(ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    self.render_toolbar_file_group(ui);
                    ui.separator();
                    self.render_toolbar_batch_group(ui);
                    ui.separator();
                    self.render_toolbar_key_ops_group(ui);
                    ui.separator();

                    ui.add_space(spacer);

                    self.render_toolbar_settings_group(ui);
                });
            });
    }

    fn render_toolbar_horizontal(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Group 1: Files
            self.render_toolbar_file_group(ui);
            ui.separator();
            
            // Group 2: Batch/Export
            self.render_toolbar_batch_group(ui);
            ui.separator();

            // Group 3: Keyfile management
            self.render_toolbar_key_ops_group(ui);
            ui.separator();

            // ── Settings group (right-aligned) ────────────────────────────────
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

    /// Group 1: New, Open, Open Directory, Save As.
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
        // User requested Save As to be in Group 1
        if Self::icon_btn(ui, &self.icons.save_as, "Save As", false, bs, is, ht, dt).clicked() {
            self.save_file_as();
        }
    }

    /// Group 2: Export to Plaintext, Batch Convert.
    fn render_toolbar_batch_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.export, "Export to Plaintext (.txt)", false, bs, is, ht, dt).clicked() {
            self.export_plaintext();
        }
        if Self::icon_btn(ui, &self.icons.batch_convert, "Batch Convert", false, bs, is, ht, dt).clicked() {
            self.show_batch_converter = !self.show_batch_converter;
        }
    }

    /// Group 3: Load, Rotate, Generate.
    fn render_toolbar_key_ops_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.key, "Load Keyfile", false, bs, is, ht, dt).clicked() {
            self.load_keyfile();
        }
        if Self::icon_btn(ui, &self.icons.rotate, "Rotate Keyfile", false, bs, is, ht, dt).clicked() {
            self.rotate_keyfile();
        }
        if Self::icon_btn(ui, &self.icons.generate, "Generate Keyfile", false, bs, is, ht, dt).clicked() {
            self.generate_new_keyfile();
        }
    }

    /// Panel-toggle group rendered in normal (top-down / left-to-right) order.
    /// Used by the **vertical** toolbar.
    /// Group 4: History, File Tree, Zen, Theme Editor, Settings, Debug.
    fn render_toolbar_settings_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(ui, &self.icons.history, "Toggle History", self.show_history_panel, bs, is, ht, dt).clicked() {
            self.show_history_panel = !self.show_history_panel;
            self.settings.show_history_panel = self.show_history_panel;
        }
        if Self::icon_btn(ui, &self.icons.file_tree, "Toggle File Tree", self.show_file_tree, bs, is, ht, dt).clicked() {
            self.show_file_tree = !self.show_file_tree;
            self.settings.show_file_tree = self.show_file_tree;
        }
        if Self::icon_btn(ui, &self.icons.zen, "Toggle Zen Mode (F11)", self.zen_mode, bs, is, ht, dt).clicked() {
            self.toggle_zen_mode(ui.ctx());
        }
        if Self::icon_btn(ui, &self.icons.theme, "Toggle Theme Editor", self.show_theme_editor, bs, is, ht, dt).clicked() {
            self.show_theme_editor = !self.show_theme_editor;
            self.settings.show_theme_editor = self.show_theme_editor;
            self.show_delete_theme_confirmation = false;
            if self.show_theme_editor {
                self.editing_theme = Some(self.current_theme.clone());
            }
        }
        if Self::icon_btn(ui, &self.icons.settings, "Toggle Settings", self.show_settings_panel, bs, is, ht, dt).clicked() {
            self.show_settings_panel = !self.show_settings_panel;
            self.settings.show_settings_panel = self.show_settings_panel;
        }
        if Self::icon_btn(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel, bs, is, ht, dt).clicked() {
            self.show_debug_panel = !self.show_debug_panel;
            self.settings.show_debug_panel = self.show_debug_panel;
        }
    }

    /// Panel-toggle group rendered in **reverse** order for the right-to-left layout.
    /// Used by the **horizontal** toolbar.
    ///
    /// In RTL the first item rendered lands at the rightmost position.
    /// Desired visual order left→right:  | theme | settings || debug | history | file_tree |
    /// So the RTL render order must be:    file_tree, history, debug, sep, settings, theme, sep
    /// Group 4 (RTL): Debug, Settings, Theme Editor, Zen, File Tree, History.
    /// In RTL, the last item in code becomes the leftmost in UI (within the right-aligned block).
    /// Desired visual order: History, File Tree, Zen, Theme Editor, Settings, Debug.
    fn render_toolbar_settings_group_rtl(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        // 6. Debug
        if Self::icon_btn(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel, bs, is, ht, dt).clicked() {
            self.show_debug_panel = !self.show_debug_panel;
            self.settings.show_debug_panel = self.show_debug_panel;
        }
        // 5. Settings
        if Self::icon_btn(ui, &self.icons.settings, "Toggle Settings", self.show_settings_panel, bs, is, ht, dt).clicked() {
            self.show_settings_panel = !self.show_settings_panel;
            self.settings.show_settings_panel = self.show_settings_panel;
        }
        // 4. Theme Editor
        if Self::icon_btn(ui, &self.icons.theme, "Toggle Theme Editor", self.show_theme_editor, bs, is, ht, dt).clicked() {
            self.show_theme_editor = !self.show_theme_editor;
            self.settings.show_theme_editor = self.show_theme_editor;
            self.show_delete_theme_confirmation = false;
            if self.show_theme_editor {
                self.editing_theme = Some(self.current_theme.clone());
            }
        }
        // 3. Zen Mode
        if Self::icon_btn(ui, &self.icons.zen, "Toggle Zen Mode (F11)", self.zen_mode, bs, is, ht, dt).clicked() {
            self.toggle_zen_mode(ui.ctx());
        }
        // 2. File Tree
        if Self::icon_btn(ui, &self.icons.file_tree, "Toggle File Tree", self.show_file_tree, bs, is, ht, dt).clicked() {
            self.show_file_tree = !self.show_file_tree;
            self.settings.show_file_tree = self.show_file_tree;
        }
        // 1. History
        if Self::icon_btn(ui, &self.icons.history, "Toggle History", self.show_history_panel, bs, is, ht, dt).clicked() {
            self.show_history_panel = !self.show_history_panel;
            self.settings.show_history_panel = self.show_history_panel;
        }
    }
}
