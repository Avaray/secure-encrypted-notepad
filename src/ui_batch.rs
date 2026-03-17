use crate::crypto::{decrypt_file, encrypt_file};
use crate::EditorApp;
use eframe::egui;
use std::path::Path;

impl EditorApp {
    pub(crate) fn render_batch_converter_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_batch_converter;

        let content_rect = ctx.content_rect();

        egui::Window::new("Batch Converter")
            .id(egui::Id::new("batch_converter_v6"))
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .default_width(content_rect.width() * 0.5)
            .default_height(content_rect.height() * 0.5)
            .pivot(egui::Align2::CENTER_CENTER)
            .constrain_to(ctx.content_rect().shrink(16.0))
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                // === TOP ===
                egui::TopBottomPanel::top("batch_top_panel")
                    .frame(egui::Frame::NONE.inner_margin(12.0))
                    .show_inside(ui, |ui| {
                        ui.heading("Batch Encryption / Decryption");
                        ui.label("Convert multiple files at once using a keyfile.");
                    });

                // === BOTTOM ===
                egui::TopBottomPanel::bottom("batch_bottom_panel")
                    .frame(egui::Frame::NONE.inner_margin(12.0))
                    .show_inside(ui, |ui| {
                        let enabled = !self.batch_files.is_empty() && self.batch_keyfile.is_some();

                        ui.add_enabled_ui(enabled, |ui| {
                            ui.horizontal(|ui| {
                                let btn_width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.0;
                                let btn_size = egui::vec2(btn_width, 32.0);

                                if ui.add_sized(btn_size, egui::Button::new("🔒 Encrypt All")).clicked() {
                                    self.status_message = "Batch encryption started...".to_string();
                                    self.log_info("Batch encryption requested");

                                    if let Some(keyfile) = self.batch_keyfile.clone() {
                                        let mut success = 0;
                                        let mut failed = 0;
                                        let batch_files = self.batch_files.clone();
                                        let batch_output_dir = self.batch_output_dir.clone();
                                        let total = batch_files.len();

                                        for file in &batch_files {
                                            let output_dir = if let Some(d) = &batch_output_dir {
                                                d.clone()
                                            } else {
                                                file.parent().unwrap_or(Path::new(".")).to_path_buf()
                                            };

                                            let file_name = file.file_name().unwrap_or_default();
                                            let output_path = output_dir.join(format!("{}.sen", file_name.to_string_lossy()));

                                            match std::fs::read_to_string(file) {
                                                Ok(content) => {
                                                    match encrypt_file(&content, &keyfile, &output_path) {
                                                        Ok(_) => {
                                                            success += 1;
                                                            let masked_in = self.mask_directory_path(file);
                                                            let masked_out = self.mask_directory_path(&output_path);
                                                            if masked_in == "Secured" && masked_out == "Secured" {
                                                                self.log_success("File encrypted successfully".to_string());
                                                            } else {
                                                                self.log_success(format!("Encrypted: {} -> {}", masked_in, masked_out));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            failed += 1;
                                                            self.log_error(format!(
                                                                "Failed to encrypt {}: {}",
                                                                self.mask_directory_path(file),
                                                                e
                                                            ));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    failed += 1;
                                                    self.log_error(format!(
                                                        "Failed to read {}: {}",
                                                        self.mask_directory_path(file),
                                                        e
                                                    ));
                                                }
                                            }
                                        }

                                        self.status_message = format!("Batch Encrypt: {}/{} succeeded, {} failed", success, total, failed);
                                    }
                                }

                                if ui.add_sized(btn_size, egui::Button::new("🔓 Decrypt All")).clicked() {
                                    self.status_message = "Batch decryption started...".to_string();
                                    self.log_info("Batch decryption requested");

                                    if let Some(keyfile) = self.batch_keyfile.clone() {
                                        let mut success = 0;
                                        let mut failed = 0;
                                        let batch_files = self.batch_files.clone();
                                        let batch_output_dir = self.batch_output_dir.clone();
                                        let total = batch_files.len();

                                        for file in &batch_files {
                                            let output_dir = if let Some(d) = &batch_output_dir {
                                                d.clone()
                                            } else {
                                                file.parent().unwrap_or(Path::new(".")).to_path_buf()
                                            };

                                            let original_name = file.file_name().unwrap_or_default().to_string_lossy();
                                            let new_name = if original_name.ends_with(".sen") {
                                                original_name.trim_end_matches(".sen").to_string()
                                            } else {
                                                format!("{}.txt", original_name)
                                            };

                                            let mut output_path = output_dir.join(&new_name);
                                            if output_path == *file {
                                                output_path = output_dir.join(format!("{}.decrypted", new_name));
                                            }

                                            match decrypt_file(&keyfile, file) {
                                                Ok(content) => {
                                                    match std::fs::write(&output_path, content) {
                                                        Ok(_) => {
                                                            success += 1;
                                                            let masked_in = self.mask_directory_path(file);
                                                            let masked_out = self.mask_directory_path(&output_path);
                                                            if masked_in == "Secured" && masked_out == "Secured" {
                                                                self.log_success("File decrypted successfully".to_string());
                                                            } else {
                                                                self.log_success(format!("Decrypted: {} -> {}", masked_in, masked_out));
                                                            }
                                                        }
                                                        Err(e) => {
                                                            failed += 1;
                                                            self.log_error(format!(
                                                                "Failed to write {}: {}",
                                                                self.mask_directory_path(&output_path),
                                                                e
                                                            ));
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    failed += 1;
                                                    self.log_error(format!(
                                                        "Failed to decrypt {}: {}",
                                                        self.mask_directory_path(file),
                                                        e
                                                    ));
                                                }
                                            }
                                        }

                                        self.status_message = format!("Batch Decrypt: {}/{} succeeded, {} failed", success, total, failed);
                                    }
                                }
                            });
                        });
                    });

                // === LEFT PANEL ===
                let half_width = ui.available_width() / 2.0;
                egui::SidePanel::left("batch_left_panel")
                    .resizable(true)
                    .default_width(half_width)
                    .width_range((half_width * 0.2)..=(half_width * 1.8))
                    .frame(egui::Frame::NONE.inner_margin(12.0))
                    .show_inside(ui, |ui| {
                        let available_w = ui.available_width();
                        if available_w > 200.0 {
                            ui.horizontal(|ui| {
                                ui.heading("Keyfile");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Select Keyfile...").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            self.batch_keyfile = Some(path);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.heading("Keyfile");
                            ui.horizontal(|ui| {
                                if ui.button("Select Keyfile...").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                                        self.batch_keyfile = Some(path);
                                    }
                                }
                            });
                        }
                        
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &self.batch_keyfile {
                                ui.label(
                                    egui::RichText::new(format!("🔑 {}", self.mask_keyfile_path(path)))
                                        .color(self.current_theme.colors.success_color()),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("No keyfile selected")
                                        .color(self.current_theme.colors.warning_color()),
                                );
                            }
                        });

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        if ui.available_width() > 320.0 {
                            ui.horizontal(|ui| {
                                ui.heading("Output Directory");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if self.batch_output_dir.is_some() {
                                        if ui.button("Clear").clicked() {
                                            self.batch_output_dir = None;
                                        }
                                    }
                                    if ui.button("Select Output Directory").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                            self.batch_output_dir = Some(path);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.heading("Output Directory");
                            ui.horizontal(|ui| {
                                if ui.button("Select Output Directory").clicked() {
                                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                        self.batch_output_dir = Some(path);
                                    }
                                }
                                if self.batch_output_dir.is_some() {
                                    if ui.button("Clear").clicked() {
                                        self.batch_output_dir = None;
                                    }
                                }
                            });
                        }
                        
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &self.batch_output_dir {
                                let masked = self.mask_directory_path(path);
                                let color = if masked == "Secured" {
                                    self.current_theme.colors.success_color()
                                } else {
                                    self.current_theme.colors.warning_color()
                                };
                                ui.label(egui::RichText::new(format!("📁 {}", masked)).color(color));
                            } else {
                                ui.label("Same as input files (default)");
                            }
                        });
                    });

                // === RIGHT/CENTER PANEL ===
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.inner_margin(12.0))
                    .show_inside(ui, |ui| {
                        if ui.available_width() > 320.0 {
                            ui.horizontal(|ui| {
                                ui.heading("Input Files");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Clean List").clicked() {
                                        self.batch_files.clear();
                                    }
                                    if ui.button("➕ Add Files...").clicked() {
                                        if let Some(files) = rfd::FileDialog::new().pick_files() {
                                            for file in files {
                                                if !self.batch_files.contains(&file) {
                                                    self.batch_files.push(file);
                                                }
                                            }
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.heading("Input Files");
                            ui.horizontal(|ui| {
                                if ui.button("➕ Add Files...").clicked() {
                                    if let Some(files) = rfd::FileDialog::new().pick_files() {
                                        for file in files {
                                            if !self.batch_files.contains(&file) {
                                                self.batch_files.push(file);
                                            }
                                        }
                                    }
                                }
                                if ui.button("Clean List").clicked() {
                                    self.batch_files.clear();
                                }
                            });
                        }

                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if self.batch_files.is_empty() {
                                    ui.label("No files added.");
                                } else {
                                    let mut to_remove = None;
                                    for (idx, file) in self.batch_files.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(file.file_name().unwrap_or_default().to_string_lossy());
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.button("❌").clicked() {
                                                    to_remove = Some(idx);
                                                }
                                            });
                                        });
                                    }

                                    if let Some(idx) = to_remove {
                                        self.batch_files.remove(idx);
                                    }
                                }
                            });
                    });
            });

        self.show_batch_converter = open;
    }
}
