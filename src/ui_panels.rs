use crate::app_state::{FileTreeEntry, LogLevel};
use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Render settings panel
    pub(crate) fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("⚙ Settings");

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(ui.available_height() - 60.0)
                .show(ui, |ui| {
                    ui.heading("Appearance");

                    // Theme selection
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        egui::ComboBox::from_id_salt("theme_selector")
                            .selected_text(&self.current_theme.name)
                            .show_ui(ui, |ui| {
                                for theme in &self.themes.clone() {
                                    if ui
                                        .selectable_label(
                                            theme.name == self.current_theme.name,
                                            &theme.name,
                                        )
                                        .clicked()
                                    {
                                        self.current_theme = theme.clone();
                                        self.settings.theme_name = theme.name.clone();
                                        self.apply_theme(ui.ctx());
                                        let _ = self.settings.save();
                                    }
                                }
                            });
                        if ui.button("🔄 Refresh").clicked() {
                            self.themes = crate::theme::load_themes();
                            self.log_info("Themes refreshed");
                        }
                    });

                    ui.separator();

                    // UI font family with keyboard navigation
                    ui.horizontal(|ui| {
                        ui.label("UI Font:");
                        let _response = egui::ComboBox::from_id_salt("ui_font_selector")
                            .selected_text(&self.available_fonts[self.ui_font_index])
                            .show_ui(ui, |ui| {
                                let mut changed = false;

                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                    if self.ui_font_index > 0 {
                                        self.ui_font_index -= 1;
                                        changed = true;
                                    }
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                    if self.ui_font_index < self.available_fonts.len() - 1 {
                                        self.ui_font_index += 1;
                                        changed = true;
                                    }
                                }

                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (idx, font) in self.available_fonts.iter().enumerate() {
                                            let is_selected = idx == self.ui_font_index;
                                            let response = ui.selectable_label(is_selected, font);
                                            if response.clicked() {
                                                self.ui_font_index = idx;
                                                changed = true;
                                            }

                                            if is_selected {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                        }
                                    });

                                if changed {
                                    self.settings.ui_font_family =
                                        self.available_fonts[self.ui_font_index].clone();
                                    let _ = self.settings.save();
                                    self.log_info(format!(
                                        "UI font changed to: {}",
                                        self.settings.ui_font_family
                                    ));
                                }
                            });
                    });

                    // UI font size
                    ui.horizontal(|ui| {
                        ui.label("UI Font Size:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.settings.ui_font_size)
                                    .speed(0.5)
                                    .range(8.0..=32.0),
                            )
                            .changed()
                        {
                            self.settings.validate_font_sizes();
                            let _ = self.settings.save();
                        }
                    });

                    ui.separator();

                    // Editor font family
                    ui.horizontal(|ui| {
                        ui.label("Editor Font:");
                        let _response = egui::ComboBox::from_id_salt("editor_font_selector")
                            .selected_text(&self.available_fonts[self.editor_font_index])
                            .show_ui(ui, |ui| {
                                let mut changed = false;

                                if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                                    if self.editor_font_index > 0 {
                                        self.editor_font_index -= 1;
                                        changed = true;
                                    }
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                                    if self.editor_font_index < self.available_fonts.len() - 1 {
                                        self.editor_font_index += 1;
                                        changed = true;
                                    }
                                }

                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        for (idx, font) in self.available_fonts.iter().enumerate() {
                                            let is_selected = idx == self.editor_font_index;
                                            let response = ui.selectable_label(is_selected, font);
                                            if response.clicked() {
                                                self.editor_font_index = idx;
                                                changed = true;
                                            }

                                            if is_selected {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                        }
                                    });

                                if changed {
                                    self.settings.editor_font_family =
                                        self.available_fonts[self.editor_font_index].clone();
                                    let _ = self.settings.save();
                                    self.log_info(format!(
                                        "Editor font changed to: {}",
                                        self.settings.editor_font_family
                                    ));
                                }
                            });
                    });

                    // Editor font size
                    ui.horizontal(|ui| {
                        ui.label("Editor Font Size:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.settings.editor_font_size)
                                    .speed(0.5)
                                    .range(8.0..=32.0),
                            )
                            .changed()
                        {
                            self.settings.validate_font_sizes();
                            let _ = self.settings.save();
                        }
                    });

                    ui.separator();
                    ui.heading("Keyfile");

                    if ui
                        .checkbox(
                            &mut self.settings.use_global_keyfile,
                            "Use global keyfile on startup",
                        )
                        .changed()
                    {
                        let _ = self.settings.save();
                    }

                    ui.horizontal(|ui| {
                        ui.label("Current:");
                        if let Some(path) = &self.settings.global_keyfile_path {
                            ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                        } else {
                            ui.label("None");
                        }
                    });

                    ui.horizontal(|ui| {
                        if ui.button("Set Global Keyfile").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.settings.global_keyfile_path = Some(path.clone());
                                if self.settings.use_global_keyfile {
                                    self.keyfile_path = Some(path);
                                }
                                let _ = self.settings.save();
                                self.log_info("Global keyfile set");
                            }
                        }
                        if ui.button("Clear").clicked() {
                            self.settings.global_keyfile_path = None;
                            let _ = self.settings.save();
                        }
                    });

                    ui.separator();
                    ui.heading("Editor");

                    if ui
                        .checkbox(&mut self.settings.show_line_numbers, "Show line numbers")
                        .changed()
                    {
                        let _ = self.settings.save();
                    }

                    if ui
                        .checkbox(
                            &mut self.settings.auto_snapshot_on_save,
                            "Auto-snapshot on save",
                        )
                        .changed()
                    {
                        let _ = self.settings.save();
                    }

                    ui.horizontal(|ui| {
                        ui.label("Default history limit:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.settings.max_history_length)
                                    .speed(1.0)
                                    .range(1..=1000),
                            )
                            .changed()
                        {
                            let _ = self.settings.save();
                        }
                    });

                    ui.label(
                        egui::RichText::new(format!(
                            "Current document: {}/{} entries",
                            self.document.get_history().len(),
                            self.settings.max_history_length
                        ))
                        .small()
                        .weak(),
                    );

                    ui.separator();
                    ui.heading("File Tree");

                    if ui
                        .checkbox(&mut self.settings.show_subfolders, "Show subfolders")
                        .changed()
                    {
                        let _ = self.settings.save();
                        self.refresh_file_tree();
                    }

                    ui.separator();
                    ui.add_space(4.0);
                });

            if ui.button("Close").clicked() {
                self.show_settings_panel = false;
            }
        });
    }

    /// Render history panel
    pub(crate) fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("📜 History");

            let history_len = self.document.get_history().len();
            let doc_max_limit = self.document.get_max_history_length();

            ui.horizontal(|ui| {
                ui.label("Max History for this file:");
                let mut temp_limit = doc_max_limit;
                if ui
                    .add(
                        egui::DragValue::new(&mut temp_limit)
                            .speed(1.0)
                            .range(1..=1000),
                    )
                    .changed()
                {
                    self.document.set_max_history_length(temp_limit);
                    self.is_modified = true;
                    self.log_info(format!("Document history limit set to {}", temp_limit));
                }
            });

            ui.label(
                egui::RichText::new(format!(
                    "Current: {}/{} entries",
                    history_len, doc_max_limit
                ))
                .small()
                .weak(),
            );

            ui.separator();

            if history_len > 0 {
                if ui.button("🗑 Clear All History").clicked() {
                    self.clear_all_history();
                    self.loaded_history_index = None;
                }
            }

            ui.separator();

            let history_entries: Vec<_> = self.document.get_history().to_vec();

            egui::ScrollArea::vertical().show(ui, |ui| {
                if history_entries.is_empty() {
                    ui.label("No history");
                } else {
                    for (index, entry) in history_entries.iter().enumerate().rev() {
                        let is_loaded = self.loaded_history_index == Some(index);

                        let frame = if is_loaded {
                            egui::Frame::none()
                                .fill(
                                    self.current_theme
                                        .colors
                                        .selection_color()
                                        .linear_multiply(0.3),
                                )
                                .stroke(egui::Stroke::new(
                                    2.0,
                                    self.current_theme.colors.cursor_color(),
                                ))
                                .inner_margin(8.0)
                                .rounding(4.0)
                        } else {
                            egui::Frame::none().inner_margin(4.0)
                        };

                        frame.show(ui, |ui| {
                            ui.group(|ui| {
                                if is_loaded {
                                    ui.label(
                                        egui::RichText::new("▶ LOADED")
                                            .color(self.current_theme.colors.cursor_color())
                                            .strong(),
                                    );
                                }

                                ui.label(format!("📅 {}", entry.display_timestamp()));
                                ui.label(format!("💾 {}", entry.display_size()));

                                if let Some(ref comment) = entry.comment {
                                    ui.label(
                                        egui::RichText::new(format!("💬 {}", comment))
                                            .italics()
                                            .weak(),
                                    );
                                }

                                ui.horizontal(|ui| {
                                    if ui.button("📂 Load").clicked() {
                                        self.load_history_version(index);
                                        self.loaded_history_index = Some(index);
                                    }

                                    if ui.button("🗑 Delete").clicked() {
                                        self.delete_history_entry(index);
                                        if self.loaded_history_index == Some(index) {
                                            self.loaded_history_index = None;
                                        }
                                    }
                                });
                            });
                        });
                        ui.add_space(4.0);
                    }
                }
            });
        });
    }

    /// Render debug panel
    pub(crate) fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Debug Log");

            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.debug_log.clear();
                }
                ui.label(format!("Entries: {}", self.debug_log.len()));
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &self.debug_log {
                        let color = match entry.level {
                            LogLevel::Info => ui.style().visuals.text_color(),
                            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 0),
                            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
                        };
                        ui.colored_label(color, entry.display());
                    }
                });
        });
    }

    /// Render file tree panel
    pub(crate) fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Files");

            if let Some(dir) = &self.file_tree_dir {
                ui.label(egui::RichText::new(dir.display().to_string()).small());
                ui.separator();

                let available_width = ui.available_width();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_max_width(available_width);

                        for entry in &self.file_tree_entries.clone() {
                            match entry {
                                FileTreeEntry::Directory(path) => {
                                    let is_parent = self
                                        .file_tree_dir
                                        .as_ref()
                                        .and_then(|d| d.parent())
                                        .map(|p| p == path)
                                        .unwrap_or(false);

                                    let display_name = if is_parent {
                                        "📁 ..".to_string()
                                    } else {
                                        format!(
                                            "📁 {}",
                                            path.file_name().unwrap_or_default().to_string_lossy()
                                        )
                                    };

                                    if ui.button(display_name).clicked() {
                                        self.change_directory(path.clone());
                                    }
                                }
                                FileTreeEntry::File(path) => {
                                    let filename =
                                        path.file_name().unwrap_or_default().to_string_lossy();
                                    if ui.button(format!("📄 {}", filename)).clicked() {
                                        self.open_file(path.clone());
                                    }
                                }
                            }
                        }
                    });
            } else {
                ui.label("No directory opened");
                if ui.button("Open Directory").clicked() {
                    self.open_directory();
                }
            }
        });
    }

    /// Render theme editor panel
    pub(crate) fn render_theme_editor_panel(&mut self, ui: &mut egui::Ui) {
        let mut theme_to_save: Option<crate::theme::Theme> = None;
        let mut should_close = false;
        let mut should_apply = false;
        let mut should_reset = false;

        ui.vertical(|ui| {
            ui.heading("🎨 Theme Editor");

            if let Some(ref mut theme) = self.editing_theme {
                let mut theme_changed = false;

                ui.label("Theme Name:");
                ui.text_edit_singleline(&mut theme.name);

                ui.horizontal(|ui| {
                    ui.label("Base Scheme:");
                    egui::ComboBox::from_id_salt("color_scheme_selector")
                        .selected_text(format!("{:?}", theme.color_scheme))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Dark),
                                    "Dark",
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Dark;
                                theme_changed = true;
                            }

                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Light),
                                    "Light",
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Light;
                                theme_changed = true;
                            }
                        });
                });

                ui.separator();

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 100.0)
                    .show(ui, |ui| {
                        ui.heading("Colors");
                        ui.add_space(4.0);

                        egui::Grid::new("all_theme_colors_grid")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .striped(false)
                            .show(ui, |ui| {
                                // Background
                                ui.label("Background:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.background[0],
                                    theme.colors.background[1],
                                    theme.colors.background[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.background = [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Foreground
                                ui.label("Foreground:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.foreground[0],
                                    theme.colors.foreground[1],
                                    theme.colors.foreground[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.foreground = [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Panel Background
                                ui.label("Panel Background:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.panel_background[0],
                                    theme.colors.panel_background[1],
                                    theme.colors.panel_background[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.panel_background =
                                        [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();

                                ui.label("");
                                ui.label("");
                                ui.end_row();

                                // Selection Background
                                ui.label("Selection Background:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.selection_background[0],
                                    theme.colors.selection_background[1],
                                    theme.colors.selection_background[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.selection_background =
                                        [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Cursor
                                ui.label("Cursor Color:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.cursor[0],
                                    theme.colors.cursor[1],
                                    theme.colors.cursor[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.cursor = [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Icon Hover
                                ui.label("Icon Hover Tint:");
                                let mut color = egui::Color32::from_rgb(
                                    theme.colors.icon_hover[0],
                                    theme.colors.icon_hover[1],
                                    theme.colors.icon_hover[2],
                                );
                                if ui.color_edit_button_srgba(&mut color).changed() {
                                    theme.colors.icon_hover = [color.r(), color.g(), color.b()];
                                    theme_changed = true;
                                }
                                ui.end_row();
                            });
                    });

                if theme_changed {
                    theme.apply(ui.ctx());
                }

                ui.separator();

                ui.vertical(|ui| {
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui.button("💾 Save Theme").clicked() {
                            theme_to_save = Some(theme.clone());
                        }

                        if ui.button("✓ Apply").clicked() {
                            should_apply = true;
                        }
                    });

                    let current_scheme = theme.color_scheme;

                    ui.horizontal(|ui| {
                        let reset_text = match current_scheme {
                            crate::theme::ColorScheme::Light => "🔄 Reset to Light",
                            crate::theme::ColorScheme::Dark => "🔄 Reset to Dark",
                        };

                        if ui.button(reset_text).clicked() {
                            should_reset = true;
                        }

                        if ui.button("✖ Close").clicked() {
                            should_close = true;
                        }
                    });
                });
            } else {
                ui.label("No theme being edited");
            }
        });

        // Execute actions
        if should_reset {
            if let Some(ref mut theme) = self.editing_theme {
                *theme = match theme.color_scheme {
                    crate::theme::ColorScheme::Light => crate::theme::Theme::light(),
                    crate::theme::ColorScheme::Dark => crate::theme::Theme::dark(),
                };
                theme.apply(ui.ctx());
            }
        }

        if should_apply {
            if let Some(theme) = &self.editing_theme {
                self.current_theme = theme.clone();
                self.settings.theme_name = theme.name.clone();
                self.current_theme.apply(ui.ctx());
                let _ = self.settings.save();
                self.status_message = "Theme applied".to_string();
                self.log_info("Theme applied");
            }
        }

        if let Some(theme) = theme_to_save {
            match crate::theme::save_theme(&theme) {
                Ok(_) => {
                    self.current_theme = theme.clone();
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.themes = crate::theme::load_themes();
                    self.status_message = format!("✓ Theme '{}' saved", theme.name);
                    self.log_info(format!("Theme '{}' saved successfully", theme.name));
                }
                Err(e) => {
                    self.status_message = format!("Error saving theme: {}", e);
                    self.log_error(format!("Failed to save theme: {}", e));
                }
            }
        }

        if should_close {
            self.show_theme_editor = false;
            self.editing_theme = None;
            self.current_theme.apply(ui.ctx());
        }
    }
}
