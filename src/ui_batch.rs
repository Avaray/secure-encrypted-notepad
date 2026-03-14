use crate::crypto::{decrypt_file, encrypt_file};
use crate::EditorApp;
use eframe::egui;
use std::path::Path;

impl EditorApp {
    pub(crate) fn render_batch_converter_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_batch_converter;

        egui::Window::new("Batch Converter")
            .open(&mut open)
            .resizable(true)
            .collapsible(false)
            .default_width(600.0)
            .default_height(400.0)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.available_rect().center())
            .show(ctx, |ui| {
                ui.heading("Batch Encryption / Decryption");
                ui.label("Convert multiple files at once using a keyfile.");

                ui.separator();

                ui.columns(2, |columns| {
                    // LEFT COLUMN: Config
                    columns[0].vertical(|ui| {
                        // Keyfile Selection
                        ui.heading("1. Keyfile");
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &self.batch_keyfile {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "🔑 {}",
                                        self.mask_keyfile_path(path)
                                    ))
                                    .color(self.current_theme.colors.success_color()),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new("No keyfile selected")
                                        .color(self.current_theme.colors.warning_color()),
                                );
                            }
                        });
                        if ui.button("Select Keyfile...").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.batch_keyfile = Some(path);
                            }
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Output Directory
                        ui.heading("2. Output Directory");
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &self.batch_output_dir {
                                ui.label(format!("📁 {}", self.mask_directory_path(path)));
                            } else {
                                ui.label("Same as input files (default)");
                            }
                        });
                        ui.horizontal(|ui| {
                            if ui.button("Select Output Dir...").clicked() {
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

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // Actions
                        ui.heading("3. Actions");
                        let enabled = !self.batch_files.is_empty() && self.batch_keyfile.is_some();

                        ui.vertical_centered_justified(|ui| {
                            if ui
                                .add_enabled(enabled, egui::Button::new("🔒 Encrypt All"))
                                .clicked()
                            {
                                self.status_message = "Batch encryption started...".to_string();
                                self.log_info("Batch encryption requested");

                                if let Some(keyfile) = self.batch_keyfile.clone() {
                                    let mut success = 0;
                                    let mut failed = 0;
                                    let batch_files = self.batch_files.clone();
                                    let batch_output_dir = self.batch_output_dir.clone();
                                    let total = batch_files.len();

                                    for file in &batch_files {
                                        // Determine output directory
                                        let output_dir = if let Some(d) = &batch_output_dir {
                                            d.clone()
                                        } else {
                                            file.parent().unwrap_or(Path::new(".")).to_path_buf()
                                        };

                                        // Determine output filename (append .sen)
                                        let file_name = file.file_name().unwrap_or_default();
                                        let output_path = output_dir
                                            .join(format!("{}.sen", file_name.to_string_lossy()));

                                        // Read content (assuming text)
                                        match std::fs::read_to_string(file) {
                                            Ok(content) => {
                                                // Encrypt
                                                match encrypt_file(&content, &keyfile, &output_path)
                                                {
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

                                    self.status_message = format!(
                                        "Batch Encrypt: {}/{} succeeded, {} failed",
                                        success, total, failed
                                    );
                                }
                            }

                            if ui
                                .add_enabled(enabled, egui::Button::new("🔓 Decrypt All"))
                                .clicked()
                            {
                                self.status_message = "Batch decryption started...".to_string();
                                self.log_info("Batch decryption requested");

                                if let Some(keyfile) = self.batch_keyfile.clone() {
                                    let mut success = 0;
                                    let mut failed = 0;
                                    let batch_files = self.batch_files.clone();
                                    let batch_output_dir = self.batch_output_dir.clone();
                                    let total = batch_files.len();

                                    for file in &batch_files {
                                        // Determine output directory
                                        let output_dir = if let Some(d) = &batch_output_dir {
                                            d.clone()
                                        } else {
                                            file.parent().unwrap_or(Path::new(".")).to_path_buf()
                                        };

                                        // Determine output filename (strip .sen or append .txt)
                                        let original_name =
                                            file.file_name().unwrap_or_default().to_string_lossy();
                                        let new_name = if original_name.ends_with(".sen") {
                                            original_name.trim_end_matches(".sen").to_string()
                                        } else {
                                            format!("{}.txt", original_name)
                                        };

                                        // Prevent overwriting source if names clash (e.g. decrypting file.txt to file.txt)
                                        let mut output_path = output_dir.join(&new_name);
                                        if output_path == *file {
                                            output_path =
                                                output_dir.join(format!("{}.decrypted", new_name));
                                        }

                                        // Decrypt
                                        match decrypt_file(&keyfile, file) {
                                            Ok(content) => {
                                                // Write result
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

                                    self.status_message = format!(
                                        "Batch Decrypt: {}/{} succeeded, {} failed",
                                        success, total, failed
                                    );
                                }
                            }
                        });
                    });

                    // RIGHT COLUMN: File List
                    columns[1].vertical(|ui| {
                        ui.heading("4. Input Files");
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

                        ui.add_space(4.0);

                        let height = ui.available_height() - 20.0;
                        egui::ScrollArea::vertical()
                            .max_height(height)
                            .show(ui, |ui| {
                                if self.batch_files.is_empty() {
                                    ui.label("No files added.");
                                } else {
                                    let mut to_remove = None;
                                    for (idx, file) in self.batch_files.iter().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(
                                                file.file_name()
                                                    .unwrap_or_default()
                                                    .to_string_lossy(),
                                            );
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.button("❌").clicked() {
                                                        to_remove = Some(idx);
                                                    }
                                                },
                                            );
                                        });
                                    }

                                    if let Some(idx) = to_remove {
                                        self.batch_files.remove(idx);
                                    }
                                }
                            });
                    });
                });
            });

        self.show_batch_converter = open;
    }
}
