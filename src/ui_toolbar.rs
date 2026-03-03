use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Render icon toolbar with hover colors
    pub(crate) fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        let is_vertical = self.settings.toolbar_position == crate::settings::ToolbarPosition::Left
            || self.settings.toolbar_position == crate::settings::ToolbarPosition::Right;

        if is_vertical {
            ui.vertical(|ui| self.render_toolbar_content(ui, is_vertical));
        } else {
            ui.horizontal(|ui| self.render_toolbar_content(ui, is_vertical));
        }
    }

    fn render_toolbar_content(&mut self, ui: &mut egui::Ui, is_vertical: bool) {
        if is_vertical {
            ui.spacing_mut().item_spacing.y = 4.0;
        } else {
            ui.spacing_mut().item_spacing.x = 4.0;
        }
        // Icon sizes based on setting directly
        let ico_s = self.settings.toolbar_icon_size;
        let btn_s = ico_s + 8.0; // padding around the icon
        let button_size = egui::vec2(btn_s, btn_s);
        let icon_size = egui::vec2(ico_s, ico_s);
        let hover_tint = self.current_theme.colors.icon_hover_color();
        let default_tint = self.current_theme.colors.icon_color();

        // Helper closure to render icon button with hover effect
        let icon_button = |ui: &mut egui::Ui,
                           icon: &egui::TextureHandle,
                           tooltip: &str,
                           selected: bool|
         -> egui::Response {
            let (rect, mut response) =
                ui.allocate_exact_size(button_size, egui::Sense::click());
            if response.clicked() {
                response.mark_changed();
            }

            // Draw background if selected
            if selected {
                ui.painter()
                    .rect_filled(rect, 4.0, ui.visuals().widgets.active.bg_fill);
            } else if response.hovered() {
                ui.painter()
                    .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
            }

            // Draw icon with tint on hover
            let icon_rect = egui::Rect::from_center_size(rect.center(), icon_size);
            let tint = if response.hovered() || selected {
                hover_tint
            } else {
                default_tint
            };
            ui.painter().image(
                icon.id(),
                icon_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
            response.on_hover_text(tooltip)
        };

        // --- LEFT SIDE: Main Actions ---
        if icon_button(ui, &self.icons.new_doc, "New (Ctrl+N)", false).clicked() {
            self.new_document();
        }

        if icon_button(ui, &self.icons.open, "Open (Ctrl+O)", false).clicked() {
            self.open_file_dialog();
        }

        if icon_button(ui, &self.icons.open_folder, "Open Directory", false).clicked() {
            self.open_directory();
        }

        if icon_button(ui, &self.icons.save, "Save (Ctrl+S)", false).clicked() {
            self.save_file();
        }

        if icon_button(ui, &self.icons.save_as, "Save As", false).clicked() {
            self.save_file_as();
        }

        if icon_button(ui, &self.icons.export, "Export to Plaintext (.txt)", false).clicked() {
            self.export_plaintext();
        }

        ui.separator();

        // Small icon buttons for keyfile operations
        let sm_ico_s = self.settings.toolbar_icon_size * 0.8; // slightly smaller
        let sm_btn_s = sm_ico_s + 4.0;
        let small_icon_size = egui::vec2(sm_ico_s, sm_ico_s);
        let small_button_size = egui::vec2(sm_btn_s, sm_btn_s);

        let small_icon_btn =
            |ui: &mut egui::Ui, icon: &egui::TextureHandle, tooltip: &str| -> egui::Response {
                let (rect, mut response) =
                    ui.allocate_exact_size(small_button_size, egui::Sense::click());
                if response.clicked() {
                    response.mark_changed();
                }

                if response.hovered() {
                    ui.painter()
                        .rect_filled(rect, 4.0, ui.visuals().widgets.hovered.bg_fill);
                }

                let icon_rect = egui::Rect::from_center_size(rect.center(), small_icon_size);
                let tint = if response.hovered() {
                    hover_tint
                } else {
                    default_tint
                };
                ui.painter().image(
                    icon.id(),
                    icon_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );
                response.on_hover_text(tooltip)
            };

        if small_icon_btn(ui, &self.icons.generate, "Generate Keyfile").clicked() {
            self.generate_new_keyfile();
        }

        if small_icon_btn(ui, &self.icons.key, "Load Keyfile").clicked() {
            self.load_keyfile();
        }

        if small_icon_btn(ui, &self.icons.rotate, "Rotate Keyfile").clicked() {
            self.rotate_keyfile();
        }

        if small_icon_btn(ui, &self.icons.export, "Export as Plaintext").clicked() {
            self.export_plaintext();
        }

        if small_icon_btn(ui, &self.icons.batch_convert, "Batch Convert to SEN").clicked() {
            self.show_batch_converter = !self.show_batch_converter;
        }

        ui.separator();
        if is_vertical {
            ui.add_space(10.0);
        } else {
            ui.add_space(20.0);
        }

        // --- RIGHT SIDE: Toggles ---
        let layout = if is_vertical {
            egui::Layout::bottom_up(egui::Align::Center)
        } else {
            egui::Layout::right_to_left(egui::Align::Center)
        };
        
        ui.with_layout(layout, |ui| {
            // Theme Editor toggle
            if icon_button(
                ui,
                &self.icons.theme,
                "Toggle Theme Editor",
                self.show_theme_editor,
            )
            .clicked()
            {
                self.show_theme_editor = !self.show_theme_editor;
                // Jeśli otwieramy, ustaw editing_theme
                if self.show_theme_editor {
                    self.editing_theme = Some(self.current_theme.clone());
                }
            }

            // Settings toggle
            if icon_button(
                ui,
                &self.icons.settings,
                "Toggle Settings",
                self.show_settings_panel,
            )
            .clicked()
            {
                self.show_settings_panel = !self.show_settings_panel;
            }

            ui.separator();

            if icon_button(ui, &self.icons.debug, "Toggle Debug", self.show_debug_panel)
                .clicked()
            {
                self.show_debug_panel = !self.show_debug_panel;
                self.settings.show_debug_panel = self.show_debug_panel;
                let _ = self.settings.save();
            }

            if icon_button(
                ui,
                &self.icons.history,
                "Toggle History",
                self.show_history_panel,
            )
            .clicked()
            {
                self.show_history_panel = !self.show_history_panel;
            }

            if icon_button(
                ui,
                &self.icons.file_tree,
                "Toggle File Tree",
                self.show_file_tree,
            )
            .clicked()
            {
                self.show_file_tree = !self.show_file_tree;
                self.settings.show_file_tree = self.show_file_tree;
                let _ = self.settings.save();
            }
        });
    }
}
