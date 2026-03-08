use crate::app_state::{FileTreeEntry, LogEntry, LogLevel};
use crate::EditorApp;

impl EditorApp {
    /// Logging functions
    pub(crate) fn log_info(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Info, message.into()));
        if self.debug_log.len() > 1000 {
            self.debug_log.drain(0..100);
        }
    }

    pub(crate) fn log_warning(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Warning, message.into()));
    }

    pub(crate) fn log_error(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Error, message.into()));
    }

    /// Load custom fonts into egui
    pub(crate) fn load_custom_fonts(&self, ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Load UI font if custom
        if !self.settings.ui_font_family.contains("(Default)") {
            if let Some(font_data) = crate::fonts::load_font_data(&self.settings.ui_font_family) {
                fonts.font_data.insert(
                    self.settings.ui_font_family.clone(),
                    egui::FontData::from_owned(font_data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Proportional)
                    .or_default()
                    .insert(0, self.settings.ui_font_family.clone());
            }
        }

        // Load Editor font if custom
        if !self.settings.editor_font_family.contains("(Default)") {
            if let Some(font_data) = crate::fonts::load_font_data(&self.settings.editor_font_family)
            {
                fonts.font_data.insert(
                    self.settings.editor_font_family.clone(),
                    egui::FontData::from_owned(font_data).into(),
                );
                fonts
                    .families
                    .entry(egui::FontFamily::Monospace)
                    .or_default()
                    .insert(0, self.settings.editor_font_family.clone());
            }
        }

        ctx.set_fonts(fonts);
    }

    /// Update UI style + fonts based on settings
    pub(crate) fn apply_style(&self, ctx: &egui::Context) {
        self.load_custom_fonts(ctx);

        ctx.style_mut(|style| {
            use egui::{FontFamily, FontId, TextStyle};

            let ui_family = if self.settings.ui_font_family.contains("(Default)") {
                if self.settings.ui_font_family.contains("Monospace") {
                    FontFamily::Monospace
                } else {
                    FontFamily::Proportional
                }
            } else {
                FontFamily::Proportional
            };

            let editor_family = if self.settings.editor_font_family.contains("(Default)") {
                if self.settings.editor_font_family.contains("Proportional") {
                    FontFamily::Proportional
                } else {
                    FontFamily::Monospace
                }
            } else {
                FontFamily::Monospace
            };

            // Set text styles
            style.text_styles = [
                (
                    TextStyle::Heading,
                    FontId::new(self.settings.ui_font_size + 4.0, ui_family.clone()),
                ),
                (
                    TextStyle::Body,
                    FontId::new(self.settings.ui_font_size, ui_family.clone()),
                ),
                (
                    TextStyle::Monospace,
                    FontId::new(self.settings.editor_font_size, editor_family),
                ),
                (
                    TextStyle::Button,
                    FontId::new(self.settings.ui_font_size, ui_family.clone()),
                ),
                (
                    TextStyle::Small,
                    FontId::new(self.settings.ui_font_size - 4.0, ui_family),
                ),
            ]
            .into();

            // ✅ POPRAWKA: Ustaw spacing aby tekst był wycentrowany w pionie
            // Button padding - zwiększ padding w pionie (drugi parametr)
            style.spacing.button_padding = egui::vec2(6.0, 4.0);

            // Item spacing - zwiększ spacing w pionie
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);

            // Interact size - wysokość interaktywnych elementów
            style.spacing.interact_size.y = self.settings.ui_font_size + 8.0;

            // Opcjonalnie: window padding
            style.spacing.window_margin = egui::Margin::same(8);
            style.spacing.item_spacing.y = 6.0; // spacing między elementami
        });
    }

    /// Apply current theme
    pub(crate) fn apply_theme(&self, ctx: &egui::Context) {
        self.current_theme.apply(ctx);
    }

    /// Toggle comment on selected lines or current line
    pub(crate) fn toggle_comment_lines(&mut self) {
        let lines_to_toggle = self.get_lines_in_selection();
        if lines_to_toggle.is_empty() {
            return;
        }

        let text = &mut self.document.current_content;
        if text.is_empty() {
            return;
        }

        // POPRAWKA: Użyj split('\n') zamiast lines() aby zachować końcowe puste linie
        let lines: Vec<&str> = text.split('\n').collect();

        // Check if all selected lines are commented
        let all_commented = lines_to_toggle.iter().all(|&line_idx| {
            if line_idx < lines.len() {
                let trimmed = lines[line_idx].trim_start();
                trimmed.starts_with("//")
            } else {
                false
            }
        });

        let mut new_lines: Vec<String> = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            if lines_to_toggle.contains(&idx) {
                let trimmed = line.trim_start();

                if all_commented {
                    // Uncomment
                    if trimmed.starts_with("// ") {
                        let uncommented = trimmed.strip_prefix("// ").unwrap();
                        let indent = line.len() - trimmed.len();
                        new_lines.push(format!("{}{}", " ".repeat(indent), uncommented));
                    } else if trimmed.starts_with("//") {
                        let uncommented = trimmed.strip_prefix("//").unwrap();
                        let indent = line.len() - trimmed.len();
                        new_lines.push(format!("{}{}", " ".repeat(indent), uncommented));
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    // Comment
                    if !trimmed.starts_with("//") {
                        let indent_count = line.len() - trimmed.len();
                        new_lines.push(format!("{}// {}", " ".repeat(indent_count), trimmed));
                    } else {
                        new_lines.push(line.to_string());
                    }
                }
            } else {
                new_lines.push(line.to_string());
            }
        }

        // POPRAWKA: Użyj join('\n') żeby zachować strukturę
        *text = new_lines.join("\n");
        self.is_modified = true;

        if lines_to_toggle.len() == 1 {
            self.log_info(format!(
                "Toggled comment on line {}",
                lines_to_toggle[0] + 1
            ));
        } else {
            self.log_info(format!(
                "Toggled comments on {} lines",
                lines_to_toggle.len()
            ));
        }
    }

    /// Get line indices that should be toggled (0-indexed)
    pub(crate) fn get_lines_in_selection(&self) -> Vec<usize> {
        let text = &self.document.current_content;

        if let Some(ref range) = self.text_cursor_range {
            let mut current_pos = 0;
            let mut line_indices = std::collections::HashSet::new();

            for (line_idx, line) in text.lines().enumerate() {
                let line_end = current_pos + line.len();
                if current_pos <= range.end && line_end >= range.start {
                    line_indices.insert(line_idx);
                }
                current_pos = line_end + 1;
            }

            let mut result: Vec<usize> = line_indices.into_iter().collect();
            result.sort();
            result
        } else {
            if let Some(line_num) = self.highlighted_line {
                if line_num > 0 {
                    vec![line_num - 1]
                } else {
                    vec![0]
                }
            } else {
                vec![0]
            }
        }
    }

    /// Refresh file tree entries
    pub(crate) fn refresh_file_tree(&mut self) {
        self.file_tree_entries.clear();
        let dir_opt = self.file_tree_dir.clone();

        if let Some(dir) = dir_opt {
            self.log_info(format!("Refreshing file tree for: {}", dir.display()));
            match std::fs::read_dir(&dir) {
                Ok(entries) => {
                    let mut folders = Vec::new();
                    let mut files = Vec::new();

                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() && self.settings.show_subfolders {
                            folders.push(FileTreeEntry::Directory(path));
                        } else if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "sen" {
                                    files.push(FileTreeEntry::File(path));
                                }
                            }
                        }
                    }

                    folders.sort_by(|a, b| {
                        if let (FileTreeEntry::Directory(a), FileTreeEntry::Directory(b)) = (a, b) {
                            a.file_name().cmp(&b.file_name())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    files.sort_by(|a, b| {
                        if let (FileTreeEntry::File(a), FileTreeEntry::File(b)) = (a, b) {
                            a.file_name().cmp(&b.file_name())
                        } else {
                            std::cmp::Ordering::Equal
                        }
                    });

                    if dir.parent().is_some() && self.settings.show_subfolders {
                        self.file_tree_entries.push(FileTreeEntry::Directory(
                            dir.parent().unwrap().to_path_buf(),
                        ));
                    }

                    self.file_tree_entries.extend(folders);
                    self.file_tree_entries.extend(files);

                    let folder_count = self
                        .file_tree_entries
                        .iter()
                        .filter(|e| matches!(e, FileTreeEntry::Directory(_)))
                        .count();
                    let file_count = self
                        .file_tree_entries
                        .iter()
                        .filter(|e| matches!(e, FileTreeEntry::File(_)))
                        .count();

                    self.log_info(format!(
                        "Found {} folders and {} .sen files",
                        folder_count, file_count
                    ));
                }
                Err(e) => {
                    self.log_error(format!("Failed to read directory: {}", e));
                }
            }
        } else {
            self.log_warning("No directory selected for file tree");
        }
    }

    /// Update window title based on current file and modified state
    pub(crate) fn update_window_title(&self, ctx: &egui::Context) {
        let title = if let Some(path) = &self.current_file_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            let modified = if self.is_modified { "*" } else { "" };
            format!("{} {} - SEN", filename, modified)
        } else {
            let modified = if self.is_modified { "*" } else { "" };
            format!("Untitled {} - SEN", modified)
        };

        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
    }
}
