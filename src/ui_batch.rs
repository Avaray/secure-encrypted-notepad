use crate::EditorApp;
use eframe::egui;


impl EditorApp {
    pub(crate) fn render_batch_converter_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_batch_converter;
        
        egui::Window::new("Batch Converter")
            .open(&mut open)
            .resizable(true)
            .default_width(600.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                ui.heading("Batch Encryption / Decryption");
                ui.label("Convert multiple files at once using a keyfile.");
                
                ui.separator();
                
                // Keyfile Selection
                ui.heading("1. Keyfile");
                ui.horizontal(|ui| {
                    if let Some(path) = &self.batch_keyfile {
                         ui.label(egui::RichText::new(format!("🔑 {}", path.display())).color(self.current_theme.colors.success_color()));
                    } else {
                         ui.label(egui::RichText::new("No keyfile selected").color(self.current_theme.colors.warning_color()));
                    }
                    
                    if ui.button("Select Keyfile...").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
                            self.batch_keyfile = Some(path);
                        }
                    }
                });
                
                ui.separator();
                
                // File List
                ui.heading("2. Input Files");
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
                
                egui::ScrollArea::vertical()
                    .max_height(200.0)
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

                 ui.separator();
                 
                 // Output Directory
                 ui.heading("3. Output Directory");
                 ui.horizontal(|ui| {
                     if let Some(path) = &self.batch_output_dir {
                         ui.label(format!("📁 {}", path.display()));
                     } else {
                         ui.label("Same as input files (default)");
                     }
                     
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
                 
                 ui.separator();
                 
                 // Actions
                 ui.horizontal(|ui| {
                     let enabled = !self.batch_files.is_empty() && self.batch_keyfile.is_some();
                     
                     if ui.add_enabled(enabled, egui::Button::new("🔒 Encrypt All")).clicked() {
                         self.status_message = "Batch encryption started...".to_string();
                         // TODO: Implement batch encryption logic
                         self.log_info("Batch encryption requested");
                     }
                     
                     if ui.add_enabled(enabled, egui::Button::new("🔓 Decrypt All")).clicked() {
                         self.status_message = "Batch decryption started...".to_string();
                         // TODO: Implement batch decryption logic
                         self.log_info("Batch decryption requested");
                     }
                 });
            });
            
        self.show_batch_converter = open;
    }
}
