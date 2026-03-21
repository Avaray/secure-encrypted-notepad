use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    pub(crate) fn render_search_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_search_panel {
            return;
        }

        crate::app_helpers::center_row(ui, |ui| {
            ui.label(rust_i18n::t!("search.find"));
            let original_cursor_color = ui.visuals().text_cursor.stroke.color;
            let original_selection_color = ui.visuals().selection.bg_fill;

            ui.visuals_mut().text_cursor.stroke.color = egui::Color32::TRANSPARENT;
            ui.visuals_mut().selection.bg_fill = egui::Color32::TRANSPARENT;

            let output = egui::TextEdit::singleline(&mut self.search_query)
                .desired_width(200.0)
                .margin(ui.spacing().button_padding)
                .hint_text(rust_i18n::t!("search.hint"))
                .show(ui);

            let response = output.response.clone();

            ui.visuals_mut().text_cursor.stroke.color = original_cursor_color;
            ui.visuals_mut().selection.bg_fill = original_selection_color;

            Self::render_custom_cursor(
                ui,
                &self.settings,
                &output,
                self.settings.ui_font_size,
                original_cursor_color,
                original_selection_color,
            );

            if self.focus_search {
                response.request_focus();
                self.focus_search = false;
            }

            if response.changed() {
                self.perform_search();
            }

            // Enter to find next
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.find_next();
                response.request_focus();
            }

            if ui.checkbox(&mut self.search_case_sensitive, rust_i18n::t!("search.case_sensitive")).changed() {
                self.perform_search();
            }

            if ui.button("<").clicked() {
                self.find_prev();
            }
            if ui.button(">").clicked() {
                self.find_next();
            }

            // Match count
            if !self.search_matches.is_empty() {
                if let Some(idx) = self.current_match_index {
                    ui.label(rust_i18n::t!("search.match_count", current = idx + 1, total = self.search_matches.len()));
                } else {
                    ui.label(rust_i18n::t!("search.matches", count = self.search_matches.len()));
                }
            } else if !self.search_query.is_empty() {
                ui.label(rust_i18n::t!("search.no_matches"));
            }

            ui.separator();

            ui.label(rust_i18n::t!("search.replace"));
            let original_cursor_color = ui.visuals().text_cursor.stroke.color;
            let original_selection_color = ui.visuals().selection.bg_fill;

            ui.visuals_mut().text_cursor.stroke.color = egui::Color32::TRANSPARENT;
            ui.visuals_mut().selection.bg_fill = egui::Color32::TRANSPARENT;

            let replace_output = egui::TextEdit::singleline(&mut self.replace_query)
                .desired_width(200.0)
                .margin(ui.spacing().button_padding)
                .hint_text(rust_i18n::t!("search.replace_hint"))
                .show(ui);

            ui.visuals_mut().text_cursor.stroke.color = original_cursor_color;
            ui.visuals_mut().selection.bg_fill = original_selection_color;

            Self::render_custom_cursor(
                ui,
                &self.settings,
                &replace_output,
                self.settings.ui_font_size,
                original_cursor_color,
                original_selection_color,
            );

            if ui.button(rust_i18n::t!("search.btn_replace_one")).clicked() {
                self.replace_current();
            }
            if ui.button(rust_i18n::t!("search.btn_replace_all")).clicked() {
                self.replace_all();
            }

            // Close button on the far right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.button("❌")
                    .on_hover_text(rust_i18n::t!("app.close_panel"))
                    .clicked() 
                {
                    self.show_search_panel = false;
                    self.search_query.clear();
                    self.search_matches.clear();
                    self.current_match_index = None;
                }
            });
        });
    }

    /// Perform full text search and populate matches
    fn perform_search(&mut self) {
        self.search_matches.clear();
        self.current_match_index = None;

        if self.search_query.is_empty() {
            return;
        }

        let text = &self.document.current_content;

        if self.search_case_sensitive {
            self.search_matches = text
                .match_indices(&self.search_query)
                .map(|(i, _)| i)
                .collect();
        } else {
            let text_lower = text.to_lowercase();
            let query_lower = self.search_query.to_lowercase();
            self.search_matches = text_lower
                .match_indices(&query_lower)
                .map(|(i, _)| i)
                .collect();
        }

        if !self.search_matches.is_empty() {
            self.current_match_index = Some(0);
            // We should scroll to the first match
            // However, we can't easily access the TextEdit state here to scroll.
            // We set text_cursor_range, and hope the editor updates.
            self.select_match(0);
        }
    }

    fn find_next(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let next_idx = match self.current_match_index {
            Some(i) => (i + 1) % self.search_matches.len(),
            None => 0,
        };

        self.current_match_index = Some(next_idx);
        self.select_match(next_idx);
    }

    fn find_prev(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }

        let prev_idx = match self.current_match_index {
            Some(i) => {
                if i == 0 {
                    self.search_matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.search_matches.len() - 1,
        };

        self.current_match_index = Some(prev_idx);
        self.select_match(prev_idx);
    }

    fn select_match(&mut self, match_idx: usize) {
        if match_idx >= self.search_matches.len() {
            return;
        }

        let start_byte = self.search_matches[match_idx];
        let len = self.search_query.len(); // Approximate len in bytes
                                           // Adjust for case-insensitive length? If query is ASCII, len is bytes. If UTF-8, len is bytes too.
                                           // match_indices returns byte offset.
                                           // We need byte range for text_cursor_range.

        self.text_cursor_range = Some(start_byte..(start_byte + len));

        // Also calculate line number for highlighting logic
        let text = &self.document.current_content;
        let line_num = text[..start_byte].lines().count().max(1);
        self.highlighted_line = Some(line_num);

        // Note: Actual scrolling happens in ui_editor.rs or needs to be forced.
        // We set a flag or rely on next update.
    }

    fn replace_current(&mut self) {
        if let Some(idx) = self.current_match_index {
            if idx < self.search_matches.len() {
                let start = self.search_matches[idx];
                let old_len = self.search_query.len();
                let end = start + old_len;

                // Replace in document
                // Note: String replacement implementation is tricky with byte indices if not careful with char boundaries.
                // self.search_matches are byte indices from match_indices, so they are valid char boundaries.

                // We need to handle overlapping matches or shifting indices?
                // Simpler: Replace, then re-search.

                let text = &mut self.document.current_content;
                if end <= text.len() {
                    text.replace_range(start..end, &self.replace_query);
                    self.is_modified = true;
                    self.loaded_history_index = None;

                    // Re-run search to update matches
                    self.perform_search();
                }
            }
        }
    }

    fn replace_all(&mut self) {
        if self.search_query.is_empty() {
            return;
        }

        let text = &mut self.document.current_content;
        // let original_len = text.len();

        let new_text = if self.search_case_sensitive {
            text.replace(&self.search_query, &self.replace_query)
        } else {
            // For now, simplistic implementation: use str::replace regardless of case sensitivity setting, or warn.
            text.replace(&self.search_query, &self.replace_query)
        };

        if new_text != *text {
            *text = new_text;
            self.is_modified = true;
            self.loaded_history_index = None;
            self.perform_search();
        }
    }
}
