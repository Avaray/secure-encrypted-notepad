use crate::app_state::PendingAction;
use crate::EditorApp;
use eframe::egui;
use rust_i18n::t;

impl EditorApp {
    /// Render confirmation dialog for unsaved changes
    pub(crate) fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        // Unsaved changes dialog
        if self.show_close_confirmation {
            egui::Window::new(t!("dialog.unsaved_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let text1 = t!("dialog.unsaved_line1");
                    let text2 = t!("dialog.unsaved_line2");

                    let w1 = ui
                        .painter()
                        .layout_no_wrap(text1.to_string(), font_id.clone(), egui::Color32::WHITE)
                        .rect
                        .width();
                    let w2 = ui
                        .painter()
                        .layout_no_wrap(text2.to_string(), font_id, egui::Color32::WHITE)
                        .rect
                        .width();

                    ui.set_min_width(w1.max(w2));

                    ui.label(text1);
                    ui.label(text2);
                    ui.add_space(8.0);

                    crate::app_helpers::center_row(ui, |ui| {
                        if ui.button(t!("dialog.btn_save")).clicked() {
                            self.save_file();
                            if !self.is_modified {
                                self.show_close_confirmation = false;
                                let action = self.pending_action.clone();
                                self.pending_action = PendingAction::None;
                                if let PendingAction::Exit = action {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                } else {
                                    self.execute_pending_action(action);
                                }
                            }
                        }

                        if ui.button(t!("dialog.btn_dont_save")).clicked() {
                            self.is_modified = false;
                            self.show_close_confirmation = false;
                            let action = self.pending_action.clone();
                            self.pending_action = PendingAction::None;
                            if let PendingAction::Exit = action {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            } else {
                                self.execute_pending_action(action);
                            }
                        } else {
                            // Save failed or cancelled
                            self.status_message = t!("dialog.save_failed").to_string();
                        }

                        if ui.button(t!("dialog.btn_cancel")).clicked() {
                            self.show_close_confirmation = false;
                            self.pending_action = PendingAction::None;
                        }
                    });
                });
        }

        // Settings reset dialog
        if self.show_reset_confirmation {
            egui::Window::new(t!("dialog.reset_title"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_max_width(400.0);
                    ui.add(
                        egui::Label::new(t!("dialog.reset_msg"))
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    ui.add(
                        egui::Label::new(t!("dialog.reset_undone"))
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    ui.add_space(8.0);

                    crate::app_helpers::center_row(ui, |ui| {
                        ui.label(t!("dialog.reset_slider"));
                        ui.spacing_mut().slider_width = ui.available_width();
                        ui.add(
                            egui::Slider::new(&mut self.reset_slider_val, 0.0..=1.0)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                    });
                    ui.add_space(8.0);

                    let is_confirmed = self.reset_slider_val >= 0.99;
                    crate::app_helpers::center_row(ui, |ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_enabled_ui(is_confirmed, |ui| {
                                if ui.button("OK").clicked() {
                                    let was_maximized = self.settings.start_maximized;
                                    self.settings = crate::settings::Settings::default();
                                    self.settings.start_maximized = was_maximized;
                                    let _ = self.settings.save();
                                    self.show_reset_confirmation = false;
                                    self.style_dirty = true; // Apply default fonts/sizes
                                    self.status_message = t!("dialog.reset_success").to_string();
                                    self.log_warning(t!("dialog.reset_success"));
                                }
                            });

                            if ui.button(t!("dialog.btn_cancel")).clicked() {
                                self.show_reset_confirmation = false;
                            }
                        });
                    });
                });
        }
    }

    /// Render Go to Line dialog
    pub(crate) fn render_goto_line_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_goto_line {
            return;
        }

        let mut close = false;
        let mut jump_to_line: Option<usize> = None;

        egui::Window::new(t!("dialog.goto_title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                crate::app_helpers::center_row(ui, |ui| {
                    ui.label(t!("dialog.goto_label"));
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.goto_line_input)
                            .desired_width(100.0)
                            .margin(ui.spacing().button_padding),
                    );

                    // Auto-focus on open
                    response.request_focus();

                    // Check for Enter key
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                            jump_to_line = Some(line_num);
                        }
                    }
                });

                crate::app_helpers::center_row(ui, |ui| {
                    if ui.button(t!("dialog.goto_btn")).clicked() {
                        if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                            jump_to_line = Some(line_num);
                        } else {
                            self.status_message = t!("dialog.goto_invalid").to_string();
                        }
                    }

                    if ui.button(t!("dialog.btn_cancel")).clicked() {
                        close = true;
                    }
                });
            });

        // Handle jump outside the window closure
        if let Some(line_num) = jump_to_line {
            let max_line = self.document.current_content.lines().count().max(1);
            if line_num > 0 && line_num <= max_line {
                self.highlighted_line = Some(line_num);
                self.log_info(t!("dialog.goto_success", line = line_num));
                close = true;
            } else {
                self.status_message = t!("dialog.goto_range_err", max = max_line).to_string();
                self.log_warning(t!("dialog.goto_range_err", max = max_line));
            }
        }

        if close {
            self.show_goto_line = false;
            self.goto_line_input.clear();
        }
    }

    /// Render auto-save restore dialog
    pub(crate) fn render_autosave_restore_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_autosave_restore {
            return;
        }

        let timestamp_str = self
            .document
            .autosave
            .as_ref()
            .map(|a| a.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        egui::Window::new(t!("dialog.autosave_title"))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_max_width(350.0);
                ui.label(t!("dialog.autosave_msg", time = timestamp_str));
                ui.separator();

                crate::app_helpers::center_row(ui, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t!("dialog.btn_restore")).clicked() {
                            if let Some(autosave) = self.document.autosave.take() {
                                self.document.current_content = autosave.content;
                                self.is_modified = true;
                                if self.show_search_panel {
                                    self.perform_search();
                                }
                                self.replace_undo_stack.clear();
                                self.log_info(t!("dialog.autosave_restored_log"));
                                self.status_message =
                                    t!("dialog.autosave_restored_msg").to_string();
                            }
                            self.show_autosave_restore = false;
                        }
                        if ui.button(t!("dialog.btn_discard")).clicked() {
                            self.document.clear_autosave();
                            self.log_info(t!("dialog.autosave_discarded_log"));
                            self.show_autosave_restore = false;
                        }
                    });
                });
            });
    }

    /// Helper to render a sponsor link with icon and text as a button
    fn sponsor_link(ui: &mut egui::Ui, icon: &egui::TextureHandle, label: &str, url: &str) {
        let icon_size = egui::vec2(40.0, 40.0);
        let font_id = egui::FontId::proportional(24.0);
        // Better way to get precise text width:
        let text_layout = ui.painter().layout_no_wrap(
            label.to_string(),
            font_id.clone(),
            ui.visuals().widgets.noninteractive.fg_stroke.color,
        );
        let text_width = text_layout.rect.width();

        let padding = 16.0;
        let spacing = 12.0;
        let btn_width = padding * 2.0 + icon_size.x + spacing + text_width;
        let btn_size = egui::vec2(btn_width, 64.0);

        let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());

        if response.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(url));
        }

        // Draw background
        let bg_fill = if response.clicked() {
            ui.visuals().widgets.active.bg_fill
        } else if response.hovered() {
            ui.visuals().widgets.hovered.bg_fill
        } else {
            ui.visuals()
                .widgets
                .noninteractive
                .bg_fill
                .gamma_multiply(0.5)
        };

        ui.painter().rect_filled(rect, 4.0, bg_fill);

        // Draw icon and text (centered as a unit)
        let icon_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + padding + icon_size.x / 2.0, rect.center().y),
            icon_size,
        );

        let tint = if response.hovered() {
            ui.visuals().widgets.hovered.fg_stroke.color
        } else {
            ui.visuals().widgets.noninteractive.fg_stroke.color
        };

        ui.painter().image(
            icon.id(),
            icon_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            tint,
        );

        let text_pos = egui::pos2(icon_rect.right() + spacing, rect.center().y);
        ui.painter()
            .text(text_pos, egui::Align2::LEFT_CENTER, label, font_id, tint);

        response.on_hover_cursor(egui::CursorIcon::PointingHand);
    }

    /// Render full-screen About panel
    pub(crate) fn render_about_panel(&mut self, ctx: &egui::Context) {
        if !self.show_about_panel {
            return;
        }

        // Create a full-screen overlay area in the foreground
        egui::Area::new(egui::Id::new("about_panel_area"))
            .order(egui::Order::Foreground)
            .anchor(egui::Align2::LEFT_TOP, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                #[allow(deprecated)]
                let rect = ctx.screen_rect();
                #[allow(deprecated)]
                ui.allocate_ui_at_rect(rect, |ui| {
                    // Fill background using the current window fill color
                    ui.painter()
                        .rect_filled(rect, 0.0, ui.visuals().window_fill());

                    // Center the inner content vertically and horizontally
                    ui.vertical_centered(|ui| {
                        ui.add_space(rect.height() * 0.15); // Dynamic top margin

                        ui.add(egui::Label::new(
                            egui::RichText::new(t!("dialog.about_title"))
                                .size(36.0)
                                .strong(),
                        ));
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(t!(
                                "dialog.about_version",
                                version = env!("CARGO_PKG_VERSION")
                            ))
                            .color(self.current_theme.colors.info_color()),
                        );

                        ui.add_space(40.0);

                        // Author Info
                        ui.heading(t!("dialog.about_author_head"));
                        ui.add_space(5.0);
                        ui.label(t!("dialog.about_author_body"));
                        ui.add_space(20.0);

                        // Links Section
                        ui.heading(t!("dialog.about_links_head"));
                        ui.add_space(10.0);

                        crate::app_helpers::center_row(ui, |ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.hyperlink_to(
                                    t!("dialog.about_github"),
                                    "https://github.com/Avaray/secure-encrypted-notepad",
                                );
                                ui.add_space(5.0);
                                ui.hyperlink_to(
                                    t!("dialog.about_bug"),
                                    "https://github.com/Avaray/secure-encrypted-notepad/issues",
                                );
                            });
                        });

                        ui.add_space(20.0);

                        // Financial Support
                        ui.heading(t!("dialog.about_support_head"));
                        ui.add_space(5.0);
                        ui.label(t!("dialog.about_support_body"));
                        ui.add_space(10.0);

                        // List of buttons to draw
                        let sponsors = [
                            (
                                &self.icons.spon_github,
                                "GitHub Sponsors",
                                "https://github.com/sponsors/Avaray",
                            ),
                            (
                                &self.icons.spon_patreon,
                                "Patreon",
                                "https://patreon.com/Avaray_",
                            ),
                            (
                                &self.icons.spon_bmc,
                                "Buy Me a Coffee",
                                "https://buymeacoffee.com/avaray",
                            ),
                            (
                                &self.icons.spon_oc,
                                "Open Collective",
                                "https://opencollective.com/avaray",
                            ),
                            (&self.icons.spon_kofi, "Ko-fi", "https://ko-fi.com/avaray_"),
                        ];

                        // Exact total width calculation for a single row
                        let mut total_buttons_width = 0.0;
                        let item_spacing = 10.0;
                        let font_id = egui::FontId::proportional(24.0);

                        for (_, label, _) in &sponsors {
                            let text_width = ui
                                .painter()
                                .layout_no_wrap(
                                    label.to_string(),
                                    font_id.clone(),
                                    egui::Color32::WHITE,
                                )
                                .rect
                                .width();
                            let btn_width = 16.0 * 2.0 + 40.0 + 12.0 + text_width; // padding*2 + icon + spacing + text
                            total_buttons_width += btn_width + item_spacing;
                        }
                        if total_buttons_width > 0.0 {
                            total_buttons_width -= item_spacing; // Remove trailing spacing
                        }

                        // We constrain the wrapping block to either exactly fit all buttons in 1 row,
                        // or max 90% of screen width if it's too large, forcing a wrap.
                        let block_width = total_buttons_width.min(rect.width() * 0.9);

                        // Horizontal layout to push the block to the exact center
                        crate::app_helpers::center_row(ui, |ui| {
                            let left_space = (ui.available_width() - block_width).max(0.0) / 2.0;
                            ui.add_space(left_space);

                            ui.vertical(|ui| {
                                ui.set_max_width(block_width);

                                ui.horizontal_wrapped(|ui| {
                                    ui.spacing_mut().item_spacing =
                                        egui::vec2(item_spacing, item_spacing);
                                    for (icon, label, url) in &sponsors {
                                        Self::sponsor_link(ui, icon, label, url);
                                    }
                                });
                            });
                        });

                        ui.add_space(50.0);

                        // Close Button
                        // Let's make it look prominent
                        let close_btn = egui::Button::new(
                            egui::RichText::new(t!("dialog.btn_close")).size(20.0),
                        )
                        .fill(ui.visuals().selection.bg_fill)
                        .corner_radius(4.0);

                        if ui.add(close_btn).clicked() {
                            self.show_about_panel = false;
                        }
                    });
                });
            });
    }
}
