use crate::app_state::{FileTreeEntry, KeyStatus, LogEntry, LogLevel};
use crate::EditorApp;
use rust_i18n::t;
use std::path::PathBuf;

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
    pub(crate) fn toggle_comment_lines(&mut self, ctx: &egui::Context) {
        let lines_to_toggle = self.get_lines_in_selection();
        if lines_to_toggle.is_empty() {
            return;
        }

        let text = &mut self.document.current_content;

        // POPRAWKA: Użyj split('\n') zamiast lines() aby zachować końcowe puste linie
        let lines: Vec<&str> = text.split('\n').collect();

        let comment_prefix = self.settings.comment_prefix.clone();
        let comment_prefix_space = format!("{} ", comment_prefix);

        let all_commented = lines_to_toggle.iter().all(|&line_idx| {
            if line_idx < lines.len() {
                let trimmed = lines[line_idx].trim_start();
                trimmed.starts_with(&comment_prefix)
            } else {
                false
            }
        });

        let was_empty = if lines_to_toggle.len() == 1 {
            let line_idx = lines_to_toggle[0];
            line_idx < lines.len() && lines[line_idx].trim_start().is_empty()
        } else {
            false
        };

        let added_len = comment_prefix_space.chars().count();

        let mut new_lines: Vec<String> = Vec::new();

        for (idx, line) in lines.iter().enumerate() {
            if lines_to_toggle.contains(&idx) {
                let trimmed = line.trim_start();

                if all_commented {
                    // Uncomment
                    if trimmed.starts_with(&comment_prefix_space) {
                        let uncommented = trimmed.strip_prefix(&comment_prefix_space).unwrap();
                        let indent = line.len() - trimmed.len();
                        new_lines.push(format!("{}{}", " ".repeat(indent), uncommented));
                    } else if trimmed.starts_with(&comment_prefix) {
                        let uncommented = trimmed.strip_prefix(&comment_prefix).unwrap();
                        let indent = line.len() - trimmed.len();
                        new_lines.push(format!("{}{}", " ".repeat(indent), uncommented));
                    } else {
                        new_lines.push(line.to_string());
                    }
                } else {
                    // Comment
                    if !trimmed.starts_with(&comment_prefix) {
                        let indent_count = line.len() - trimmed.len();
                        new_lines.push(format!("{}{}{}", " ".repeat(indent_count), comment_prefix_space, trimmed));
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

        if lines_to_toggle.len() == 1 && was_empty && !all_commented {
            if let Some(id) = self.text_edit_id {
                if let Some(mut state) = egui::TextEdit::load_state(ctx, id) {
                    if let Some(mut range) = state.cursor.char_range() {
                        range.primary.index += added_len;
                        range.secondary.index += added_len;
                        state.cursor.set_char_range(Some(range));
                        state.store(ctx, id);
                    }
                }
            }
        }

        if lines_to_toggle.len() == 1 {
            self.log_info(t!("helpers.log_comment_line", line = lines_to_toggle[0] + 1));
        } else {
            self.log_info(t!("helpers.log_comment_lines", count = lines_to_toggle.len()));
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
                self.log_error(t!("helpers.log_watcher_failed", e = e));
                return;
            }
        };
 
        let mode = if self.settings.tree_style_file_tree {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
 
        if let Err(e) = watcher.watch(dir, mode) {
            self.log_error(t!("helpers.log_watch_dir_failed", e = e));
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

            for (line_idx, line) in text.split('\n').enumerate() {
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
            self.log_info(t!(
                "helpers.log_refresh_tree",
                file = self.mask_directory_path(&dir)
            ));
            
            if self.settings.tree_style_file_tree {
                // Tree View (recursive lazy load)
                let mut entries = Vec::new();
                
                let has_parent = dir.parent().is_some();
                // Add parent directory ".." button at the top
                if has_parent {
                    entries.push(FileTreeEntry {
                        path: dir.parent().unwrap().to_path_buf(),
                        is_dir: true,
                        is_expanded: false,
                        depth: 0,
                    });
                }
                
                let start_depth = if has_parent { 1 } else { 0 };
                
                self.build_tree_recursive(&dir, start_depth, &mut entries);
                self.file_tree_entries = entries;
                
            } else {
                // Simple View (flat list)
                match std::fs::read_dir(&dir) {
                    Ok(read_dir) => {
                        let mut folders = Vec::new();
                        let mut files = Vec::new();

                        for entry_res in read_dir {
                            if let Ok(entry) = entry_res {
                                let path = entry.path();
                                if path.is_dir() && self.settings.show_subfolders {
                                    folders.push(FileTreeEntry {
                                        path: path.clone(),
                                        is_dir: true,
                                        is_expanded: false,
                                        depth: 0,
                                    });
                                } else if path.is_file() {
                                    if let Some(ext) = path.extension() {
                                        if ext == "sen" {
                                            files.push(FileTreeEntry {
                                                path: path.clone(),
                                                is_dir: false,
                                                is_expanded: false,
                                                depth: 0,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        folders.sort_by(|a, b| a.path.file_name().unwrap_or_default().cmp(b.path.file_name().unwrap_or_default()));
                        files.sort_by(|a, b| a.path.file_name().unwrap_or_default().cmp(b.path.file_name().unwrap_or_default()));

                        if dir.parent().is_some() && self.settings.show_subfolders {
                            self.file_tree_entries.push(FileTreeEntry {
                                path: dir.parent().unwrap().to_path_buf(),
                                is_dir: true,
                                is_expanded: false,
                                depth: 0,
                            });
                        }

                        self.file_tree_entries.extend(folders);
                        self.file_tree_entries.extend(files);
                    }
                    Err(_) => {
                        self.log_error(t!("helpers.log_read_dir_failed"));
                    }
                }
            }
            
            let folder_count = self.file_tree_entries.iter().filter(|e| e.is_dir).count();
            let file_count = self.file_tree_entries.iter().filter(|e| !e.is_dir).count();

            self.log_info(t!("helpers.log_tree_stats", folders = folder_count, files = file_count));

            // Refresh access status for the newly loaded tree
            self.refresh_file_access_status();
            
        } else {
            self.log_info(t!("helpers.log_tree_no_dir"));
        }
    }

    /// Recursively build the file tree entry list for Tree View
    fn build_tree_recursive(
        &self,
        dir: &PathBuf,
        depth: usize,
        out_entries: &mut Vec<FileTreeEntry>,
    ) {
        let Ok(read_dir) = std::fs::read_dir(dir) else { return };
        
        let mut child_folders = Vec::new();
        let mut child_files = Vec::new();

        for entry_res in read_dir {
            if let Ok(entry) = entry_res {
                let path = entry.path();
                if path.is_dir() {
                    child_folders.push(path);
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "sen" {
                            child_files.push(path);
                        }
                    }
                }
            }
        }

        child_folders.sort_by(|a, b| a.file_name().unwrap_or_default().cmp(b.file_name().unwrap_or_default()));
        child_files.sort_by(|a, b| a.file_name().unwrap_or_default().cmp(b.file_name().unwrap_or_default()));


        for path in child_folders {
            let is_expanded = self.expanded_directories.contains(&path);

            out_entries.push(FileTreeEntry {
                path: path.clone(),
                is_dir: true,
                is_expanded,
                depth,
            });
            
            if is_expanded {
                self.build_tree_recursive(&path, depth + 1, out_entries);
            }
        }

        for path in child_files {
            out_entries.push(FileTreeEntry {
                path,
                is_dir: false,
                is_expanded: false,
                depth,
            });
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
            self.pending_access_checks.clear();
            self.log_info(t!("helpers.log_key_changed"));
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
            if !entry.is_dir {
                if entry.path.extension().and_then(|s| s.to_str()) == Some("sen") {
                    if !self.file_access_cache.contains_key(&entry.path) && !self.pending_access_checks.contains(&entry.path) {
                        paths_to_check.push(entry.path.clone());
                        self.pending_access_checks.insert(entry.path.clone());
                    }
                }
            }
        }

        if paths_to_check.is_empty() {
            return;
        }

        self.log_info(t!("helpers.log_check_bg", count = paths_to_check.len()));

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
                self.file_access_cache.insert(path.clone(), status);
                self.pending_access_checks.remove(&path);
                got_any = true;
            }

            // If we got results, we need a repaint to show them
            if got_any {
                ctx.request_repaint();
            }
        }
    }

    /// Refresh access status for all files in the batch list (ASYNCHRONOUS)
    pub(crate) fn refresh_batch_file_access_status(&mut self) {
        // 1. Update/validate current batch key hash
        let mut new_hash = None;
        if let Some(kp) = &self.batch_keyfile {
            if let Ok(hash) = crate::crypto::get_keyfile_hash(kp) {
                new_hash = Some(hash);
            }
        }

        let key_changed = new_hash != self.batch_current_key_hash;
        if key_changed {
            self.batch_current_key_hash = new_hash;
            self.batch_file_access_cache.clear();
        }

        let key_hash = match self.batch_current_key_hash {
            Some(h) => h,
            None => {
                self.batch_file_access_cache.clear();
                return;
            }
        };

        // 2. Prevent spawning multiple threads if one is already active
        if self.batch_access_check_receiver.is_some() {
            return;
        }

        // 3. Identify files that need checking
        let mut paths_to_check = Vec::new();
        for path in &self.batch_files {
            if !self.batch_file_access_cache.contains_key(path) {
                // Efficiency: Only background check .sen files
                if path.extension().and_then(|s| s.to_str()) == Some("sen") {
                    paths_to_check.push(path.clone());
                } else {
                    // Mark others as NotSen immediately (synchronously)
                    self.batch_file_access_cache.insert(path.clone(), KeyStatus::NotSen);
                }
            }
        }

        if paths_to_check.is_empty() {
            return;
        }

        // 4. Spawn background checking thread
        let (tx, rx) = std::sync::mpsc::channel();
        self.batch_access_check_receiver = Some(rx);

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

    /// Process results from background batch access checks (call every frame)
    pub(crate) fn process_batch_access_check_results(&mut self, ctx: &egui::Context) {
        if let Some(rx) = &self.batch_access_check_receiver {
            let mut got_any = false;
            let mut finished = false;
            
            // Try to receive results
            while let Ok((path, status)) = rx.try_recv() {
                self.batch_file_access_cache.insert(path, status);
                got_any = true;
            }

            // Check if the sender has been dropped (thread finished)
            if let Err(std::sync::mpsc::TryRecvError::Disconnected) = rx.try_recv() {
                finished = true;
            }

            if finished {
                self.batch_access_check_receiver = None;
            }

            if got_any {
                ctx.request_repaint();
            }
        }
    }

    /// Process results from background batch operation progress (call every frame)
    pub(crate) fn process_batch_progress_results(&mut self, ctx: &egui::Context) {
        if let Some(rx) = self.batch_progress_receiver.take() {
            let mut got_any = false;
            let mut finished = false;

            while let Ok(update) = rx.try_recv() {
                got_any = true;
                match update {
                    crate::app_state::BatchProgressUpdate::Log(level, msg) => {
                        match level {
                            crate::app_state::LogLevel::Info => self.log_info(msg),
                            crate::app_state::LogLevel::Success => self.log_success(msg),
                            crate::app_state::LogLevel::Warning => self.log_warning(msg),
                            crate::app_state::LogLevel::Error => self.log_error(msg),
                        }
                    }
                    crate::app_state::BatchProgressUpdate::Progress(count, success, failed) => {
                        self.batch_progress_count = count;
                        self.batch_success_count = success;
                        self.batch_failed_count = failed;
                    }
                    crate::app_state::BatchProgressUpdate::Finished(success, failed) => {
                        self.batch_success_count = success;
                        self.batch_failed_count = failed;
                        self.batch_is_running = false;
                        finished = true;
                        

                        let mode_key = match self.batch_mode {
                            crate::app_state::BatchMode::Encrypt => t!("batch.mode_encrypt"),
                            crate::app_state::BatchMode::Decrypt => t!("batch.mode_decrypt"),
                            crate::app_state::BatchMode::Rotate => t!("batch.mode_rotate"),
                        };
                        self.status_message = t!("helpers.status_batch_finished", mode = mode_key, success = success, failed = failed).to_string();
                    }
                }
            }

            if !finished {
                self.batch_progress_receiver = Some(rx);
            }

            if got_any {
                ctx.request_repaint();
            }
        }
    }

    /// Update window title based on current file and modified state
    pub(crate) fn update_window_title(&self, ctx: &egui::Context) {
        let title = if self.settings.hide_filename_in_title {
            if self.is_modified {
                t!("helpers.title_sen_mod").to_string()
            } else {
                t!("helpers.title_sen").to_string()
            }
        } else if let Some(path) = &self.current_file_path {
            let filename = path.file_name().unwrap_or_default().to_string_lossy();
            let display_name = filename.strip_suffix(".sen").unwrap_or(&filename);
            
            if self.is_modified {
                t!("helpers.title_file_mod", file = display_name).to_string()
            } else {
                t!("helpers.title_file", file = display_name).to_string()
            }
        } else {
            if self.is_modified {
                t!("helpers.title_untitled_mod", file = t!("helpers.title_untitled")).to_string()
            } else {
                t!("helpers.title_untitled").to_string()
            }
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

    /// Toggle Zen Mode (minimalist UI + Fullscreen)
    pub(crate) fn toggle_zen_mode(&mut self, ctx: &egui::Context) {
        self.zen_mode = !self.zen_mode;
        self.settings.zen_mode = self.zen_mode;
        
        if self.zen_mode {
            // Enter Fullscreen
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
            self.log_info(t!("helpers.log_zen_on"));
        } else {
            // Exit Fullscreen
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            // Force restore maximized state if it was set
            if self.settings.start_maximized {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
            self.log_info(t!("helpers.log_zen_off"));
        }
    }
    /// Truncate text to fit width, eating any trailing dots before the ellipsis
    pub(crate) fn smart_truncate_text(&self, ui: &egui::Ui, text: &str, font_id: egui::FontId, max_width: f32) -> String {
        // Measure the full text to see if we need truncation
        let galley = ui.painter().layout_no_wrap(text.to_string(), font_id.clone(), egui::Color32::BLACK);
        if galley.rect.width() <= max_width {
            return text.to_string();
        }

        let ellipsis = "...";
        let ellipsis_galley = ui.painter().layout_no_wrap(ellipsis.to_string(), font_id.clone(), egui::Color32::BLACK);
        let ellipsis_width = ellipsis_galley.rect.width();

        let target_width = max_width - ellipsis_width;
        if target_width <= 0.0 {
            return ellipsis.to_string();
        }

        // Binary search for the best truncation point
        let chars: Vec<char> = text.chars().collect();
        let mut lo = 0;
        let mut hi = chars.len();
        let mut best_count = 0;

        while lo <= hi {
            let mid = (lo + hi) / 2;
            let sub: String = chars[..mid].iter().collect();
            let w = ui.painter().layout_no_wrap(sub, font_id.clone(), egui::Color32::BLACK).rect.width();
            if w <= target_width {
                best_count = mid;
                lo = mid + 1;
            } else {
                hi = mid - 1;
            }
        }

        let mut result: String = chars[..best_count].iter().collect();
        // Eat the trailing dot(s) and spaces to avoid "...." or ". ..." look
        while result.ends_with('.') || result.ends_with(' ') {
            result.pop();
        }

        format!("{}{}", result, ellipsis)
    }

    /// Reusable panel header with title, optional subtitle, close button and vertical centering
    /// Returns true if the close button was clicked.
    pub(crate) fn render_panel_header(
        &self,
        ui: &mut egui::Ui,
        title: &str,
        subtitle: Option<&str>,
        add_separator: bool,
    ) -> bool {
        if self.settings.hide_panel_headers {
            return false;
        }

        let mut close_clicked = false;

        crate::app_helpers::center_row(ui, |ui| {
            // Focus on vertical centering by ensuring enough row height based on current font
            let heading_height = ui.text_style_height(&egui::TextStyle::Heading);
            ui.set_min_height(heading_height + 4.0); 
            
            ui.heading(title);
            
            if let Some(sub) = subtitle {
                ui.add_space(8.0);
                ui.label(egui::RichText::new(sub).weak());
            }
 
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0); // Consistent padding from right edge
                if ui.button("❌")
                    .on_hover_text(rust_i18n::t!("app.close_panel"))
                    .clicked() 
                {
                    close_clicked = true;
                }
            });
        });
        
        if add_separator {
            let space = (self.settings.ui_font_size * 0.25).max(2.0).min(6.0);
            ui.add_space(space);
            ui.separator();
            ui.add_space(space);
        }

        close_clicked
    }
}

pub fn center_row<R>(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    // We use the base interaction height to provide a stable vertical axis
    // for all elements in the row, preventing inconsistencies between 
    // plain labels and interactive widgets.
    let h = ui.spacing().interact_size.y;
    
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), h),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            add_contents(ui)
        }
    )
}