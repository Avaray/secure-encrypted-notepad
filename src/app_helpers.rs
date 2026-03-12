use crate::app_state::{FileTreeEntry, KeyStatus, LogEntry, LogLevel};
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

    pub(crate) fn log_success(&mut self, message: impl Into<String>) {
        self.debug_log
            .push(LogEntry::new(LogLevel::Success, message.into()));
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

            // Text cursor settings
            style.visuals.text_cursor.blink = self.settings.cursor_blink;
            style.visuals.text_cursor.stroke.width = match self.settings.cursor_shape {
                crate::settings::CursorShape::Bar => 2.0,
                crate::settings::CursorShape::Block => self.settings.editor_font_size * 0.6, // Roughly character width
                crate::settings::CursorShape::Underscore => 2.0, // Fallback for now, maybe custom draw later
            };

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

    /// Setup file system watcher for the current directory
    pub(crate) fn setup_watcher(&mut self) {
        use notify::{RecursiveMode, Watcher};

        // Stop previous watcher if any
        self.watcher = None;

        let Some(dir) = &self.file_tree_dir else {
            return;
        };

        let (tx, rx) = std::sync::mpsc::channel();
        self.watcher_receiver = Some(rx);

        let mut watcher = match notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        }) {
            Ok(w) => w,
            Err(e) => {
                self.log_error(format!("Failed to create watcher: {}", e));
                return;
            }
        };

        if let Err(e) = watcher.watch(dir, RecursiveMode::NonRecursive) {
            self.log_error(format!("Failed to watch directory: {}", e));
            return;
        }

        self.watcher = Some(watcher);
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
            self.log_info(format!(
                "Refreshing file tree for: {}",
                self.mask_directory_path(&dir)
            ));
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

                    // Refresh access status for the newly loaded tree
                    self.refresh_file_access_status();
                }
                Err(e) => {
                    self.log_error(format!("Failed to read directory: {}", e));
                }
            }
        } else {
            self.log_warning("No directory selected for file tree");
        }
    }

    /// Refresh access status for all SEN files in the tree (ASYNCHRONOUS)
    pub(crate) fn refresh_file_access_status(&mut self) {
        // 1. Update/validate current key hash
        let mut new_hash = None;
        if let Some(kp) = &self.keyfile_path {
            if let Ok(hash) = crate::crypto::get_keyfile_hash(kp) {
                new_hash = Some(hash);
            }
        }

        let key_changed = new_hash != self.current_key_hash;
        if key_changed {
            self.current_key_hash = new_hash;
            self.file_access_cache.clear();
            self.log_info("Keyfile changed, clearing access cache");
        }

        let key_hash = match self.current_key_hash {
            Some(h) => h,
            None => {
                self.file_access_cache.clear();
                return;
            }
        };

        // 2. Identify files that need checking
        let mut paths_to_check = Vec::new();
        for entry in &self.file_tree_entries {
            if let FileTreeEntry::File(path) = entry {
                if path.extension().and_then(|s| s.to_str()) == Some("sen") {
                    if !self.file_access_cache.contains_key(path) {
                        paths_to_check.push(path.clone());
                    }
                }
            }
        }

        if paths_to_check.is_empty() {
            return;
        }

        self.log_info(format!(
            "Checking access for {} files in background...",
            paths_to_check.len()
        ));

        // 3. Spawn background checking thread
        let (tx, rx) = std::sync::mpsc::channel();
        self.access_check_receiver = Some(rx);

        // We'll use a thread to check these files one by one.
        // For very large trees, we might want a pool, but this is simple and doesn't block UI.
        std::thread::spawn(move || {
            for path in paths_to_check {
                let status = match crate::crypto::check_key_compatibility(&key_hash, &path) {
                    Ok(true) => KeyStatus::Decryptable,
                    Ok(false) => KeyStatus::WrongKey,
                    Err(e) => match e {
                        crate::crypto::CryptoError::InvalidMagicNumber => KeyStatus::NotSen,
                        _ => KeyStatus::WrongKey,
                    },
                };
                let _ = tx.send((path, status));
            }
        });
    }

    /// Process results from background file access checks (call every frame)
    pub(crate) fn process_access_check_results(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.access_check_receiver {
            let mut got_any = false;
            // Drain as many results as possible in one frame (non-blocking)
            while let Ok((path, status)) = rx.try_recv() {
                self.file_access_cache.insert(path, status);
                got_any = true;
            }

            // If we got results, we need a repaint to show them
            if got_any {
                ctx.request_repaint();
                // Optionally log some progress or remove receiver if empty and thread finished
                // Wait, mpsc channel doesn't tell us if it's "finished" easily unless we check for hung tx.
                // But try_recv will error on empty.
            }
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

    /// Standard cursor rendering logic for TextEdit widgets to follow app settings
    pub(crate) fn render_custom_cursor(
        ui: &mut egui::Ui,
        settings: &crate::settings::Settings,
        output: &egui::text_edit::TextEditOutput,
        font_size: f32,
        cursor_color: egui::Color32,
        selection_color: egui::Color32,
    ) {
        if !ui.memory(|mem| mem.has_focus(output.response.id)) {
            return;
        }

        if let Some(cursor_range) = output.cursor_range {
            let cursor_pos = cursor_range.primary;
            let galley = &output.galley;
            let cursor_rect = galley.pos_from_cursor(cursor_pos);

            let row_top_screen = cursor_rect.min.y + output.galley_pos.y;
            let painter_rect = egui::Rect::from_min_max(
                egui::pos2(cursor_rect.min.x + output.galley_pos.x, row_top_screen),
                egui::pos2(
                    cursor_rect.max.x + output.galley_pos.x,
                    row_top_screen + font_size,
                ),
            );

            let is_visible = if settings.cursor_blink {
                (ui.input(|i| i.time) * 2.0) as i32 % 2 == 0
            } else {
                true
            };

            if is_visible {
                match settings.cursor_shape {
                    crate::settings::CursorShape::Bar => {
                        ui.painter().line_segment(
                            [painter_rect.left_top(), painter_rect.left_bottom()],
                            egui::Stroke::new(2.0, cursor_color),
                        );
                    }
                    crate::settings::CursorShape::Block => {
                        let char_width = font_size * 0.6;
                        let block_rect = egui::Rect::from_min_size(
                            painter_rect.min,
                            egui::vec2(char_width, painter_rect.height()),
                        );
                        ui.painter().rect_filled(
                            block_rect,
                            0.0,
                            cursor_color.linear_multiply(0.7),
                        );
                    }
                    crate::settings::CursorShape::Underscore => {
                        let char_width = font_size * 0.6;
                        let line_y = painter_rect.bottom() - 1.0;
                        ui.painter().line_segment(
                            [
                                egui::pos2(painter_rect.left(), line_y),
                                egui::pos2(painter_rect.left() + char_width, line_y),
                            ],
                            egui::Stroke::new(2.0, cursor_color),
                        );
                    }
                }
            }

            // Draw manual selection highlights
            if cursor_range.primary != cursor_range.secondary {
                let (min_idx, max_idx) =
                    if cursor_range.primary.index < cursor_range.secondary.index {
                        (cursor_range.primary.index, cursor_range.secondary.index)
                    } else {
                        (cursor_range.secondary.index, cursor_range.primary.index)
                    };

                let mut current_char_idx = 0;
                for row in &galley.rows {
                    let row_char_count = row.char_count_including_newline();
                    let row_end_idx = current_char_idx + row_char_count;

                    if row_end_idx > min_idx && current_char_idx < max_idx {
                        let local_min = if min_idx > current_char_idx {
                            min_idx - current_char_idx
                        } else {
                            0
                        };
                        let local_max = if max_idx < row_end_idx {
                            max_idx - current_char_idx
                        } else {
                            row_char_count
                        };

                        let x_min = row.x_offset(local_min);
                        let x_max = row.x_offset(local_max);

                        let rect = egui::Rect::from_min_max(
                            egui::pos2(
                                x_min + output.galley_pos.x,
                                row.min_y() + output.galley_pos.y,
                            ),
                            egui::pos2(
                                x_max + output.galley_pos.x,
                                row.min_y() + output.galley_pos.y + font_size,
                            ),
                        );

                        ui.painter()
                            .rect_filled(rect, 0.0, selection_color.linear_multiply(0.7));
                    }
                    current_char_idx = row_end_idx;
                }
            }
        }
    }
}
