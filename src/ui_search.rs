use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    pub(crate) fn render_search_panel(&mut self, ui: &mut egui::Ui) {
        if !self.show_search_panel {
            return;
        }

        ui.horizontal_centered(|ui| {
            if ui.button("X").clicked() {
                self.show_search_panel = false;
                self.search_query.clear();
                self.search_matches.clear();
                self.current_match_index = None;
            }

            ui.label("Find:");
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(180.0)
                    .hint_text("Search..."),
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
            
            if ui.checkbox(&mut self.search_case_sensitive, "Aa").changed() {
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
                    ui.label(format!("{} of {}", idx + 1, self.search_matches.len()));
                } else {
                    ui.label(format!("{} matches", self.search_matches.len()));
                }
            } else if !self.search_query.is_empty() {
                ui.label("No matches");
            }
            
            ui.separator();
            
            ui.label("Replace:");
            ui.add(egui::TextEdit::singleline(&mut self.replace_query).desired_width(120.0).hint_text("Text"));
            
            if ui.button("Replace").clicked() {
                 self.replace_current();
            }
            if ui.button("All").clicked() {
                 self.replace_all();
            }
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
            self.search_matches = text.match_indices(&self.search_query).map(|(i, _)| i).collect();
        } else {
            let text_lower = text.to_lowercase();
            let query_lower = self.search_query.to_lowercase();
            self.search_matches = text_lower.match_indices(&query_lower).map(|(i, _)| i).collect();
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
        if match_idx >= self.search_matches.len() { return; }
        
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
        if self.search_query.is_empty() { return; }
        
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
