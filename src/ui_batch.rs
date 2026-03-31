use crate::app_state::{BatchMode, KeyStatus};
use crate::crypto::{decrypt_bytes, encrypt_bytes};
use crate::EditorApp;
use eframe::egui;
use rust_i18n::t;
use std::path::Path;

impl EditorApp {
    pub(crate) fn render_batch_converter_panel(&mut self, ctx: &egui::Context) {
        if !self.show_batch_converter {
            return;
        }

        self.refresh_batch_file_access_status();

        let mut central_panel_frame = egui::Frame::NONE;
        central_panel_frame.inner_margin = egui::Margin::same(0);
        central_panel_frame.fill = self
            .current_theme
            .colors
            .to_egui_color32(self.current_theme.colors.background.unwrap_or([27, 27, 27, 255]));

        egui::CentralPanel::default()
            .frame(central_panel_frame)
            .show(ctx, |ui| {
                self.render_batch_converter_body(ui);
            });
    }

    fn render_batch_converter_body(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);

        // --- Unified Header ---
        egui::TopBottomPanel::top("batch_header_panel")
            .resizable(false)
            .show_separator_line(true)
            .frame(egui::Frame::NONE.inner_margin(egui::Margin {
                left: 12,
                right: 4,
                top: 12,
                bottom: 12,
            }))
            .show_inside(ui, |ui| {
                let h = ls.get_height("batch_header");
                if self.render_panel_header(
                    ui,
                    &t!("batch.title"),
                    Some(&t!("batch.subtitle")),
                    false, // add_separator
                    h,
                ) {
                    self.show_batch_converter = false;
                }
            });

        self.layout_state = ls;

        ui.add_space(2.0);

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
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(t!("batch.mode_label")).strong());
                    ui.add_space(4.0);
                    crate::app_helpers::center_row(ui, |ui| {
                        ui.selectable_value(
                            &mut self.batch_mode,
                            BatchMode::Encrypt,
                            t!("batch.mode_encrypt"),
                        );
                        ui.selectable_value(
                            &mut self.batch_mode,
                            BatchMode::Decrypt,
                            t!("batch.mode_decrypt"),
                        );
                        ui.selectable_value(
                            &mut self.batch_mode,
                            BatchMode::Rotate,
                            t!("batch.mode_rotate"),
                        );
                    });
                    ui.add_space(4.0);
                    let mode_desc = match self.batch_mode {
                        BatchMode::Encrypt => t!("batch.desc_encrypt"),
                        BatchMode::Decrypt => t!("batch.desc_decrypt"),
                        BatchMode::Rotate => t!("batch.desc_rotate"),
                    };
                    ui.label(egui::RichText::new(mode_desc).weak().small());
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // --- Keyfile Section ---
                let keyfile_label = match self.batch_mode {
                    BatchMode::Rotate => t!("batch.key_old_label"),
                    _ => t!("batch.key_label"),
                };

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(keyfile_label).strong());
                    ui.add_space(4.0);
                    crate::app_helpers::center_row(ui, |ui| {
                        if ui.button(t!("batch.btn_select_key")).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.batch_keyfile = Some(path);
                            }
                        }
                        if let Some(path) = &self.keyfile_path {
                            if ui
                                .button(t!("batch.btn_load_editor"))
                                .on_hover_text(t!("batch.load_editor_tooltip"))
                                .clicked()
                            {
                                self.batch_keyfile = Some(path.clone());
                            }
                        }
                    });
                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        if let Some(path) = &self.batch_keyfile {
                            ui.label(
                                egui::RichText::new(self.mask_keyfile_path(path))
                                    .color(self.current_theme.colors.success_color()),
                            );
                        } else {
                            ui.label(egui::RichText::new(t!("batch.key_none")).weak());
                        }
                    });
                });

                // --- New Keyfile (Rotate mode only) ---
                if self.batch_mode == BatchMode::Rotate {
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(12.0);

                    ui.vertical(|ui| {
                        ui.label(egui::RichText::new(t!("batch.key_new_label")).strong());
                        ui.add_space(4.0);
                        crate::app_helpers::center_row(ui, |ui| {
                            if ui.button(t!("batch.btn_select_key")).clicked() {
                                if let Some(path) = rfd::FileDialog::new().pick_file() {
                                    self.batch_keyfile_new = Some(path);
                                }
                            }
                        });
                        ui.add_space(2.0);
                        ui.horizontal_wrapped(|ui| {
                            if let Some(path) = &self.batch_keyfile_new {
                                ui.label(
                                    egui::RichText::new(self.mask_keyfile_path(path))
                                        .color(self.current_theme.colors.success_color()),
                                );
                            } else {
                                ui.label(egui::RichText::new(t!("batch.key_none")).weak());
                            }
                        });
                    });
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // --- Output Configuration ---
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(t!("batch.output_label")).strong());
                    ui.add_space(4.0);

                    // Output Directory
                    crate::app_helpers::center_row(ui, |ui| {
                        if ui.button(t!("batch.btn_select_out_dir")).clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                self.batch_output_dir = Some(path);
                            }
                        }
                        if self.batch_output_dir.is_some() {
                            if ui.button(t!("batch.btn_clear_out_dir")).clicked() {
                                self.batch_output_dir = None;
                            }
                        }
                    });
                    ui.add_space(2.0);
                    ui.horizontal_wrapped(|ui| {
                        if let Some(path) = &self.batch_output_dir {
                            let masked = self.mask_directory_path(path);
                            let color = if masked == "Secured" {
                                self.current_theme.colors.success_color()
                            } else {
                                self.current_theme.colors.warning_color()
                            };
                            ui.label(egui::RichText::new(masked).color(color));
                        } else {
                            ui.label(egui::RichText::new(t!("batch.output_default")).weak());
                        }
                    });

                    // Output Extension (Decrypt mode only)
                    if self.batch_mode == BatchMode::Decrypt {
                        ui.add_space(8.0);
                        crate::app_helpers::center_row(ui, |ui| {
                            ui.label(t!("batch.extension_label"));
                            let mut ext = self.batch_output_extension.clone();
                            if ui
                                .add(
                                    egui::TextEdit::singleline(&mut ext)
                                        .hint_text("txt")
                                        .margin(ui.spacing().button_padding),
                                )
                                .changed()
                            {
                                let new_ext = ext.trim_start_matches('.').to_string();
                                self.batch_output_extension = new_ext.clone();
                                self.settings.batch_last_extension = new_ext;
                            }
                        });
                    }
                });

                // --- Main Action Button ---
                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(8.0);
                    let is_running = self.batch_is_running;
                    let has_files = !self.batch_files.is_empty();
                    let has_keyfile = self.batch_keyfile.is_some();
                    let has_new_keyfile = self.batch_keyfile_new.is_some();

                    let enabled = !is_running
                        && match self.batch_mode {
                            BatchMode::Encrypt | BatchMode::Decrypt => has_files && has_keyfile,
                            BatchMode::Rotate => has_files && has_keyfile && has_new_keyfile,
                        };

                    let (label, icon) = if is_running {
                        let mode_icon = match self.batch_mode {
                            _ => "",
                        };
                        let verb = match self.batch_mode {
                            BatchMode::Encrypt => t!("batch.running_encrypt"),
                            BatchMode::Decrypt => t!("batch.running_decrypt"),
                            BatchMode::Rotate => t!("batch.running_rotate"),
                        };
                        (
                            format!(
                                "{} {}/{}",
                                verb, self.batch_progress_count, self.batch_total_count
                            ),
                            mode_icon,
                        )
                    } else {
                        match self.batch_mode {
                            BatchMode::Encrypt => (t!("batch.btn_encrypt").to_string(), ""),
                            BatchMode::Decrypt => (t!("batch.btn_decrypt").to_string(), ""),
                            BatchMode::Rotate => (t!("batch.btn_rotate").to_string(), ""),
                        }
                    };

                    ui.add_enabled_ui(enabled || is_running, |ui| {
                        let btn_size = egui::vec2(ui.available_width(), 36.0);

                        let button_response = if is_running {
                            let response = ui.add_sized(btn_size, egui::Button::new(""));
                            // Paint spinner + text centered within the button rect
                            let rect = response.rect;
                            let spinner_size = 14.0;
                            let spacing = 6.0;
                            let text_galley = ui.painter().layout_no_wrap(
                                label.clone(),
                                egui::TextStyle::Button.resolve(ui.style()),
                                ui.visuals().text_color(),
                            );
                            let total_w = spinner_size + spacing + text_galley.rect.width();
                            let start_x = rect.center().x - total_w / 2.0;
                            let center_y = rect.center().y;

                            // Draw spinner
                            let spinner_rect = egui::Rect::from_center_size(
                                egui::pos2(start_x + spinner_size / 2.0, center_y),
                                egui::vec2(spinner_size, spinner_size),
                            );
                            ui.put(spinner_rect, egui::Spinner::new().size(spinner_size));

                            // Draw text
                            let text_pos = egui::pos2(
                                start_x + spinner_size + spacing,
                                center_y - text_galley.rect.height() / 2.0,
                            );
                            ui.painter()
                                .galley(text_pos, text_galley, ui.visuals().text_color());
                            response
                        } else {
                            ui.add_sized(btn_size, egui::Button::new(format!("{} {}", icon, label)))
                        };

                        if button_response.clicked() && !is_running {
                            self.execute_batch_action();
                        }
                    });
                });
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
                    t!("batch.input_files_head", count = files_count).to_string()
                } else {
                    t!("batch.input_label").to_string()
                };

                if ui.available_width() > 320.0 {
                    crate::app_helpers::center_row(ui, |ui| {
                        self.render_heading(ui, &heading_text);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button(t!("batch.btn_clean")).clicked() {
                                self.batch_files.clear();
                            }
                            if ui.button(t!("batch.btn_add")).clicked() {
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
                    self.render_heading(ui, &heading_text);
                    crate::app_helpers::center_row(ui, |ui| {
                        if ui.button(t!("batch.btn_add")).clicked() {
                            if let Some(files) = rfd::FileDialog::new().pick_files() {
                                for file in files {
                                    if !self.batch_files.contains(&file) {
                                        self.batch_files.push(file);
                                    }
                                }
                            }
                        }
                        if ui.button(t!("batch.btn_clean")).clicked() {
                            self.batch_files.clear();
                        }
                    });
                }

                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.batch_files.is_empty() {
                            ui.label(t!("batch.files_none"));
                        } else {
                            let mut to_remove = None;
                            for (idx, file) in self.batch_files.iter().enumerate() {
                                let status = self
                                    .batch_file_access_cache
                                    .get(file)
                                    .cloned()
                                    .unwrap_or(KeyStatus::Unknown);
                                let text_color = match status {
                                    KeyStatus::Decryptable => {
                                        self.current_theme.colors.success_color()
                                    }
                                    KeyStatus::WrongKey => self.current_theme.colors.error_color(),
                                    KeyStatus::Unknown => self.current_theme.colors.warning_color(),
                                    _ => ui.visuals().text_color(),
                                };

                                crate::app_helpers::center_row(ui, |ui| {
                                    // Reserve button space first so long filenames can't push it off-screen.
                                    let btn_width = ui.spacing().interact_size.y
                                        + ui.spacing().button_padding.x * 2.0;
                                    let label_width = (ui.available_width()
                                        - btn_width
                                        - ui.spacing().item_spacing.x)
                                        .max(0.0);

                                    let file_name =
                                        file.file_name().unwrap_or_default().to_string_lossy();
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
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                crate::app_state::LogLevel::Error,
                t!("batch.log_err_keyfile", e = e).to_string(),
            ));
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
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                        crate::app_state::LogLevel::Warning,
                        t!(
                            "batch.log_enc_ignored_sen",
                            file = file.file_name().unwrap_or_default().to_string_lossy()
                        )
                        .to_string(),
                    ));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                        i + 1,
                        success,
                        failed,
                    ));
                    continue;
                }

                let output_dir = if let Some(d) = &batch_output_dir {
                    d.clone()
                } else {
                    file.parent().unwrap_or(Path::new(".")).to_path_buf()
                };

                let file_name = file.file_name().unwrap_or_default();
                let output_path =
                    output_dir.join(format!("{}.sen", file_name.to_string_lossy().to_string()));

                match std::fs::read(file) {
                    Ok(buffer) => {
                        // Binary Check: Heuristic check if it's actually text
                        if !crate::crypto::is_buffer_text(&buffer) {
                            failed += 1;
                            let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                crate::app_state::LogLevel::Warning,
                                t!(
                                    "batch.log_enc_ignored_binary",
                                    file = file.file_name().unwrap_or_default().to_string_lossy()
                                )
                                .to_string()
                                .to_string(),
                            ));
                            let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                                i + 1,
                                success,
                                failed,
                            ));
                            continue;
                        }

                        match encrypt_bytes(&buffer, &keyfile, &output_path) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Success,
                                    t!(
                                        "batch.log_enc_success",
                                        file =
                                            file.file_name().unwrap_or_default().to_string_lossy()
                                    )
                                    .to_string(),
                                ));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Error,
                                    t!(
                                        "batch.log_enc_failed",
                                        file =
                                            file.file_name().unwrap_or_default().to_string_lossy(),
                                        e = e
                                    )
                                    .to_string(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                            crate::app_state::LogLevel::Error,
                            t!(
                                "batch.log_enc_read_err",
                                file = file.file_name().unwrap_or_default().to_string_lossy(),
                                e = e
                            )
                            .to_string(),
                        ));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                    i + 1,
                    success,
                    failed,
                ));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(
                success, failed,
            ));
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
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                        crate::app_state::LogLevel::Warning,
                        t!(
                            "batch.log_dec_ignored_magic",
                            file = file.file_name().unwrap_or_default().to_string_lossy()
                        )
                        .to_string(),
                    ));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                        i + 1,
                        success,
                        failed,
                    ));
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
                        let content_str = String::from_utf8_lossy(&buffer);
                        let doc =
                            crate::history::DocumentWithHistory::from_file_content(&content_str);
                        let final_content = doc.current_content;

                        match std::fs::write(&output_path, final_content) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Success,
                                    t!(
                                        "batch.log_dec_success",
                                        file =
                                            file.file_name().unwrap_or_default().to_string_lossy()
                                    )
                                    .to_string(),
                                ));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Error,
                                    t!(
                                        "batch.log_dec_write_err",
                                        file = output_path
                                            .file_name()
                                            .unwrap_or_default()
                                            .to_string_lossy(),
                                        e = e
                                    )
                                    .to_string(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                            crate::app_state::LogLevel::Error,
                            t!(
                                "batch.log_dec_err",
                                file = file.file_name().unwrap_or_default().to_string_lossy(),
                                e = e
                            )
                            .to_string(),
                        ));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                    i + 1,
                    success,
                    failed,
                ));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(
                success, failed,
            ));
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
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                        crate::app_state::LogLevel::Warning,
                        t!(
                            "batch.log_rot_ignored_magic",
                            file = file.file_name().unwrap_or_default().to_string_lossy()
                        )
                        .to_string(),
                    ));
                    let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                        i + 1,
                        success,
                        failed,
                    ));
                    continue;
                }

                // Step 1: Decrypt with old keyfile
                match decrypt_bytes(&old_keyfile, file) {
                    Ok(buffer) => {
                        // Step 2: Re-encrypt with new keyfile, writing back to the same file
                        match encrypt_bytes(&buffer, &new_keyfile, file) {
                            Ok(_) => {
                                success += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Success,
                                    t!(
                                        "batch.log_rot_success",
                                        file =
                                            file.file_name().unwrap_or_default().to_string_lossy()
                                    )
                                    .to_string(),
                                ));
                            }
                            Err(e) => {
                                failed += 1;
                                let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                                    crate::app_state::LogLevel::Error,
                                    t!(
                                        "batch.log_rot_enc_err",
                                        file =
                                            file.file_name().unwrap_or_default().to_string_lossy(),
                                        e = e
                                    )
                                    .to_string(),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        failed += 1;
                        let _ = tx.send(crate::app_state::BatchProgressUpdate::Log(
                            crate::app_state::LogLevel::Error,
                            t!(
                                "batch.log_rot_dec_err",
                                file = file.file_name().unwrap_or_default().to_string_lossy(),
                                e = e
                            )
                            .to_string(),
                        ));
                    }
                }
                let _ = tx.send(crate::app_state::BatchProgressUpdate::Progress(
                    i + 1,
                    success,
                    failed,
                ));
            }
            let _ = tx.send(crate::app_state::BatchProgressUpdate::Finished(
                success, failed,
            ));
        });
    }
}
