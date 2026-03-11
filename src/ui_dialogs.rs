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
                    ui.add(egui::Label::new("This will restore all settings to their factory defaults.").wrap_mode(egui::TextWrapMode::Extend));
                    ui.add(egui::Label::new("This action cannot be undone.").wrap_mode(egui::TextWrapMode::Extend));
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

                    // Only enable OK if slider is fully to the right
                    let is_confirmed = self.reset_slider_val >= 0.99;
                    
                    ui.horizontal(|ui| {
                        ui.add_enabled_ui(is_confirmed, |ui| {
                            if ui.button("OK").clicked() {
                                self.settings = crate::settings::Settings::default();
                                let _ = self.settings.save();
                                self.show_reset_confirmation = false;
                                self.style_dirty = true; // Apply default fonts/sizes
                                self.status_message = "All settings have been reset to factory defaults".to_string();
                                self.log_warning("All settings have been reset to factory defaults");
                            }
                        });
                        
                        if ui.button("Cancel").clicked() {
                            self.show_reset_confirmation = false;
                        }
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
}
