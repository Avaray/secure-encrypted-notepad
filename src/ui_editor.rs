use crate::EditorApp;
use eframe::egui;

fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(s.len())
}

fn byte_to_char_idx(s: &str, byte_idx: usize) -> usize {
    s.get(..byte_idx).map(|sub| sub.chars().count()).unwrap_or_else(|| s.chars().count())
}

impl EditorApp {
    pub(crate) fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = if text.is_empty() {
            1
        } else {
            text.lines().count() + if text.ends_with('\n') { 1 } else { 0 }
        };
        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;
        let highlight_line = self.highlighted_line;
        let word_wrap = self.settings.word_wrap;

        // Capture colors
        let foreground_color = self.current_theme.colors.editor_foreground_color();
        let selection_bg = self.current_theme.colors.selection_color();
        let line_number_color = self.current_theme.colors.line_number_color();
        let comment_color = self.current_theme.colors.comment_color();
        let _highlight_bg = selection_bg.linear_multiply(0.2);

        // Capture search state for the layouter
        let search_matches: Vec<usize> = self.search_matches.clone();
        let search_query_len = self.search_query.len();
        let search_active = !self.search_query.is_empty() && !search_matches.is_empty();
        let highlight_color = self.current_theme.colors.highlight_color();

        // Symmetric padding around line numbers
        let line_number_side_padding = 10.0;
        let text_left_padding = 10.0;

        // Calculate precise width for line numbers
        let line_number_width = if show_line_numbers {
            let font_id = egui::FontId::monospace(editor_font_size);
            let max_line_text = format!("{}", line_count);
            let text_width =
                ui.fonts(|f| f.glyph_width(&font_id, ' ') * max_line_text.len() as f32);
            line_number_side_padding + text_width + line_number_side_padding
        } else {
            0.0
        };

        // Fixed ID for TextEdit
        let text_edit_id = ui.id().with("main_text_editor");

        // Save fixed position for line numbers (left edge of view)
        let editor_left_edge = ui.cursor().left();
        let line_numbers_x = if show_line_numbers {
            editor_left_edge + line_number_width - line_number_side_padding
        } else {
            0.0
        };
        let separator_x = if show_line_numbers {
            editor_left_edge + line_number_width
        } else {
            0.0
        };

        // Save original clip rect of the whole editor
        let full_clip_rect = ui.clip_rect();

        // Load current scroll state
        let scroll_id = ui.id().with("main_editor");
        let mut scroll_state =
            egui::scroll_area::State::load(ui.ctx(), scroll_id).unwrap_or_default();

        // MIDDLE MOUSE BUTTON PANNING
        let pointer = ui.input(|i| i.pointer.clone());
        let middle_button_active = pointer.middle_down();

        if middle_button_active {
            if let Some(pos) = pointer.latest_pos() {
                if full_clip_rect.contains(pos) {
                    let delta = pointer.delta();
                    scroll_state.offset.x -= delta.x;
                    scroll_state.offset.y -= delta.y;
                    scroll_state.offset.x = scroll_state.offset.x.max(0.0);
                    scroll_state.offset.y = scroll_state.offset.y.max(0.0);
                    scroll_state.store(ui.ctx(), scroll_id);
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                }
            }
        }

        let scroll_output = egui::ScrollArea::both()
            .id_salt("main_editor")
            .auto_shrink(false)
            .scroll_offset(scroll_state.offset.into())
            .show(ui, |ui| {
                // Set clip rect for text - starts AFTER line numbers
                let text_area_clip = if show_line_numbers {
                    egui::Rect::from_min_max(
                        egui::pos2(separator_x + text_left_padding, full_clip_rect.min.y),
                        full_clip_rect.max,
                    )
                } else {
                    full_clip_rect
                };
                ui.set_clip_rect(text_area_clip);

                // Save scroll area rect for auto-scroll during selection
                let scroll_area_rect = ui.clip_rect();

                // Create clickable area over the FULL height
                let full_area = ui.available_rect_before_wrap();
                let sense_rect = ui.interact(
                    full_area,
                    ui.id().with("editor_click_area"),
                    egui::Sense::click(),
                );

                ui.horizontal_top(|ui| {
                    if show_line_numbers {
                        ui.add_space(line_number_width + text_left_padding);
                    }

                    let layouter = |ui: &egui::Ui, text_str: &str, wrap_width: f32| {
                        let mut layout_job = egui::text::LayoutJob::default();
                        let font_id = egui::FontId::monospace(editor_font_size);

                        // Set wrapping
                        if word_wrap {
                            layout_job.wrap.max_width = wrap_width;
                        } else {
                            layout_job.wrap.max_width = f32::INFINITY;
                        }

                        // Split text into lines & process each
                        let mut byte_offset: usize = 0;

                        // Handle empty text
                        if text_str.is_empty() {
                            layout_job.append(
                                "",
                                0.0,
                                egui::TextFormat {
                                    font_id: font_id.clone(),
                                    color: foreground_color,
                                    ..Default::default()
                                },
                            );
                            return ui.fonts(|f| f.layout_job(layout_job));
                        }

                        // Iterate lines, including trailing empty line after final \n
                        let mut lines_iter = text_str.split('\n').peekable();
                        let mut line_idx = 0;
                        while let Some(line) = lines_iter.next() {
                            let is_last = lines_iter.peek().is_none();

                            // Determine colors
                            let trimmed = line.trim_start();
                            let is_comment = trimmed.starts_with("//");
                            let content_color = if is_comment {
                                comment_color
                            } else {
                                foreground_color
                            };

                            // Add the line content with search highlighting
                            if search_active && !line.is_empty() {
                                // Render line with search match highlighting
                                let line_start_byte = byte_offset;
                                let line_end_byte = byte_offset + line.len();
                                let mut pos = 0usize; // position within the line

                                for &match_start in &search_matches {
                                    let match_end = match_start + search_query_len;
                                    // Check if this match overlaps with current line
                                    if match_end <= line_start_byte || match_start >= line_end_byte {
                                        continue;
                                    }
                                    // Clamp to line boundaries
                                    let local_start = if match_start > line_start_byte {
                                        match_start - line_start_byte
                                    } else {
                                        0
                                    };
                                    let local_end = if match_end < line_end_byte {
                                        match_end - line_start_byte
                                    } else {
                                        line.len()
                                    };

                                    // Append text before match
                                    if local_start > pos {
                                        layout_job.append(
                                            &line[pos..local_start],
                                            if pos == 0 { 0.0 } else { 0.0 },
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color: content_color,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    // Append matched text with highlight background
                                    if local_end > local_start {
                                        layout_job.append(
                                            &line[local_start..local_end],
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color: content_color,
                                                background: highlight_color,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    pos = local_end;
                                }
                                // Append remaining text after last match
                                if pos < line.len() {
                                    layout_job.append(
                                        &line[pos..],
                                        if pos == 0 { 0.0 } else { 0.0 },
                                        egui::TextFormat {
                                            font_id: font_id.clone(),
                                            color: content_color,
                                            ..Default::default()
                                        },
                                    );
                                }
                                // Handle empty line edge case (pos == 0 and line is empty)
                            } else {
                                // No search highlighting - simple append
                                layout_job.append(
                                    line,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: content_color,
                                        ..Default::default()
                                    },
                                );
                            }

                            // Add newline separator (except after the very last segment if text doesn't end with \n)
                            if !is_last {
                                layout_job.append(
                                    "\n",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        ..Default::default()
                                    },
                                );
                                byte_offset += line.len() + 1; // +1 for '\n'
                            } else {
                                byte_offset += line.len();
                            }

                            line_idx += 1;
                        }

                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    // Add invisible blocking layer during middle mouse
                    if middle_button_active {
                        let overlay_rect = ui.max_rect();
                        ui.allocate_rect(overlay_rect, egui::Sense::click_and_drag());
                    }

                    if ui.memory(|mem| mem.has_focus(text_edit_id)) {
                        if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                            Self::handle_tab_key(ui, text_edit_id, false, text, &self.settings, &mut self.is_modified);
                        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)) {
                            Self::handle_tab_key(ui, text_edit_id, true, text, &self.settings, &mut self.is_modified);
                        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
                            Self::handle_enter_key(ui, text_edit_id, text, &mut self.is_modified);
                        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::C)) {
                            Self::handle_copy_key(ui, text_edit_id, text, &mut self.last_copy_time);
                        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::X)) {
                            Self::handle_cut_key(ui, text_edit_id, text, &mut self.is_modified, &mut self.last_copy_time);
                        } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::A)) {
                            // Select all without scrolling view to end
                            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                                let current_pos = state.cursor.char_range()
                                    .map(|r| r.primary.index)
                                    .unwrap_or(0);
                                state.cursor.set_char_range(Some(egui::text::CCursorRange {
                                    primary: egui::text::CCursor::new(current_pos),
                                    secondary: egui::text::CCursor::new(
                                        if current_pos == 0 { text.chars().count() } else { 0 }
                                    ),
                                }));
                                state.store(ui.ctx(), text_edit_id);
                            }
                        }
                    }

                    let available_width = if word_wrap {
                        ui.available_width()
                    } else {
                        f32::INFINITY
                    };

                    let output = egui::TextEdit::multiline(text)
                        .id(text_edit_id)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(available_width)
                        .desired_rows(line_count)
                        .frame(false)
                        .lock_focus(true)
                        .interactive(!middle_button_active)
                        .layouter(&mut |ui, text_str, wrap_width| {
                            layouter(ui, text_str, wrap_width)
                        })
                        .show(ui);

                    let text_rect = output.response.rect;
                    let galley = &output.galley;

                    // AUTO-SCROLL during text selection with left button
                    if pointer.primary_down() && !middle_button_active {
                        if let Some(pointer_pos) = pointer.latest_pos() {
                            let scroll_speed = 15.0;
                            let scroll_margin = 30.0;

                            if pointer_pos.y < scroll_area_rect.top() + scroll_margin {
                                scroll_state.offset.y =
                                    (scroll_state.offset.y - scroll_speed).max(0.0);
                            } else if pointer_pos.y > scroll_area_rect.bottom() - scroll_margin {
                                scroll_state.offset.y += scroll_speed;
                            }

                            if pointer_pos.x < scroll_area_rect.left() + scroll_margin {
                                scroll_state.offset.x =
                                    (scroll_state.offset.x - scroll_speed).max(0.0);
                            } else if pointer_pos.x > scroll_area_rect.right() - scroll_margin {
                                scroll_state.offset.x += scroll_speed;
                            }

                            scroll_state.store(ui.ctx(), scroll_id);
                        }
                    }

                    // Calculate content height
                    let content_height = if !galley.rows.is_empty() {
                        galley.rows.last().unwrap().max_y()
                    } else {
                        0.0
                    };
                    let content_bottom = text_rect.min.y + content_height;

                    // Detect click in empty space BELOW text (only when not middle mouse)
                    if sense_rect.clicked() && !middle_button_active {
                        if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                            if pointer_pos.y > content_bottom && pointer_pos.x >= text_rect.min.x {
                                // Clicked below content - move cursor to end
                                let end_pos = text.chars().count();
                                if let Some(mut state) =
                                    egui::TextEdit::load_state(ui.ctx(), text_edit_id)
                                {
                                    state.cursor.set_char_range(Some(
                                        egui::text::CCursorRange::one(egui::text::CCursor::new(
                                            end_pos,
                                        )),
                                    ));
                                    state.store(ui.ctx(), text_edit_id);
                                }
                                self.highlighted_line = Some(line_count);
                                ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                            } else if pointer_pos.y <= content_bottom {
                                // Clicked within text area - request focus
                                ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                            }
                        }
                    }

                    // Restore full clip rect for drawing line numbers and separator
                    ui.set_clip_rect(full_clip_rect);

                    // Draw line numbers at FIXED position (don't scroll horizontally)
                    if show_line_numbers {
                        let painter = ui.painter();
                        let font_id = egui::FontId::monospace(editor_font_size);
                        let scroll_rect = ui.clip_rect();

                        // Track line numbers using Row::ends_with_newline
                        // Each row that ends_with_newline signals a logical line boundary.
                        // Rows that don't end with newline (and aren't the last) are wrapped.
                        let mut current_line: usize = 1;
                        let mut is_continuation = false;

                        for row in galley.rows.iter() {
                            let line_y = text_rect.min.y + row.min_y();

                            // Only draw if visible
                            if line_y >= scroll_rect.top() - editor_font_size
                                && line_y <= scroll_rect.bottom() + editor_font_size
                            {
                                if !is_continuation {
                                    // First row of a logical line - show line number
                                    let text_color = if Some(current_line) == highlight_line {
                                        foreground_color
                                    } else {
                                        line_number_color
                                    };

                                    painter.text(
                                        egui::pos2(line_numbers_x, line_y),
                                        egui::Align2::RIGHT_TOP,
                                        format!("{}", current_line),
                                        font_id.clone(),
                                        text_color,
                                    );
                                }
                                // Wrapped continuation rows get no line number (blank)
                            }

                            // Advance: if row ends with newline, next row is a new logical line
                            if row.ends_with_newline {
                                current_line += 1;
                                is_continuation = false;
                            } else {
                                // This row is wrapped, so the NEXT row is a continuation
                                is_continuation = true;
                            }
                        }

                        // Separator also at fixed position
                        painter.vline(
                            separator_x,
                            scroll_rect.top()..=scroll_rect.bottom(),
                            ui.visuals().widgets.noninteractive.bg_stroke,
                        );
                    }

                    if output.response.changed() {
                        self.is_modified = true;
                        self.loaded_history_index = None;
                    }

                    // UPDATE HIGHLIGHTING EVERY FRAME when TextEdit has focus
                    if output.response.has_focus() {
                        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                            if let Some(cursor_range) = state.cursor.char_range() {
                                let cursor_char_pos = cursor_range.primary.index;
                                let cursor_byte_pos = char_to_byte_idx(text, cursor_char_pos);

                                // Count '\n' characters before cursor + 1
                                let line_num = text[..cursor_byte_pos]
                                    .as_bytes()
                                    .iter()
                                    .filter(|&&b| b == b'\n')
                                    .count()
                                    + 1;

                                self.highlighted_line = Some(line_num);

                                let char_start =
                                    cursor_range.primary.index.min(cursor_range.secondary.index);
                                let char_end =
                                    cursor_range.primary.index.max(cursor_range.secondary.index);
                                let byte_start = char_to_byte_idx(text, char_start);
                                let byte_end = char_to_byte_idx(text, char_end);
                                self.text_cursor_range = Some(byte_start..byte_end);
                            }
                        }
                    }
                });
            });

        // Save final scroll state
        scroll_output.state.store(ui.ctx(), scroll_id);
    }

    fn handle_tab_key(ui: &egui::Ui, id: egui::Id, unindent: bool, text: &mut String, settings: &crate::settings::Settings, is_modified: &mut bool) {
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                 let start_char = range.primary.index.min(range.secondary.index);
                 let end_char = range.primary.index.max(range.secondary.index);
                 let start_byte = char_to_byte_idx(text, start_char);
                 
                 let tab_str = if settings.use_spaces_for_tabs {
                     " ".repeat(settings.tab_size)
                 } else {
                     "\t".to_string()
                 };
                 
                 if start_char == end_char && !unindent {
                     // Simple insert at cursor
                     if start_byte <= text.len() {
                         text.insert_str(start_byte, &tab_str);
                         let chars_added = tab_str.chars().count();
                         state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                             egui::text::CCursor::new(start_char + chars_added)
                         )));
                         state.store(ui.ctx(), id);
                         *is_modified = true;
                     }
                } else {
                     // Multi-line indent/unindent or single line unindent
                     if unindent && start_char == end_char {
                         let text_ref = &*text;
                         // Find line start/end
                         let line_start = text_ref[..start_byte].rfind('\n').map(|i| i + 1).unwrap_or(0);
                         let line_end = text_ref[start_byte..].find('\n').map(|i| start_byte + i).unwrap_or(text.len());
                         let line = &text_ref[line_start..line_end];
                         
                         let tab_len = if settings.use_spaces_for_tabs { settings.tab_size } else { 1 };
                         
                         // Check for indentation
                         let leading_spaces = line.chars().take_while(|c| *c == ' ').count();
                         
                         if leading_spaces >= tab_len && settings.use_spaces_for_tabs {
                             // Remove one level of spaces
                             text.replace_range(line_start..line_start + tab_len, "");
                             let new_cursor_byte = start_byte.saturating_sub(tab_len).max(line_start);
                             let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                             state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_char))));
                             state.store(ui.ctx(), id);
                             *is_modified = true;
                         } else if leading_spaces > 0 {
                             // Remove leftover spaces
                             text.replace_range(line_start..line_start + leading_spaces, "");
                             let new_cursor_byte = start_byte.saturating_sub(leading_spaces).max(line_start);
                             let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                             state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_char))));
                             state.store(ui.ctx(), id);
                             *is_modified = true;
                         } else if line.starts_with('\t') {
                             // Remove tab
                             text.replace_range(line_start..line_start + 1, "");
                             let new_cursor_byte = start_byte.saturating_sub(1).max(line_start);
                             let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                             state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_cursor_char))));
                             state.store(ui.ctx(), id);
                             *is_modified = true;
                         }
                     }
                }
            }
        }
    }

    fn handle_enter_key(ui: &egui::Ui, id: egui::Id, text: &mut String, is_modified: &mut bool) {
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                let cursor_char_idx = range.primary.index;
                let cursor_byte_idx = char_to_byte_idx(text, cursor_char_idx);
                
                // Find start of current line to read indentation
                let text_ref = &*text;
                let line_start = text_ref[..cursor_byte_idx].rfind('\n').map(|i| i + 1).unwrap_or(0);
                let current_line = &text_ref[line_start..cursor_byte_idx];
                
                // Calculate indentation
                let indent: String = current_line.chars().take_while(|c| c.is_whitespace()).collect();
                
                let insert_str = format!("\n{}", indent);
                
                // Replace selection (if any) or insert
                let start_char = range.primary.index.min(range.secondary.index);
                let end_char = range.primary.index.max(range.secondary.index);
                let start_byte = char_to_byte_idx(text, start_char);
                let end_byte = char_to_byte_idx(text, end_char);
                
                if start_byte <= text.len() && end_byte <= text.len() {
                    text.replace_range(start_byte..end_byte, &insert_str);
                    
                    let new_pos_char = start_char + insert_str.chars().count();
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                        egui::text::CCursor::new(new_pos_char)
                    )));
                    state.store(ui.ctx(), id);
                    *is_modified = true;
                }
            }
        }
    }
    
    fn handle_copy_key(ui: &egui::Ui, id: egui::Id, text: &str, last_copy_time: &mut Option<std::time::Instant>) {
        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                 let start_byte = char_to_byte_idx(text, range.primary.index.min(range.secondary.index));
                 let end_byte = char_to_byte_idx(text, range.primary.index.max(range.secondary.index));
                 
                 if start_byte != end_byte && end_byte <= text.len() {
                     let selected_text = &text[start_byte..end_byte];
                     ui.output_mut(|o| o.copied_text = selected_text.to_string());
                     *last_copy_time = Some(std::time::Instant::now());
                 }
            }
        }
    }

    fn handle_cut_key(ui: &egui::Ui, id: egui::Id, text: &mut String, is_modified: &mut bool, last_copy_time: &mut Option<std::time::Instant>) {
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                 let start_char = range.primary.index.min(range.secondary.index);
                 let end_char = range.primary.index.max(range.secondary.index);
                 let start_byte = char_to_byte_idx(text, start_char);
                 let end_byte = char_to_byte_idx(text, end_char);
                 
                 if start_byte != end_byte && end_byte <= text.len() {
                     let selected_text = &text[start_byte..end_byte];
                     ui.output_mut(|o| o.copied_text = selected_text.to_string());
                     *last_copy_time = Some(std::time::Instant::now());
                     
                     // Delete selection
                     text.replace_range(start_byte..end_byte, "");
                     *is_modified = true;
                     
                     // Update cursor
                     state.cursor.set_char_range(Some(egui::text::CCursorRange::one(
                         egui::text::CCursor::new(start_char)
                     )));
                     state.store(ui.ctx(), id);
                 }
            }
        }
    }
}
