use crate::app_helpers::ScrollAreaExt;
use crate::theme::ThemeColorsExt;
use crate::EditorApp;
use eframe::egui;
use sen_core::theme_egui::UiSeparatorExt;

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

        // Group 1: 6 btns (new, open, open_folder, save, save_as, close)
        // Group 2: 1 btn (export)
        // Group 3: 3 btns (load, rotate, generate)
        // Group 4: 7 btns (history, file_tree, zen, theme, settings, batch_convert, debug)
        // Total: 17 buttons
        // Separators: 3
        // Gaps calculation (using item_spacing):
        // 17 items + 3 separators = 20 elements -> 19 gaps
        // + 1 additional gap for the spacer
        let total_content_h = (17.0 * btn_h) + (3.0 * sep_h) + (20.0 * spacing);
        let min_gap = 16.0;
        let spacer = (panel_h - total_content_h - 8.0).max(min_gap);

        egui::ScrollArea::vertical()
            .id_salt("tb_scroll")
            .show_themed(self.current_theme.colors.clone(), ui, |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.add_enabled_ui(!self.show_batch_converter && !self.zen_mode, |ui| {
                        self.render_toolbar_file_group(ui);
                        ui.app_separator();
                        self.render_toolbar_batch_group(ui);
                        ui.app_separator();
                        self.render_toolbar_key_ops_group(ui);
                        ui.app_separator();
                    });

                    ui.add_space(spacer);

                    self.render_toolbar_settings_group(ui);
                });
            });
    }

    fn render_toolbar_horizontal(&mut self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), 0.0),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                // Group 1-3: Handled with global disable
                ui.add_enabled_ui(!self.show_batch_converter && !self.zen_mode, |ui| {
                    // Group 1: Files
                    self.render_toolbar_file_group(ui);
                    ui.app_separator();

                    // Group 2: Batch/Export
                    self.render_toolbar_batch_group(ui);
                    ui.app_separator();

                    // Group 3: Keyfile management
                    self.render_toolbar_key_ops_group(ui);
                    ui.app_separator();
                });

                // ── Settings group (right-aligned) ────────────────────────────────
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    self.render_toolbar_settings_group_rtl(ui);
                });
            },
        );
    }

    // ─── Shared icon-button primitive (no `self` borrow – avoids conflicts) ─────

    /// Allocate and draw one icon button, returning its `Response`.
    ///
    /// Being an associated function (no `self`) allows callers to hold an
    /// immutable borrow on e.g. `self.icons.foo` for the call duration, then
    /// immediately follow with a `&mut self` method call on the result.
    #[allow(clippy::too_many_arguments)]
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

        let is_enabled = ui.is_enabled();

        if is_enabled && response.clicked() {
            response.mark_changed();
        }

        if is_enabled {
            if selected {
                ui.painter()
                    .rect_filled(rect, 4.0, ui.visuals().widgets.active.bg_fill);
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
            }
        }

        let tint = if !is_enabled {
            ui.visuals().widgets.noninteractive.fg_stroke.color
        } else if response.hovered() || selected {
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

    /// Group 1: New, Open, Open Directory, Save, Save As, Close.
    fn render_toolbar_file_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(
            ui,
            &self.icons.new_doc,
            &t!("toolbar.new"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.new_document();
        }

        if Self::icon_btn(
            ui,
            &self.icons.save,
            &t!("toolbar.save"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.save_file();
        }

        if Self::icon_btn(
            ui,
            &self.icons.save_as,
            &t!("toolbar.save_as"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.save_file_as();
        }

        if Self::icon_btn(
            ui,
            &self.icons.open,
            &t!("toolbar.open"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.open_file_dialog();
        }

        if Self::icon_btn(
            ui,
            &self.icons.open_folder,
            &t!("toolbar.open_dir"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.open_directory();
        }

        if Self::icon_btn(
            ui,
            &self.icons.close,
            &t!("toolbar.close"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.close_file();
        }
    }

    /// Group 2: Export tasks.
    fn render_toolbar_batch_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        if Self::icon_btn(
            ui,
            &self.icons.export,
            &t!("toolbar.export"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.export_plaintext();
        }
    }

    /// Group 3: Load, Rotate, Generate.
    fn render_toolbar_key_ops_group(&mut self, ui: &mut egui::Ui) {
        let ico_s = self.settings.toolbar_icon_size;
        let bs = egui::vec2(ico_s + 4.0, ico_s + 4.0);
        let is = egui::vec2(ico_s, ico_s);
        let ht = self.current_theme.colors.icon_hover_color();
        let dt = self.current_theme.colors.icon_color();

        let mut load_key_tint = dt;
        if self.keyfile_path.is_none() {
            let pulse_alpha =
                0.1 + 0.9 * (self.start_time.elapsed().as_secs_f32() * 3.0).cos().abs();
            load_key_tint = self
                .current_theme
                .colors
                .warning_color()
                .gamma_multiply(pulse_alpha);
            ui.ctx().request_repaint(); // Animation needs repaint
        }

        if Self::icon_btn(
            ui,
            &self.icons.key,
            &t!("toolbar.load_keyfile"),
            false,
            bs,
            is,
            ht,
            load_key_tint,
        )
        .clicked()
        {
            self.load_keyfile();
        }
        if Self::icon_btn(
            ui,
            &self.icons.rotate,
            &t!("toolbar.rotate_keyfile"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
            self.rotate_keyfile();
        }
        if Self::icon_btn(
            ui,
            &self.icons.generate,
            &t!("toolbar.generate_keyfile"),
            false,
            bs,
            is,
            ht,
            dt,
        )
        .clicked()
        {
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

        let global_enabled = !self.show_batch_converter && !self.zen_mode;
        let zen_toggle_enabled = !self.show_batch_converter;

        ui.add_enabled_ui(global_enabled, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.history,
                &t!("toolbar.toggle_history"),
                self.show_history_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_history_panel = !self.show_history_panel;
                self.settings.show_history_panel = self.show_history_panel;
            }
            if Self::icon_btn(
                ui,
                &self.icons.file_tree,
                &t!("toolbar.toggle_file_tree"),
                self.show_file_tree,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_file_tree = !self.show_file_tree;
                self.settings.show_file_tree = self.show_file_tree;
            }
        });

        ui.add_enabled_ui(zen_toggle_enabled, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.zen,
                &t!("toolbar.toggle_zen"),
                self.zen_mode,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.toggle_zen_mode(ui.ctx());
            }
        });

        ui.add_enabled_ui(global_enabled, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.theme,
                &t!("toolbar.toggle_theme"),
                self.show_theme_editor,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_theme_editor = !self.show_theme_editor;
                self.settings.show_theme_editor = self.show_theme_editor;
                self.show_delete_theme_confirmation = false;
                if self.show_theme_editor {
                    self.editing_theme = Some(self.current_theme.clone());
                }
            }
            if Self::icon_btn(
                ui,
                &self.icons.settings,
                &t!("toolbar.toggle_settings"),
                self.show_settings_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_settings_panel = !self.show_settings_panel;
                self.settings.show_settings_panel = self.show_settings_panel;
            }
        });

        // Batch converter (always clickable to exit, but disabled in zen mode)
        ui.add_enabled_ui(!self.zen_mode, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.batch_convert,
                &t!("toolbar.toggle_batch"),
                self.show_batch_converter,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_batch_converter = !self.show_batch_converter;
            }
        });

        ui.add_enabled_ui(global_enabled, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.debug,
                &t!("toolbar.toggle_debug"),
                self.show_debug_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_debug_panel = !self.show_debug_panel;
                self.settings.show_debug_panel = self.show_debug_panel;
            }
        });
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

        let global_enabled = !self.show_batch_converter && !self.zen_mode;
        let zen_toggle_enabled = !self.show_batch_converter;

        // 7. Batch Converter (always clickable, but disabled in zen mode)
        ui.add_enabled_ui(!self.zen_mode, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.batch_convert,
                &t!("toolbar.toggle_batch"),
                self.show_batch_converter,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_batch_converter = !self.show_batch_converter;
            }
        });

        ui.add_enabled_ui(global_enabled, |ui| {
            // 6. Debug
            if Self::icon_btn(
                ui,
                &self.icons.debug,
                &t!("toolbar.toggle_debug"),
                self.show_debug_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_debug_panel = !self.show_debug_panel;
                self.settings.show_debug_panel = self.show_debug_panel;
            }
            // 5. Settings
            if Self::icon_btn(
                ui,
                &self.icons.settings,
                &t!("toolbar.toggle_settings"),
                self.show_settings_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_settings_panel = !self.show_settings_panel;
                self.settings.show_settings_panel = self.show_settings_panel;
            }
            // 4. Theme Editor
            if Self::icon_btn(
                ui,
                &self.icons.theme,
                &t!("toolbar.toggle_theme"),
                self.show_theme_editor,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_theme_editor = !self.show_theme_editor;
                self.settings.show_theme_editor = self.show_theme_editor;
                self.show_delete_theme_confirmation = false;
                if self.show_theme_editor {
                    self.editing_theme = Some(self.current_theme.clone());
                }
            }
        });

        // 3. Zen Mode (toggle enabled even in zen)
        ui.add_enabled_ui(zen_toggle_enabled, |ui| {
            if Self::icon_btn(
                ui,
                &self.icons.zen,
                &t!("toolbar.toggle_zen"),
                self.zen_mode,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.toggle_zen_mode(ui.ctx());
            }
        });

        ui.add_enabled_ui(global_enabled, |ui| {
            // 2. File Tree
            if Self::icon_btn(
                ui,
                &self.icons.file_tree,
                &t!("toolbar.toggle_file_tree"),
                self.show_file_tree,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_file_tree = !self.show_file_tree;
                self.settings.show_file_tree = self.show_file_tree;
            }
            // 1. History
            if Self::icon_btn(
                ui,
                &self.icons.history,
                &t!("toolbar.toggle_history"),
                self.show_history_panel,
                bs,
                is,
                ht,
                dt,
            )
            .clicked()
            {
                self.show_history_panel = !self.show_history_panel;
                self.settings.show_history_panel = self.show_history_panel;
            }
        });
    }
}
