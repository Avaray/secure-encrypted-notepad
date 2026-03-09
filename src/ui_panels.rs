use crate::app_state::{FileTreeEntry, LogLevel};
use crate::history::HistoryEntry;
use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Render settings panel
    pub(crate) fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.heading("⚙ Settings");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("📁").on_hover_text("Open settings folder").clicked() {
                        if let Some(path) = crate::settings::Settings::get_config_dir() {
                            #[cfg(target_os = "windows")]
                            {
                                let _ = std::process::Command::new("explorer").arg(path).spawn();
                            }
                            #[cfg(target_os = "linux")]
                            {
                                let _ = std::process::Command::new("xdg-open").arg(path).spawn();
                            }
                            #[cfg(target_os = "macos")]
                            {
                                let _ = std::process::Command::new("open").arg(path).spawn();
                            }
                        }
                    }
                });
            });

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
                                        self.editing_theme = Some(theme.clone()); // Sync theme editor
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
                                        let current_time = ui.input(|i| i.time);
                                        let is_first_frame = ui.ctx().data_mut(|d| {
                                            let last_time = d.get_temp::<f64>(ui.id());
                                            d.insert_temp(ui.id(), current_time);
                                            match last_time {
                                                Some(last) => current_time > last + 0.1,
                                                None => true,
                                            }
                                        });

                                        for (idx, font) in self.available_fonts.iter().enumerate() {
                                            let is_selected = idx == self.ui_font_index;
                                            let response = ui.selectable_label(is_selected, font);
                                            
                                            if is_selected && is_first_frame {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                            
                                            if response.clicked() || (is_selected && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                                                self.ui_font_index = idx;
                                                changed = true;
                                                ui.close_kind(egui::UiKind::Menu);
                                            }
                                        }
                                    });

                                if changed {
                                    self.settings.ui_font_family =
                                        self.available_fonts[self.ui_font_index].clone();
                                    let _ = self.settings.save();
                                    self.style_dirty = true;
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
                                    .range(8.0..=128.0),
                            )
                            .changed()
                        {
                            self.settings.validate_font_sizes();
                            let _ = self.settings.save();
                            self.style_dirty = true;
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
                                        let current_time = ui.input(|i| i.time);
                                        let is_first_frame = ui.ctx().data_mut(|d| {
                                            let last_time = d.get_temp::<f64>(ui.id());
                                            d.insert_temp(ui.id(), current_time);
                                            match last_time {
                                                Some(last) => current_time > last + 0.1,
                                                None => true,
                                            }
                                        });

                                        for (idx, font) in self.available_fonts.iter().enumerate() {
                                            let is_selected = idx == self.editor_font_index;
                                            let response = ui.selectable_label(is_selected, font);
                                            
                                            if is_selected && is_first_frame {
                                                response.scroll_to_me(Some(egui::Align::Center));
                                            }
                                            
                                            if response.clicked() || (is_selected && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                                                self.editor_font_index = idx;
                                                changed = true;
                                                ui.close_kind(egui::UiKind::Menu);
                                            }
                                        }
                                    });

                                if changed {
                                    self.settings.editor_font_family =
                                        self.available_fonts[self.editor_font_index].clone();
                                    let _ = self.settings.save();
                                    self.style_dirty = true;
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
                                    .range(8.0..=128.0),
                            )
                            .changed()
                        {
                            self.settings.validate_font_sizes();
                            let _ = self.settings.save();
                            self.style_dirty = true;
                        }
                    });

                    // Toolbar icon size
                    ui.horizontal(|ui| {
                        ui.label("Toolbar Icon Size:");
                        if ui
                            .add(
                                egui::DragValue::new(&mut self.settings.toolbar_icon_size)
                                    .speed(1.0)
                                    .range(12.0..=64.0),
                            )
                            .changed()
                        {
                            let _ = self.settings.save();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("Toolbar Position:");
                        let mut changed = false;
                        changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Top, "Top").changed();
                        changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Left, "Left").changed();
                        changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Right, "Right").changed();
                        if changed {
                            let _ = self.settings.save();
                        }
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Tab Size:");
                        if ui
                            .add(egui::DragValue::new(&mut self.settings.tab_size).range(2..=8))
                            .changed()
                        {
                            let _ = self.settings.save();
                        }
                    });

                    if ui
                        .checkbox(&mut self.settings.use_spaces_for_tabs, "Use spaces for tabs")
                        .changed()
                    {
                        let _ = self.settings.save();
                    }

                    if ui.checkbox(&mut self.settings.word_wrap, "Word wrap").changed() {
                        let _ = self.settings.save();
                    }

                    if ui
                        .checkbox(&mut self.settings.start_maximized, "Start Maximized")
                        .changed()
                    {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(
                                self.settings.start_maximized,
                            ));
                        let _ = self.settings.save();
                    }

                    ui.separator();
                    ui.heading("Reliability");

                    ui.group(|ui| {
                        ui.label("Auto Save");
                        if ui
                            .checkbox(&mut self.settings.auto_save_enabled, "Enable Auto-save")
                            .changed()
                        {
                            let _ = self.settings.save();
                        }

                        ui.horizontal(|ui| {
                            ui.label("Interval (seconds):");
                            if ui
                                .add(
                                    egui::DragValue::new(&mut self.settings.auto_save_interval_secs)
                                        .range(5..=3600),
                                )
                                .changed()
                            {
                                let _ = self.settings.save();
                            }
                        });
                    });

                    ui.add_space(4.0);



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
                            if self.settings.show_keyfile_path {
                                ui.label(path.file_name().unwrap_or_default().to_string_lossy());
                            } else {
                                ui.label("Secured");
                            }
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
                            self.settings.keyfile_path_encrypted = None;
                            let _ = self.settings.save();
                        }
                    });

                    if ui.checkbox(&mut self.settings.show_keyfile_path, "Show keyfile paths globally").changed() {
                        let _ = self.settings.save();
                    }

                    ui.separator();
                    ui.separator();
                    ui.heading("Editor");

                    if ui
                        .checkbox(&mut self.settings.show_line_numbers, "Show line numbers")
                        .changed()
                    {
                        let _ = self.settings.save();
                    }

                    if ui
                        .checkbox(&mut self.settings.show_whitespace, "Show special symbols")
                        .on_hover_text("Displays spaces as dots, tabs as arrows, and returns as a return arrow.")
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

                    // Cursor settings
                    ui.horizontal(|ui| {
                        ui.label("Cursor Shape:");
                        egui::ComboBox::from_id_salt("cursor_shape_combo")
                            .selected_text(format!("{:?}", self.settings.cursor_shape))
                            .show_ui(ui, |ui| {
                                if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Bar, "Bar").changed() {
                                    let _ = self.settings.save();
                                    self.style_dirty = true;
                                }
                                if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Block, "Block").changed() {
                                    let _ = self.settings.save();
                                    self.style_dirty = true;
                                }
                                if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Underscore, "Underscore").changed() {
                                    let _ = self.settings.save();
                                    self.style_dirty = true;
                                }
                            });
                    });

                    if ui.checkbox(&mut self.settings.cursor_blink, "Cursor blinking").changed() {
                        let _ = self.settings.save();
                        self.style_dirty = true;
                    }
                    // Max lines
                    ui.horizontal(|ui| {
                        ui.label("Max Lines Limit:");
                        let mut limit_val = self.settings.max_lines;
                        if ui
                            .add(
                                egui::DragValue::new(&mut limit_val)
                                    .speed(10.0)
                                    .range(0..=1000000),
                            )
                            .changed()
                        {
                            self.settings.max_lines = limit_val;
                            let _ = self.settings.save();
                        }
                        if self.settings.max_lines == 0 {
                            ui.label(egui::RichText::new("(No limit)").italics().weak());
                        }
                    })
                    .response
                    .on_hover_text("Maximum number of lines allowed in the editor. Set to 0 to disable the limit.");

                    // History capacity
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

                    // Starting directory setting
                    ui.add_space(4.0);
                    ui.label("Starting directory:");
                    if let Some(ref dir) = self.settings.file_tree_starting_dir {
                        ui.label(
                            egui::RichText::new(dir.display().to_string())
                                .small()
                                .weak(),
                        );
                    } else {
                        ui.label(egui::RichText::new("Not set (uses last opened)").small().weak());
                    }
                    ui.horizontal(|ui| {
                        if ui.button("📁 Browse").clicked() {
                            if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                                self.settings.file_tree_starting_dir = Some(dir.clone());
                                self.file_tree_dir = Some(dir);
                                let _ = self.settings.save();
                                self.refresh_file_tree();
                                self.log_info("Starting directory set");
                            }
                        }
                        if self.settings.file_tree_starting_dir.is_some() {
                            if ui.button("✕ Clear").clicked() {
                                self.settings.file_tree_starting_dir = None;
                                self.settings.file_tree_dir_encrypted = None;
                                let _ = self.settings.save();
                                self.log_info("Starting directory cleared");
                            }
                        }
                    });

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
        // Zbierz wszystkie dane PRZED closure
        let visible_history: Vec<(usize, HistoryEntry)> = self
            .document
            .get_visible_history()
            .into_iter()
            .map(|(idx, entry)| (idx, entry.clone()))
            .collect();

        let history_len = visible_history.len();
        let doc_max_limit = self.document.get_max_history_length();

        ui.vertical(|ui| {
            ui.heading("📜 History");

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

            egui::ScrollArea::vertical().show(ui, |ui| {
                if history_len == 0 {
                    ui.label("No history");
                } else {
                    // Iteruj po sklonowanych danych
                    for (original_index, entry) in visible_history.iter().rev() {
                        let is_loaded = self.loaded_history_index == Some(*original_index);

                        let frame = if is_loaded {
                            egui::Frame::NONE
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
                                .inner_margin(8)
                                .corner_radius(4.0)
                        } else {
                            egui::Frame::NONE.inner_margin(4)
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
                                        self.load_history_version(*original_index);
                                        self.loaded_history_index = Some(*original_index);
                                    }

                                    if ui.button("🗑 Delete").clicked() {
                                        self.delete_history_entry(*original_index);
                                        if self.loaded_history_index == Some(*original_index) {
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
            // Ensure panel has width to be grabbed
            ui.set_min_width(ui.available_width());
            
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
        let mut should_reset = false;

        ui.vertical(|ui| {
            ui.heading("🎨 Theme Editor");

            // Top bar: Theme selector and actions
            ui.horizontal(|ui| {
                let current_name = self.editing_theme.as_ref().map(|t| t.name.clone()).unwrap_or_default();
                
                egui::ComboBox::from_id_salt("theme_editor_selector")
                    .selected_text(&current_name)
                    .show_ui(ui, |ui| {
                        for theme in &self.themes {
                            if ui.selectable_label(theme.name == current_name, &theme.name).clicked() {
                                self.editing_theme = Some(theme.clone());
                                self.current_theme = theme.clone();
                                self.settings.theme_name = theme.name.clone();
                                self.apply_theme(ui.ctx());
                                let _ = self.settings.save();
                            }
                        }
                    });

                if ui.button("➕ New").clicked() {
                    let mut new_theme = self.current_theme.clone();
                    new_theme.name = format!("{} (Copy)", new_theme.name);
                    self.editing_theme = Some(new_theme);
                }

                // Delete button with confirmation
                if let Some(theme) = &self.editing_theme {
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    if !is_builtin {
                        if ui.button("🗑 Delete").clicked() {
                             // This actually needs a way to confirm. For now, simple delete.
                             // In a real app we'd show a modal. 
                             // Since we don't have modals easily, we'll just delete and select default.
                             let _ = crate::theme::delete_theme(&theme.name);
                             self.themes = crate::theme::load_themes(); // Reload
                             self.editing_theme = Some(crate::theme::Theme::dark()); // Reset to safe default
                        }
                    }
                }
            });

            ui.separator();

            if let Some(ref mut theme) = self.editing_theme {
                let mut theme_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut theme.name);
                });

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
                    .max_height(ui.available_height() - 80.0)
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
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Foreground
                                ui.label("Foreground:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.foreground)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Editor Foreground
                                ui.label("Editor Foreground:");
                                ui.horizontal(|ui| {
                                    let mut editor_fg = theme
                                        .colors
                                        .editor_foreground
                                        .unwrap_or(theme.colors.foreground);
                                    if ui.color_edit_button_srgb(&mut editor_fg).changed() {
                                        theme.colors.editor_foreground = Some(editor_fg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.editor_foreground.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to match UI Foreground").clicked() {
                                            theme.colors.editor_foreground = None;
                                            theme_changed = true;
                                        }
                                    }
                                    if theme.colors.editor_foreground.is_none() {
                                         ui.weak("(Same as UI)");
                                    }
                                });
                                ui.end_row();

                                // Panel Background
                                ui.label("Panel Background:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.panel_background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                ui.label("");
                                ui.label("");
                                ui.end_row();

                                // Selection Background
                                ui.label("Selection Background:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.selection_background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Cursor
                                ui.label("Cursor Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.cursor)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Icon Hover
                                ui.label("Icon Hover Tint:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.icon_hover)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Icon Default (New)
                                ui.label("Icon Default Tint:");
                                let mut icon_def = theme.colors.icon_color.unwrap_or(
                                    if theme.color_scheme == crate::theme::ColorScheme::Dark { [200, 200, 200] } else { [80, 80, 80] }
                                );
                                if ui.color_edit_button_srgb(&mut icon_def).changed() {
                                    theme.colors.icon_color = Some(icon_def);
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Highlight (New)
                                ui.label("Search Highlight:");
                                let mut highlight = theme.colors.highlight.unwrap_or(
                                    theme.colors.cursor // fallback
                                );
                                ui.horizontal(|ui| {
                                    if ui.color_edit_button_srgb(&mut highlight).changed() {
                                        theme.colors.highlight = Some(highlight);
                                        theme_changed = true;
                                    }
                                    if theme.colors.highlight.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.highlight = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Button Background
                                ui.label("Button Background:");
                                ui.horizontal(|ui| {
                                    let mut bg = theme.colors.button_bg.unwrap_or([60, 60, 60]); // Approx default
                                    if ui.color_edit_button_srgb(&mut bg).changed() {
                                        theme.colors.button_bg = Some(bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_bg.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.button_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Button Hover Background
                                ui.label("Button Hover:");
                                ui.horizontal(|ui| {
                                    let mut h_bg = theme.colors.button_hover_bg.unwrap_or(theme.colors.background); // Fallback
                                    if ui.color_edit_button_srgb(&mut h_bg).changed() {
                                        theme.colors.button_hover_bg = Some(h_bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_hover_bg.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.button_hover_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Button Active Background
                                ui.label("Button Active:");
                                ui.horizontal(|ui| {
                                    let mut a_bg = theme.colors.button_active_bg.unwrap_or(theme.colors.background);
                                    if ui.color_edit_button_srgb(&mut a_bg).changed() {
                                        theme.colors.button_active_bg = Some(a_bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_active_bg.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.button_active_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Button Foreground
                                ui.label("Button Text:");
                                ui.horizontal(|ui| {
                                    let mut fg = theme.colors.button_fg.unwrap_or(theme.colors.foreground);
                                    if ui.color_edit_button_srgb(&mut fg).changed() {
                                        theme.colors.button_fg = Some(fg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_fg.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.button_fg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Separator
                                ui.label("Separator:");
                                ui.horizontal(|ui| {
                                    let mut sep = theme.colors.separator.unwrap_or([80, 80, 80]);
                                    if ui.color_edit_button_srgb(&mut sep).changed() {
                                        theme.colors.separator = Some(sep);
                                        theme_changed = true;
                                    }
                                    if theme.colors.separator.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.separator = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                // Line Number Color
                                ui.label("Line Numbers:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.line_number)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Comment Color
                                ui.label("Comments:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.comment)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                // Whitespace Symbols Color
                                ui.label("Whitespace Symbols:");
                                ui.horizontal(|ui| {
                                    let mut ws = theme.colors.whitespace_symbols.unwrap_or_else(|| {
                                        let c = theme.colors.comment;
                                        [(c[0] as f32 * 0.4) as u8, (c[1] as f32 * 0.4) as u8, (c[2] as f32 * 0.4) as u8]
                                    });
                                    if ui.color_edit_button_srgb(&mut ws).changed() {
                                        theme.colors.whitespace_symbols = Some(ws);
                                        theme_changed = true;
                                    }
                                    if theme.colors.whitespace_symbols.is_some() {
                                        if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                            theme.colors.whitespace_symbols = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();

                                ui.label("");
                                ui.label("");
                                ui.end_row();

                                // Status colors section
                                ui.label("Success Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.success)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                ui.label("Warning Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.warning)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();

                                ui.label("Error Color:");
                                if ui.color_edit_button_srgb(&mut theme.colors.error).changed() {
                                    theme_changed = true;
                                }
                                ui.end_row();
                            });
                    });

                // KLUCZOWA ZMIANA: Synchronizuj z current_theme natychmiast!
                if theme_changed {
                    theme.apply(ui.ctx());
                    self.current_theme = theme.clone();
                }

                ui.separator();

                ui.horizontal_wrapped(|ui| {
                    if ui.button("💾 Save Theme").clicked() {
                        theme_to_save = Some(theme.clone());
                    }

                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    let reset_text = if is_builtin {
                         format!("↺ Reset to Default ({:?})", theme.color_scheme)
                    } else {
                         "↺ Reset to Saved".to_string()
                    };
                    
                    if ui.button(reset_text).clicked() {
                        should_reset = true;
                    }
                });
            } else {
                ui.label("No theme being edited");
            }
        });

        // Execute actions
        if should_reset {
            if let Some(ref mut theme) = self.editing_theme {
                let is_builtin = theme.name == "Dark" || theme.name == "Light";
                if is_builtin {
                    // Reset to factory defaults
                    *theme = match theme.color_scheme {
                        crate::theme::ColorScheme::Light => crate::theme::Theme::light(),
                        crate::theme::ColorScheme::Dark => crate::theme::Theme::dark(),
                    };
                } else {
                    // Reset to saved file
                    let all_themes = crate::theme::load_themes();
                    if let Some(saved) = all_themes.iter().find(|t| t.name == theme.name) {
                        *theme = saved.clone();
                    } else {
                        // If not found (e.g. unsaved new theme), maybe reset to parent scheme?
                        // Or just keep as is? Let's reset to scheme default as fallback.
                        *theme = match theme.color_scheme {
                            crate::theme::ColorScheme::Light => crate::theme::Theme::light(),
                            crate::theme::ColorScheme::Dark => crate::theme::Theme::dark(),
                        };
                        theme.name = theme.name.clone(); // Keep the name if it was a new custom theme
                        // Actually if it's new and unsaved, "Reset to Saved" is ambiguous.
                        // But usually this handles the "I messed up editing 'Ocean', let me revert to what's on disk" case.
                    }
                }
                theme.apply(ui.ctx());
                self.current_theme = theme.clone();
            }
        }

        if let Some(theme) = theme_to_save {
            match crate::theme::save_theme(&theme) {
                Ok(_) => {
                    self.current_theme = theme.clone();
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.themes = crate::theme::load_themes();
                    self.status_message = format!("OK: Theme saved: {}", theme.name);
                    self.log_info(format!("Theme saved successfully: {}", theme.name));
                }
                Err(e) => {
                    self.status_message = format!("Error saving theme: {}", e);
                    self.log_error(format!("Failed to save theme: {}", e));
                }
            }
        }
    }
}
