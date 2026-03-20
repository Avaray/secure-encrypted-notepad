use crate::app_state::PendingAction;
use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Render confirmation dialog for unsaved changes
    pub(crate) fn render_confirmation_dialog(&mut self, ctx: &egui::Context) {
        // Unsaved changes dialog
        if self.show_close_confirmation {
            egui::Window::new("Unsaved Changes")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_max_width(300.0);
                    ui.label("You have unsaved changes. Do you want to save them?");
                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
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

                        if ui.button("Don't Save").clicked() {
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
                            self.status_message = "Save cancelled or failed".to_string();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_close_confirmation = false;
                            self.pending_action = PendingAction::None;
                        }
                    });
                });
        }

        // Settings reset dialog
        if self.show_reset_confirmation {
            egui::Window::new("Reset All Settings")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .show(ctx, |ui| {
                    ui.set_max_width(400.0);
                    ui.add(
                        egui::Label::new(
                            "This will restore all settings to their factory defaults.",
                        )
                        .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    ui.add(
                        egui::Label::new("This action cannot be undone.")
                            .wrap_mode(egui::TextWrapMode::Extend),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Slide to the right to confirm:");
                        ui.spacing_mut().slider_width = ui.available_width();
                        ui.add(
                            egui::Slider::new(&mut self.reset_slider_val, 0.0..=1.0)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                    });
                    ui.add_space(8.0);

                    let is_confirmed = self.reset_slider_val >= 0.99;
                    ui.horizontal(|ui| {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.add_enabled_ui(is_confirmed, |ui| {
                                if ui.button("OK").clicked() {
                                    let was_maximized = self.settings.start_maximized;
                                    self.settings = crate::settings::Settings::default();
                                    self.settings.start_maximized = was_maximized;
                                    let _ = self.settings.save();
                                    self.show_reset_confirmation = false;
                                    self.style_dirty = true; // Apply default fonts/sizes
                                    self.status_message =
                                        "All settings have been reset to factory defaults".to_string();
                                    self.log_warning(
                                        "All settings have been reset to factory defaults",
                                    );
                                }
                            });

                            if ui.button("Cancel").clicked() {
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

        egui::Window::new("Go to Line")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Line number:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.goto_line_input).desired_width(100.0),
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

                ui.horizontal(|ui| {
                    if ui.button("Go").clicked() {
                        if let Ok(line_num) = self.goto_line_input.parse::<usize>() {
                            jump_to_line = Some(line_num);
                        } else {
                            self.status_message = "Invalid line number".to_string();
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
            });

        // Handle jump outside the window closure
        if let Some(line_num) = jump_to_line {
            let max_line = self.document.current_content.lines().count().max(1);
            if line_num > 0 && line_num <= max_line {
                self.highlighted_line = Some(line_num);
                self.log_info(format!("Jumped to line {}", line_num));
                close = true;
            } else {
                self.status_message = format!("Line out of range (1-{})", max_line);
                self.log_warning(format!("Line {} out of range (1-{})", line_num, max_line));
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

        let timestamp_str = self.document.autosave
            .as_ref()
            .map(|a| a.timestamp.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_default();

        egui::Window::new("Auto-save Found")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.set_max_width(350.0);
                ui.label(format!(
                    "An auto-saved version was found ({}). Would you like to restore it?",
                    timestamp_str
                ));
                ui.separator();

                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Restore").clicked() {
                            if let Some(autosave) = self.document.autosave.take() {
                                self.document.current_content = autosave.content;
                                self.is_modified = true;
                                self.log_info("Restored content from auto-save");
                                self.status_message = "Auto-save restored".to_string();
                            }
                            self.show_autosave_restore = false;
                        }
                        if ui.button("Discard").clicked() {
                            self.document.clear_autosave();
                            self.log_info("Auto-save discarded");
                            self.show_autosave_restore = false;
                        }
                    });
                });
            });
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
                    ui.painter().rect_filled(rect, 0.0, ui.visuals().window_fill());

                    // Center the inner content vertically and horizontally
                    ui.vertical_centered(|ui| {
                        ui.add_space(rect.height() * 0.15); // Dynamic top margin

                        ui.add(
                            egui::Label::new(egui::RichText::new("Secure Encrypted Notepad (SEN)").size(36.0).strong())
                        );
                        ui.add_space(10.0);
                        ui.label(
                            egui::RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                                .size(18.0)
                                .weak(),
                        );

                        ui.add_space(40.0);
                        
                        // Author Info
                        ui.heading("About the Author");
                        ui.add_space(5.0);
                        ui.label("Created by Avaray — building privacy-focused and minimal tools.");
                        ui.add_space(20.0);

                        // Links Section
                        ui.heading("Links & Support");
                        ui.add_space(10.0);
                        
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.hyperlink_to("🔗 GitHub Repository", "https://github.com/Avaray/secure-encrypted-notepad");
                                ui.add_space(5.0);
                                ui.hyperlink_to("🐛 Report a Bug / Issue", "https://github.com/Avaray/secure-encrypted-notepad/issues");
                            });
                        });
                        
                        ui.add_space(20.0);

                        // Financial Support
                        ui.heading("Support the Project");
                        ui.add_space(5.0);
                        ui.label("If you find this tool useful, consider supporting its development:");
                        ui.add_space(5.0);
                        ui.horizontal(|ui| {
                            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                                ui.hyperlink_to("💖 GitHub Sponsors", "https://github.com/sponsors/Avaray");
                                ui.add_space(5.0);
                                ui.hyperlink_to("🅿️ Patreon", "https://patreon.com/Avaray_");
                                ui.add_space(5.0);
                                ui.hyperlink_to("☕ Buy Me a Coffee", "https://buymeacoffee.com/avaray");
                                ui.add_space(5.0);
                                ui.hyperlink_to("🤝 Open Collective", "https://opencollective.com/avaray");
                                ui.add_space(5.0);
                                ui.hyperlink_to("🎈 Ko-fi", "https://ko-fi.com/avaray_");
                            });
                        });

                        ui.add_space(50.0);

                        // Close Button
                        // Let's make it look prominent
                        let close_btn = egui::Button::new(egui::RichText::new("   Close Panel (F1)   ").size(20.0))
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
