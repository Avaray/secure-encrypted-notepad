use crate::app_state::{BatchMode, KeyStatus};
use crate::crypto::{decrypt_bytes, encrypt_bytes};
use crate::EditorApp;
use eframe::egui;
use std::path::Path;

impl EditorApp {
    pub(crate) fn render_batch_converter_panel(&mut self, ctx: &egui::Context) {
        if !self.show_batch_converter {
            return;
        }

        self.refresh_batch_file_access_status();

        let mut central_panel_frame = egui::Frame::NONE;
        central_panel_frame.inner_margin = egui::Margin::same(0);

        egui::CentralPanel::default()
            .frame(central_panel_frame)
            .show(ctx, |ui| {
                self.render_batch_converter_body(ui);
            });
    }

    fn render_batch_converter_body(&mut self, ui: &mut egui::Ui) {
        // --- Unified Header ---
        egui::TopBottomPanel::top("batch_header_panel")
            .resizable(false)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                left: 12,
                right: 8,
                top: 4,
                bottom: 4,
            }))
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Batch File Converter");
                    ui.label("(Encrypt, decrypt, or rotate keyfiles)");
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("✕").on_hover_text("Close and return to editor").clicked() {
                            self.show_batch_converter = false;
                        }
                    });
                });
            });
        
        // Horizontal separator after header
        ui.separator();

                // === TOP / MODE INFO ===
                egui::TopBottomPanel::top("batch_top_info_panel")
                    .resizable(false)
                    .frame(egui::Frame::NONE.inner_margin(8.0))
                    .show_inside(ui, |ui| {
                        ui.label("Batch process multiple files at once. All operations are local and secure.");
                    });

                // === BOTTOM ===
                egui::TopBottomPanel::bottom("batch_bottom_panel")
                    .resizable(false)
                    .frame(egui::Frame::NONE.inner_margin(8.0))
                    .show_inside(ui, |ui| {
                        let is_running = self.batch_is_running;
                        let has_files = !self.batch_files.is_empty();
                        let has_keyfile = self.batch_keyfile.is_some();
                        let has_new_keyfile = self.batch_keyfile_new.is_some();

                        let enabled = !is_running && match self.batch_mode {
                            BatchMode::Encrypt | BatchMode::Decrypt => has_files && has_keyfile,
                            BatchMode::Rotate => has_files && has_keyfile && has_new_keyfile,
                        };

                        let (label, icon) = if is_running {
                            let mode_icon = match self.batch_mode {
                                BatchMode::Encrypt => "🔒",
                                BatchMode::Decrypt => "🔓",
                                BatchMode::Rotate => "🔄",
                            };
                            let verb = match self.batch_mode {
                                BatchMode::Encrypt => "Encrypting",
                                BatchMode::Decrypt => "Decrypting",
                                BatchMode::Rotate => "Rotating",
                            };
                            (format!("{} {}/{} (OK: {}, ERR: {})", verb, self.batch_progress_count, self.batch_total_count, self.batch_success_count, self.batch_failed_count), mode_icon)
                        } else {
                            match self.batch_mode {
                                BatchMode::Encrypt => ("Encrypt All".to_string(), "🔒"),
                                BatchMode::Decrypt => ("Decrypt All".to_string(), "🔓"),
                                BatchMode::Rotate => ("Rotate All".to_string(), "🔄"),
                            }
                        };

                        ui.add_enabled_ui(enabled || is_running, |ui| {
                            let btn_size = egui::vec2(ui.available_width(), 32.0);
                            if ui.add_sized(btn_size, egui::Button::new(format!("{} {}", icon, label))).clicked() && !is_running {
                                self.execute_batch_action();
                            }
                        });
                    });

                // === LEFT PANEL ===
                let initial_width = ui.available_width() / 3.0;
                egui::SidePanel::left("batch_left_panel")
                    .resizable(true)
                    .default_width(initial_width)
                    .width_range((initial_width * 0.5)..=(ui.available_width() * 0.8))
                    .frame(egui::Frame::NONE.inner_margin(8.0))
                    .show_inside(ui, |ui| {
                        if self.batch_is_running {
                            ui.disable();
                        }
                        // --- Mode Selector ---
                        let available_w = ui.available_width();
                        if available_w > 370.0 {
                            ui.horizontal(|ui| {
                                ui.heading("Mode");
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.selectable_value(&mut self.batch_mode, BatchMode::Rotate, "🔄 Rotate");
                                    ui.selectable_value(&mut self.batch_mode, BatchMode::Decrypt, "🔓 Decrypt");
                                    ui.selectable_value(&mut self.batch_mode, BatchMode::Encrypt, "🔒 Encrypt");
                                });
                            });
                        } else {
                            ui.heading("Mode");
                            ui.horizontal(|ui| {
                                ui.selectable_value(&mut self.batch_mode, BatchMode::Encrypt, "🔒 Encrypt");
                                ui.selectable_value(&mut self.batch_mode, BatchMode::Decrypt, "🔓 Decrypt");
                                ui.selectable_value(&mut self.batch_mode, BatchMode::Rotate, "🔄 Rotate");
                            });
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // --- Keyfile Section ---
                        let keyfile_label = match self.batch_mode {
                            BatchMode::Rotate => "Old Keyfile",
                            _ => "Keyfile",
                        };

                        let available_w = ui.available_width();
                        if available_w > 200.0 {
                            ui.horizontal(|ui| {
                                ui.heading(keyfile_label);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Select keyfile").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            self.batch_keyfile = Some(path);
                                        }
                                    }
                                });
                            });
                        } else {
                            ui.heading(keyfile_label);
                            ui.horizontal(|ui| {
                                if ui.button("Select keyfile").clicked() {
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

                        // --- New Keyfile (Rotate mode only) ---
                        if self.batch_mode == BatchMode::Rotate {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            if available_w > 200.0 {
                                ui.horizontal(|ui| {
                                    ui.heading("New Keyfile");
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Select keyfile").clicked() {
                                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                                self.batch_keyfile_new = Some(path);
                                            }
                                        }
                                    });
                                });
                            } else {
                                ui.heading("New Keyfile");
                                ui.horizontal(|ui| {
                                    if ui.button("Select keyfile").clicked() {
                                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                                            self.batch_keyfile_new = Some(path);
                                        }
                                    }
                                });
                            }

                            ui.horizontal_wrapped(|ui| {
                                if let Some(path) = &self.batch_keyfile_new {
                                    ui.label(
                                        egui::RichText::new(format!("🔑 {}", self.mask_keyfile_path(path)))
                                            .color(self.current_theme.colors.success_color()),
                                    );
                                } else {
                                    ui.label(
                                        egui::RichText::new("No new keyfile selected")
                                            .color(self.current_theme.colors.warning_color()),
                                    );
                                }
                            });
                        }

                        ui.add_space(4.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // --- Output Directory ---
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

                        // --- Output Extension (Decrypt mode only) ---
                        if self.batch_mode == BatchMode::Decrypt {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.horizontal(|ui| {
                                ui.heading("Output Extension");
                                ui.add_space(8.0);
                                let mut ext = self.batch_output_extension.clone();
                                if ui.add(egui::TextEdit::singleline(&mut ext).hint_text("no extension")).changed() {
                                    self.batch_output_extension = ext.trim_start_matches('.').to_string();
                                }
                            });
                        }
                    });

                // === RIGHT/CENTER PANEL ===
                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE.inner_margin(8.0))
                    .show_inside(ui, |ui| {
                        if self.batch_is_running {
                            ui.disable();
                        }
                        let files_count = self.batch_files.len();
                        let heading_text = if files_count > 0 {
                            format!("Input Files ({})", files_count)
                        } else {
                            "Input Files".to_string()
                        };

                        if ui.available_width() > 320.0 {
                            ui.horizontal(|ui| {
                                ui.heading(&heading_text);
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button("Clean List").clicked() {
                                        self.batch_files.clear();
                                    }
                                    if ui.button("Add Files").clicked() {
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
                            ui.heading(&heading_text);
                            ui.horizontal(|ui| {
                                if ui.button("Add Files").clicked() {
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
                                        let status = self.batch_file_access_cache.get(file).cloned().unwrap_or(KeyStatus::Unknown);
                                        let text_color = match status {
                                            KeyStatus::Decryptable => self.current_theme.colors.success_color(),
                                            KeyStatus::WrongKey => self.current_theme.colors.error_color(),
                                            _ => ui.visuals().text_color(),
                                        };

                                        ui.horizontal(|ui| {
                                            // Reserve button space first so long filenames can't push it off-screen.
                                            let btn_width = ui.spacing().interact_size.y
                                                + ui.spacing().button_padding.x * 2.0;
                                            let label_width = (ui.available_width()
                                                - btn_width
                                                - ui.spacing().item_spacing.x)
                                                .max(0.0);

                                            let file_name = file
                                                .file_name()
                                                .unwrap_or_default()
                                                .to_string_lossy();
                                            ui.allocate_ui_with_layout(
                                                egui::vec2(label_width, ui.spacing().interact_size.y),
                                                egui::Layout::left_to_right(egui::Align::Center),
                                                |ui| {
                                                    ui.add(
                                                        egui::Label::new(
                                                            egui::RichText::new(file_name.as_ref())
                                                                .color(text_color),
                                                        )
                                                        .truncate(),
                                                    );
                                                },
                                            );

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
    }

    /// Execute the batch action based on the current mode
    fn execute_batch_action(&mut self) {
        match self.batch_mode {
            BatchMode::Encrypt => self.batch_encrypt(),
            BatchMode::Decrypt => self.batch_decrypt(),
            BatchMode::Rotate => self.batch_rotate(),
        }
    }

    fn batch_encrypt(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.batch_progress_receiver = Some(rx);
        self.batch_is_running = true;
        self.batch_progress_count = 0;
        self.batch_total_count = self.batch_files.len();
        self.batch_success_count = 0;
        self.batch_failed_count = 0;

        let batch_files = self.batch_files.clone();
        let keyfile = self.batch_keyfile.clone().unwrap();
        let batch_output_dir = self.batch_output_dir.clone();

        if let Err(e) = crate::crypto::get_keyfile_hash(&keyfile) {
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("Critical: Keyfile error: {}", e)));
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(0, 0));
            return;
        }

        std::thread::spawn(move || {
            let mut success = 0;
            let mut failed = 0;

            for (i, file) in batch_files.iter().enumerate() {
                // Heuristic Check: Is it already a SEN file?
                if crate::crypto::is_sen_file(file) {
                    failed += 1;
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Warning, format!("[Encrypt] Ignored: File is already encrypted (SEN header detected): {}", file.file_name().unwrap_or_default().to_string_lossy())));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
                    continue;
                }

                let output_dir = if let Some(d) = &batch_output_dir {
                    d.clone()
                } else {
                    file.parent().unwrap_or(Path::new(".")).to_path_buf()
                };

                let file_name = file.file_name().unwrap_or_default();
                let output_path = output_dir.join(format!("{}.sen", file_name.to_string_lossy()));

                match std::fs::read(file) {
                    Ok(buffer) => {
                        // Binary Check: Heuristic check if it's actually text
                        if !crate::crypto::is_buffer_text(&buffer) {
                             failed += 1;
                             let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Warning, format!("[Encrypt] Ignored: File appears to be binary (non-text): {}", file.file_name().unwrap_or_default().to_string_lossy())));
                             let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
                             continue;
                        }

                        match encrypt_bytes(&buffer, &keyfile, &output_path) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Success, format!("[Encrypt] Success: {}", file.file_name().unwrap_or_default().to_string_lossy())));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Encrypt] Failed {}: {}", file.file_name().unwrap_or_default().to_string_lossy(), e)));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Encrypt] Read Error {}: {}", file.file_name().unwrap_or_default().to_string_lossy(), e)));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(success, failed));
        });
    }

    fn batch_decrypt(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.batch_progress_receiver = Some(rx);
        self.batch_is_running = true;
        self.batch_progress_count = 0;
        self.batch_total_count = self.batch_files.len();
        self.batch_success_count = 0;
        self.batch_failed_count = 0;

        let batch_files = self.batch_files.clone();
        let keyfile = self.batch_keyfile.clone().unwrap();
        let batch_output_dir = self.batch_output_dir.clone();
        let batch_output_extension = self.batch_output_extension.clone();

        std::thread::spawn(move || {
            let mut success = 0;
            let mut failed = 0;

            for (i, file) in batch_files.iter().enumerate() {
                // Heuristic Check: Is it a SEN file?
                if !crate::crypto::is_sen_file(file) {
                    failed += 1;
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Warning, format!("[Decrypt] Ignored: Not a recognized SEN file (invalid magic): {}", file.file_name().unwrap_or_default().to_string_lossy())));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
                    continue;
                }

                let output_dir = if let Some(d) = &batch_output_dir {
                    d.clone()
                } else {
                    file.parent().unwrap_or(Path::new(".")).to_path_buf()
                };

                let original_name = file.file_name().unwrap_or_default().to_string_lossy();
                let stem = if original_name.ends_with(".sen") {
                    original_name.trim_end_matches(".sen").to_string()
                } else {
                    original_name.to_string()
                };

                let new_name = if batch_output_extension.is_empty() {
                    stem
                } else {
                    format!("{}.{}", stem, batch_output_extension)
                };

                let mut output_path = output_dir.join(&new_name);
                if output_path == *file {
                    output_path = output_dir.join(format!("{}.decrypted", new_name));
                }

                match decrypt_bytes(&keyfile, file) {
                    Ok(buffer) => {
                        match std::fs::write(&output_path, buffer) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Success, format!("[Decrypt] Success: {}", file.file_name().unwrap_or_default().to_string_lossy())));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Decrypt] Write Error {}: {}", output_path.file_name().unwrap_or_default().to_string_lossy(), e)));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Decrypt] Error {}: {}", file.file_name().unwrap_or_default().to_string_lossy(), e)));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(success, failed));
        });
    }

    fn batch_rotate(&mut self) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.batch_progress_receiver = Some(rx);
        self.batch_is_running = true;
        self.batch_progress_count = 0;
        self.batch_total_count = self.batch_files.len();
        self.batch_success_count = 0;
        self.batch_failed_count = 0;

        let batch_files = self.batch_files.clone();
        let old_keyfile = self.batch_keyfile.clone().unwrap();
        let new_keyfile = self.batch_keyfile_new.clone().unwrap();

        std::thread::spawn(move || {
            let mut success = 0;
            let mut failed = 0;

            for (i, file) in batch_files.iter().enumerate() {
                // Heuristic Check: Is it a SEN file?
                if !crate::crypto::is_sen_file(file) {
                    failed += 1;
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Warning, format!("[Rotate] Ignored: Not a recognized SEN file (invalid magic): {}", file.file_name().unwrap_or_default().to_string_lossy())));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
                    continue;
                }

                // Step 1: Decrypt with old keyfile
                match decrypt_bytes(&old_keyfile, file) {
                    Ok(buffer) => {
                        // Step 2: Re-encrypt with new keyfile, writing back to the same file
                        match encrypt_bytes(&buffer, &new_keyfile, file) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Success, format!("[Rotate] Success: {}", file.file_name().unwrap_or_default().to_string_lossy())));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Rotate] Re-encrypt Error {}: {}", file.file_name().unwrap_or_default().to_string_lossy(), e)));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(crate::app_state::LogLevel::Error, format!("[Rotate] Decrypt Error {}: {}", file.file_name().unwrap_or_default().to_string_lossy(), e)));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(i + 1, success, failed));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(success, failed));
        });
    }
}
