use crate::app_helpers::ScrollAreaExt;
use crate::EditorApp;
use eframe::egui;
use sen_core::theme_egui::ThemeColorsExt;

fn char_to_byte_idx(s: &str, char_idx: usize) -> usize {
    s.char_indices()
        .nth(char_idx)
        .map(|(b, _)| b)
        .unwrap_or(s.len())
}

fn byte_to_char_idx(s: &str, byte_idx: usize) -> usize {
    s.get(..byte_idx)
        .map(|sub| sub.chars().count())
        .unwrap_or_else(|| s.chars().count())
}

fn find_urls(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut urls = Vec::new();
    let mut start = 0;
    while let Some(idx) = text[start..]
        .find("http://")
        .or_else(|| text[start..].find("https://"))
    {
        let url_start = start + idx;
        let mut url_end = url_start;
        for (i, c) in text[url_start..].char_indices() {
            if c.is_whitespace()
                || c == '<'
                || c == '>'
                || c == '"'
                || c == '\''
                || c == '`'
                || c == ')'
            {
                url_end = url_start + i;
                break;
            } else {
                url_end = url_start + i + c.len_utf8();
            }
        }
        urls.push(url_start..url_end);
        start = url_end;
    }
    urls
}

impl EditorApp {
    pub(crate) fn render_editor(&mut self, ui: &mut egui::Ui) {
        let line_count = if self.document.current_content.is_empty() {
            1
        } else {
            self.document.current_content.lines().count()
                + if self.document.current_content.ends_with('\n') {
                    1
                } else {
                    0
                }
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
        let find_match_bg = self.current_theme.colors.find_match_bg_color();
        let find_current_bg = self.current_theme.colors.find_current_match_bg_color();
        let current_match_index = self.current_match_index;

        // Capture link state for layouter
        let link_matches: Vec<std::ops::Range<usize>> = find_urls(&self.document.current_content);
        let link_color = self.current_theme.colors.hyperlink_color();

        // Line numbers are right-aligned so digits stack correctly.
        // We set symmetric 8.0 padding here, but note that right-alignment naturally
        // pushes smaller numbers to the left, leaving more empty space on the left side
        // of the column when the total line count has multiple digits.
        let line_number_left_padding = 0.0;
        let line_number_right_padding = 8.0;
        let text_left_padding = 10.0;

        // Calculate precise width for line numbers
        let line_number_width = if show_line_numbers {
            let font_id = egui::FontId::monospace(editor_font_size);
            let max_line_text = format!("{}", line_count);
            let text_width = ui
                .painter()
                .layout_no_wrap(max_line_text, font_id, egui::Color32::WHITE)
                .rect
                .width();
            line_number_left_padding + text_width + line_number_right_padding
        } else {
            0.0
        };

        // Fixed ID for TextEdit
        let text_edit_id = ui.id().with("main_text_editor");
        self.text_edit_id = Some(text_edit_id);

        // Save fixed position for line numbers (left edge of view)
        let editor_left_edge = ui.cursor().left();

        // Separator is at the very right edge of the entire line number block
        let separator_x = if show_line_numbers {
            editor_left_edge + line_number_width
        } else {
            0.0
        };

        // Line numbers are drawn right-aligned starting exactly right-padding pixels before the separator
        let line_numbers_x = if show_line_numbers {
            separator_x - line_number_right_padding
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
        let middle_button_active =
            pointer.middle_down() && !self.document.current_content.is_empty();

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

        // Apply pending horizontal scroll reset BEFORE ScrollArea
        if self.reset_scroll_x_pending {
            scroll_state.offset.x = 0.0;
            scroll_state.store(ui.ctx(), scroll_id);
            self.reset_scroll_x_pending = false;
        }

        let line_height_multiplier = self.settings.line_height;

        // The user requested the cursor/scroll animation to be 75% faster
        let original_animation_time = ui.style().animation_time;
        ui.style_mut().animation_time = original_animation_time * 0.25;

        // ═══════════════════════════════════════════════════════════════════
        // Create a sub-UI for the text area only (starting AFTER line numbers).
        // This ensures the ScrollArea and its scrollbars only span the text area,
        // never overlapping the line number gutter.
        // ═══════════════════════════════════════════════════════════════════
        let text_area_start_x = if show_line_numbers {
            editor_left_edge + line_number_width + text_left_padding
        } else {
            editor_left_edge
        };
        let text_area_rect = egui::Rect::from_min_max(
            egui::pos2(text_area_start_x, full_clip_rect.top()),
            full_clip_rect.max,
        );
        let mut text_ui = ui.new_child(egui::UiBuilder::new().max_rect(text_area_rect));

        // ═══════════════════════════════════════════════════════════════════
        // Handle focus-based shortcuts BEFORE TextEdit (Option 1)
        // Consuming keys here prevents TextEdit from seeing them and adding
        // duplicate characters (like double tabs).
        // ═══════════════════════════════════════════════════════════════════
        if ui.memory(|mem| mem.has_focus(text_edit_id)) {
            let text = &mut self.document.current_content;
            if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)) {
                Self::handle_tab_key(
                    ui,
                    text_edit_id,
                    false,
                    text,
                    &self.settings,
                    &mut self.is_modified,
                );
                self.last_modification_time = std::time::Instant::now();
            } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab)) {
                Self::handle_tab_key(
                    ui,
                    text_edit_id,
                    true,
                    text,
                    &self.settings,
                    &mut self.is_modified,
                );
                self.last_modification_time = std::time::Instant::now();
            } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::C)) {
                Self::handle_copy_key(ui, text_edit_id, text);
            } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::X)) {
                Self::handle_cut_key(ui, text_edit_id, text, &mut self.is_modified);
                self.last_modification_time = std::time::Instant::now();
            } else if ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::A)) {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                    let total_chars = text.chars().count();
                    let current_pos = state
                        .cursor
                        .char_range()
                        .map(|r| r.primary.index)
                        .unwrap_or(0);
                    let (primary, secondary) = if current_pos < total_chars / 2 {
                        (0, total_chars)
                    } else {
                        (total_chars, 0)
                    };
                    state.cursor.set_char_range(Some(egui::text::CCursorRange {
                        primary: egui::text::CCursor::new(primary),
                        secondary: egui::text::CCursor::new(secondary),
                        h_pos: None,
                    }));
                    state.store(ui.ctx(), text_edit_id);
                    self.previous_cursor_byte_pos = Some(char_to_byte_idx(text, primary));
                }
            }
        }

        // Use vertical-only scroll when word wrap is ON (no horizontal content to scroll).
        // Use both axes when word wrap is OFF so long lines can scroll horizontally.
        let scroll_output = if word_wrap {
            egui::ScrollArea::vertical()
        } else {
            egui::ScrollArea::both()
        }
        .id_salt("main_editor")
        .auto_shrink(false)
        .scroll_offset(scroll_state.offset)
        .show_themed(self.current_theme.colors.clone(), &mut text_ui, |ui| {
            let text_ptr = &mut self.document.current_content;
            let scroll_area_rect = ui.clip_rect();

            let layouter = |ui: &egui::Ui, text_str: &str, wrap_width: f32| {
                let mut layout_job = egui::text::LayoutJob::default();
                let font_id = egui::FontId::monospace(editor_font_size);
                let line_height = Some(editor_font_size * line_height_multiplier);

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
                            line_height,
                            valign: egui::Align::Center,
                            ..Default::default()
                        },
                    );
                    return ui.fonts_mut(|f| f.layout_job(layout_job));
                }

                // Iterate lines, including trailing empty line after final \n
                let mut lines_iter = text_str.split('\n').peekable();
                let mut _line_idx = 0;
                while let Some(line) = lines_iter.next() {
                    let is_last = lines_iter.peek().is_none();

                    // Determine colors
                    let trimmed = line.trim_start();
                    let is_comment = !self.settings.comment_prefix.is_empty()
                        && trimmed.starts_with(&self.settings.comment_prefix);
                    let content_color = if is_comment {
                        comment_color
                    } else {
                        foreground_color
                    };

                    if !line.is_empty() {
                        let line_start_byte = byte_offset;
                        let line_end_byte = byte_offset + line.len();

                        // 1. Identify URL segments overlapping the current line
                        let mut line_links = Vec::new();
                        for link_range in &link_matches {
                            if link_range.end <= line_start_byte
                                || link_range.start >= line_end_byte
                            {
                                continue;
                            }
                            let local_start = link_range.start.saturating_sub(line_start_byte);
                            let local_end = if link_range.end < line_end_byte {
                                link_range.end - line_start_byte
                            } else {
                                line.len()
                            };
                            line_links.push(local_start..local_end);
                        }

                        // 2. Build base segments grouping the line into (Content, IsLink) parts
                        let mut base_segments = Vec::new();
                        let mut pos = 0;
                        for link_range in line_links {
                            if link_range.start > pos {
                                base_segments.push((pos..link_range.start, false));
                            }
                            if link_range.end > link_range.start {
                                base_segments.push((link_range.start..link_range.end, true));
                            }
                            pos = link_range.end;
                        }
                        if pos < line.len() {
                            base_segments.push((pos..line.len(), false));
                        }

                        // 3. Render segments, applying search highlights on top if overlapping
                        for (seg_range, is_link) in base_segments {
                            let seg_start_global = line_start_byte + seg_range.start;
                            let seg_end_global = line_start_byte + seg_range.end;

                            let base_color = if is_link && !is_comment {
                                link_color
                            } else {
                                content_color
                            };
                            let underline = if is_link && !is_comment {
                                egui::Stroke::new(1.0, link_color)
                            } else {
                                egui::Stroke::NONE
                            };

                            if search_active {
                                let mut s_pos = seg_range.start;
                                for (match_idx, &m_start) in search_matches.iter().enumerate() {
                                    let m_end = m_start + search_query_len;
                                    if m_end <= seg_start_global || m_start >= seg_end_global {
                                        continue;
                                    }

                                    let is_current = Some(match_idx) == current_match_index;
                                    let match_bg = if is_current {
                                        find_current_bg
                                    } else {
                                        find_match_bg
                                    };

                                    let local_s_start =
                                        s_pos.max(m_start.saturating_sub(line_start_byte));
                                    let local_s_end =
                                        seg_range.end.min(m_end.saturating_sub(line_start_byte));

                                    // Segment BEFORE the highlight
                                    if local_s_start > s_pos {
                                        layout_job.append(
                                            &line[s_pos..local_s_start],
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color: base_color,
                                                line_height,
                                                valign: egui::Align::Center,
                                                underline,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    // Highlighted portion
                                    if local_s_end > local_s_start {
                                        layout_job.append(
                                            &line[local_s_start..local_s_end],
                                            0.0,
                                            egui::TextFormat {
                                                font_id: font_id.clone(),
                                                color: base_color,
                                                line_height,
                                                valign: egui::Align::Center,
                                                background: match_bg,
                                                underline,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    s_pos = local_s_end;
                                }
                                // Segment AFTER the last highlight
                                if s_pos < seg_range.end {
                                    layout_job.append(
                                        &line[s_pos..seg_range.end],
                                        0.0,
                                        egui::TextFormat {
                                            font_id: font_id.clone(),
                                            color: base_color,
                                            line_height,
                                            valign: egui::Align::Center,
                                            underline,
                                            ..Default::default()
                                        },
                                    );
                                }
                            } else {
                                // Fast path: No search highlight filtering needed, write entire segment directly
                                layout_job.append(
                                    &line[seg_range],
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: base_color,
                                        line_height,
                                        valign: egui::Align::Center,
                                        underline,
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                    }

                    // Add newline separator (except after the very last segment if text doesn't end with \n)
                    if !is_last {
                        layout_job.append(
                            "\n",
                            0.0,
                            egui::TextFormat {
                                font_id: font_id.clone(),
                                line_height,
                                valign: egui::Align::Center,
                                ..Default::default()
                            },
                        );
                        byte_offset += line.len() + 1; // +1 for '\n'
                    } else {
                        byte_offset += line.len();
                    }

                    _line_idx += 1;
                }

                ui.fonts_mut(|f| f.layout_job(layout_job))
            };

            // Panning blocking handled by .interactive(!middle_button_active) in TextEdit

            if ui.memory(|mem| mem.has_focus(text_edit_id)) {
                // We will handle Tab/Copy/Cut/SelectAll OUTSIDE the ScrollArea to avoid borrow checker issues
            }

            // Viewport width is now simply the text_area width (the ScrollArea's own width)
            let scrollbar_outer_margin = ui.style().spacing.scroll.bar_outer_margin
                + ui.style().spacing.scroll.bar_width
                + ui.style().spacing.scroll.bar_inner_margin;
            // Added more right margin so text doesn't touch the edge (Editor Comfort Phase 2)
            let text_right_padding = 24.0;
            let viewport_width =
                (text_area_rect.width() - scrollbar_outer_margin - text_right_padding).max(100.0);

            // desired_width controls text wrapping:
            //   word_wrap ON  -> wrap at viewport width
            //   word_wrap OFF -> 0.0 so TextEdit sizes to actual content width;
            //                   min_size ensures it fills the viewport for short/empty files,
            //                   preventing a phantom horizontal scrollbar.
            //                   The layouter already sets wrap.max_width = INFINITY independently.
            let desired_width = if word_wrap { viewport_width } else { 0.0 };

            let min_height = (text_area_rect.height() - scrollbar_outer_margin).max(100.0);

            // HACK: Hide default cursor to draw our own. Use default selection background.
            let original_cursor_color = ui.visuals().text_cursor.stroke.color;
            let original_selection_color = ui.visuals().selection.bg_fill;
            ui.visuals_mut().text_cursor.stroke.color = egui::Color32::TRANSPARENT;
            // Native egui handles selection backgrounds much better with proper Z indexing (behind text)
            ui.visuals_mut().selection.bg_fill = selection_bg.linear_multiply(0.7);

            let output = egui::TextEdit::multiline(text_ptr)
                .id(text_edit_id)
                .font(egui::TextStyle::Monospace)
                .code_editor()
                .desired_width(desired_width)
                .desired_rows(line_count)
                // min_size ensures TextEdit fills the visible area but is NEVER infinite
                .min_size(egui::vec2(viewport_width, min_height))
                // Add explicit margin so text is comfortable by the edges
                .margin(egui::Margin {
                    left: 0,
                    right: text_right_padding as i8,
                    top: 8,
                    bottom: 8,
                })
                .frame(false)
                .lock_focus(true)
                .interactive(!middle_button_active)
                .layouter(&mut |ui, text_str: &dyn egui::TextBuffer, wrap_width| {
                    layouter(ui, text_str.as_str(), wrap_width)
                })
                .show(ui);

            // Handle URL hovering and CTRL + Click
            if output.response.hovered() && ui.input(|i| i.modifiers.ctrl) {
                if let Some(pos) = output.response.hover_pos() {
                    let cursor = output.galley.cursor_from_pos(pos - output.galley_pos);
                    let char_idx = cursor.index;
                    let byte_idx = char_to_byte_idx(text_ptr, char_idx);

                    for link_range in &link_matches {
                        if link_range.contains(&byte_idx) {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            if ui.input(|i| i.pointer.primary_clicked()) {
                                let url = &text_ptr[link_range.clone()];
                                ui.ctx().open_url(egui::OpenUrl {
                                    url: url.to_string(),
                                    new_tab: true,
                                });
                            }
                            break;
                        }
                    }
                }
            }

            // Restore colors for other widgets
            ui.visuals_mut().text_cursor.stroke.color = original_cursor_color;
            ui.visuals_mut().selection.bg_fill = original_selection_color;

            // Draw Custom Cursor
            Self::render_custom_cursor(
                ui,
                &self.settings,
                &output,
                editor_font_size,
                original_cursor_color,
            );

            // Keep focus when interaction happens within the editor (e.g. scrollbars)
            // but not on other focusable widgets.
            if output.response.lost_focus() && ui.memory(|mem| mem.focused().is_none()) {
                let pointer_pos = ui
                    .input(|i| i.pointer.latest_pos())
                    .unwrap_or(egui::pos2(-1.0, -1.0));
                if full_clip_rect.contains(pointer_pos) {
                    ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                }
            }

            // Removed unconditionally frame-based cursor scroll here to prevent fighting manual mouse scrolling.

            let _text_rect = output.response.rect;
            let galley = &output.galley;

            // AUTO-SCROLL during text selection with left button
            if output.response.dragged() && !middle_button_active {
                if let Some(pointer_pos) = pointer.latest_pos() {
                    let scroll_speed = 15.0;
                    let scroll_margin = 30.0;

                    if pointer_pos.y < scroll_area_rect.top() + scroll_margin {
                        scroll_state.offset.y = (scroll_state.offset.y - scroll_speed).max(0.0);
                    } else if pointer_pos.y > scroll_area_rect.bottom() - scroll_margin {
                        scroll_state.offset.y += scroll_speed;
                    }

                    if pointer_pos.x < scroll_area_rect.left() + scroll_margin {
                        scroll_state.offset.x = (scroll_state.offset.x - scroll_speed).max(0.0);
                    } else if pointer_pos.x > scroll_area_rect.right() - scroll_margin {
                        scroll_state.offset.x += scroll_speed;
                    }

                    let content_rect = galley.rect;
                    let max_offset_y = (content_rect.height() - scroll_area_rect.height()).max(0.0);
                    let max_offset_x = (content_rect.width() - scroll_area_rect.width()).max(0.0);
                    scroll_state.offset.y = scroll_state.offset.y.clamp(0.0, max_offset_y);
                    scroll_state.offset.x = scroll_state.offset.x.clamp(0.0, max_offset_x);

                    scroll_state.store(ui.ctx(), scroll_id);
                }
            }

            if output.response.changed() {
                // Logic moved outside
            }

            // UPDATE HIGHLIGHTING AND SCROLLING when TextEdit has focus
            if output.response.has_focus() {
                if let Some(state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                    if let Some(cursor_range_state) = state.cursor.char_range() {
                        let cursor_char_pos = cursor_range_state.primary.index;
                        let cursor_byte_pos = char_to_byte_idx(text_ptr, cursor_char_pos);

                        // Only scroll to cursor manually if the cursor actually changed position since last frame
                        if self.previous_cursor_byte_pos != Some(cursor_byte_pos) {
                            if let Some(cursor_range_galley) = output.cursor_range {
                                let cursor_rect =
                                    output.galley.pos_from_cursor(cursor_range_galley.primary);
                                let screen_cursor_rect =
                                    cursor_rect.translate(output.galley_pos.to_vec2());
                                let padded_rect = screen_cursor_rect.expand(4.0);
                                ui.scroll_to_rect(padded_rect, None);
                            }
                        }
                    }
                }
            }

            let galley_clone = output.galley.clone();
            let separator_stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            let galley_pos = output.galley_pos;
            (
                output,
                scroll_area_rect,
                galley_clone,
                galley_pos,
                separator_stroke,
            )
        });

        // --- POST-PROCESSING OUTSIDE CLOSURE ---
        let (output, _scroll_area_rect, galley_data, galley_pos_data, separator_stroke_data) =
            scroll_output.inner;
        let mut status_update = None;
        let mut warning_log = None;
        let mut needs_search = false;

        {
            let text = &mut self.document.current_content;

            if output.response.changed() {
                self.is_modified = true;
                self.loaded_history_index = None;
                self.last_modification_time = std::time::Instant::now();

                // If the search panel is open, we MUST re-run the search on every change
                // to keep highlights and match counts in sync (handles CTRL + Z fix).
                if self.show_search_panel {
                    needs_search = true;
                }

                // Enforce max lines limit
                if self.settings.max_lines > 0 {
                    let current_lines = text.lines().count();
                    if current_lines > self.settings.max_lines {
                        let mut lines = text
                            .lines()
                            .take(self.settings.max_lines)
                            .collect::<Vec<_>>()
                            .join("\n");
                        if text.ends_with('\n') {
                            lines.push('\n');
                        }
                        *text = lines;
                        status_update = Some(format!(
                            "⚠️ Line limit of {} reached!",
                            self.settings.max_lines
                        ));
                        warning_log = Some(format!(
                            "Line limit ({}) exceeded and content truncated.",
                            self.settings.max_lines
                        ));
                    }
                }
            }
        }

        if needs_search {
            self.perform_search();
        }

        if let Some(msg) = status_update {
            self.status_message = msg;
        }
        if let Some(log) = warning_log {
            self.log_warning(log);
        }

        // Highlight line and cursor tracking
        if output.response.has_focus() {
            let text = &self.document.current_content;
            if let Some(state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                if let Some(cursor_range) = state.cursor.char_range() {
                    let cursor_char_pos = cursor_range.primary.index;
                    let cursor_byte_pos = char_to_byte_idx(text, cursor_char_pos).min(text.len());

                    let line_num = text.as_bytes()[..cursor_byte_pos]
                        .iter()
                        .filter(|&&b| b == b'\n')
                        .count()
                        + 1;
                    self.highlighted_line = Some(line_num);

                    let line_start = text[..cursor_byte_pos]
                        .rfind('\n')
                        .map(|i| i + 1)
                        .unwrap_or(0);
                    let line_end = text[cursor_byte_pos..]
                        .find('\n')
                        .map(|i| cursor_byte_pos + i)
                        .unwrap_or(text.len());
                    let current_line_text = &text[line_start..line_end];

                    if self.previous_cursor_byte_pos != Some(cursor_byte_pos)
                        && current_line_text.trim().is_empty()
                    {
                        self.reset_scroll_x_pending = true;
                    }
                    self.previous_cursor_byte_pos = Some(cursor_byte_pos);

                    let char_start = cursor_range.primary.index.min(cursor_range.secondary.index);
                    let char_end = cursor_range.primary.index.max(cursor_range.secondary.index);
                    let byte_start = char_to_byte_idx(text, char_start);
                    let byte_end = char_to_byte_idx(text, char_end);
                    self.text_cursor_range = Some(byte_start..byte_end);
                }
            }
        }

        // Apply horizontal scroll reset for current frame too
        let mut final_state = scroll_output.state;
        if self.reset_scroll_x_pending {
            final_state.offset.x = 0.0;
        }
        final_state.store(ui.ctx(), scroll_id);

        // ═══════════════════════════════════════════════════════════════════
        // DRAW LINE NUMBERS ON TOP OF SCROLLBARS (Option 3)
        // Everything below renders AFTER the ScrollArea, so it paints
        // on a layer above the scrollbar tracks.
        // ═══════════════════════════════════════════════════════════════════
        let text = &self.document.current_content;
        if show_line_numbers {
            let painter = ui.painter();
            let font_id = egui::FontId::monospace(editor_font_size);

            // Line numbers now sit cleanly on the CentralPanel background without interference.

            // Draw the vertical separator line
            painter.vline(
                separator_x,
                full_clip_rect.top()..=full_clip_rect.bottom(),
                separator_stroke_data,
            );

            // Draw line numbers aligned with galley rows
            let rows = &galley_data.rows;
            let first_visible_row_idx = rows.partition_point(|row| {
                galley_pos_data.y + row.max_y() < full_clip_rect.top() - editor_font_size
            });

            let mut current_line: usize = 1 + rows[..first_visible_row_idx]
                .iter()
                .filter(|r| r.ends_with_newline)
                .count();
            let mut is_continuation = if first_visible_row_idx > 0 {
                !rows[first_visible_row_idx - 1].ends_with_newline
            } else {
                false
            };

            for row in rows.iter().skip(first_visible_row_idx) {
                let line_y = galley_pos_data.y + row.min_y();

                // Stop if we went past the visible bottom
                if line_y > full_clip_rect.bottom() + editor_font_size {
                    break;
                }

                if !is_continuation {
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

                if row.ends_with_newline {
                    current_line += 1;
                    is_continuation = false;
                } else {
                    is_continuation = true;
                }
            }

            // LINE NUMBER CLICK → select entire line
            {
                let gutter_rect = egui::Rect::from_min_max(
                    egui::pos2(editor_left_edge, full_clip_rect.top()),
                    egui::pos2(separator_x, full_clip_rect.bottom()),
                );
                let gutter_response = ui.interact(
                    gutter_rect,
                    ui.id().with("gutter_click"),
                    egui::Sense::click(),
                );

                if gutter_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                }

                if gutter_response.clicked() {
                    if let Some(click_pos) = gutter_response.interact_pointer_pos() {
                        let mut clicked_line: Option<usize> = None;
                        let mut current_line: usize = 1;

                        for row in galley_data.rows.iter() {
                            let row_top = galley_pos_data.y + row.min_y();
                            let row_bottom = galley_pos_data.y + row.max_y();

                            if click_pos.y >= row_top && click_pos.y < row_bottom {
                                clicked_line = Some(current_line);
                                break;
                            }

                            if row.ends_with_newline {
                                current_line += 1;
                            }
                        }

                        if let Some(line_num) = clicked_line {
                            let mut byte_pos = 0usize;
                            let mut cur = 1usize;
                            while cur < line_num {
                                if byte_pos >= text.len() {
                                    break;
                                }
                                match text[byte_pos..].find('\n') {
                                    Some(idx) => {
                                        byte_pos += idx + 1;
                                        cur += 1;
                                    }
                                    None => break,
                                }
                            }
                            let start_byte = byte_pos.min(text.len());
                            let end_byte = if start_byte < text.len() {
                                text[start_byte..]
                                    .find('\n')
                                    .map(|i| start_byte + i)
                                    .unwrap_or(text.len())
                            } else {
                                text.len()
                            };

                            let start_char = byte_to_char_idx(text, start_byte);
                            let end_char = byte_to_char_idx(text, end_byte);

                            if let Some(mut state) =
                                egui::TextEdit::load_state(ui.ctx(), text_edit_id)
                            {
                                state.cursor.set_char_range(Some(egui::text::CCursorRange {
                                    primary: egui::text::CCursor::new(end_char),
                                    secondary: egui::text::CCursor::new(start_char),
                                    h_pos: None,
                                }));
                                state.store(ui.ctx(), text_edit_id);
                            }
                            ui.memory_mut(|mem| mem.request_focus(text_edit_id));
                            self.highlighted_line = Some(line_num);
                        }
                    }
                }
            }
        }

        // ═══════════════════════════════════════════════════════════════════
        // DRAW WHITESPACE CHARACTERS (SPACES, TABS, RETURNS)
        // ═══════════════════════════════════════════════════════════════════
        if self.settings.show_whitespace {
            let mut clip_rect = full_clip_rect;
            if show_line_numbers {
                clip_rect.min.x = separator_x;
            }
            let painter = ui.painter_at(clip_rect);
            let whitespace_color = self.current_theme.colors.whitespace_symbols_color();
            let font_id = egui::FontId::monospace(editor_font_size);

            let start_y = full_clip_rect.top() - galley_pos_data.y;
            let end_y = full_clip_rect.bottom() - galley_pos_data.y;

            // Get visible character range with generous padding
            let start_cursor = galley_data.cursor_from_pos(egui::vec2(0.0, start_y));
            let end_cursor = galley_data.cursor_from_pos(egui::vec2(full_clip_rect.width(), end_y));

            let total_chars = text.chars().count();
            let start_char_idx = start_cursor.index.saturating_sub(100);
            let end_char_idx = (end_cursor.index.saturating_add(200)).min(total_chars);

            let byte_start = char_to_byte_idx(text, start_char_idx);
            let byte_end = char_to_byte_idx(text, end_char_idx);

            for (current_char_idx, (_byte_offset, chr)) in
                (start_char_idx..).zip(text[byte_start..byte_end].char_indices())
            {
                if chr == ' ' || chr == '\t' || chr == '\n' {
                    let cursor_rect1 =
                        galley_data.pos_from_cursor(egui::text::CCursor::new(current_char_idx));
                    let cursor_rect2 =
                        galley_data.pos_from_cursor(egui::text::CCursor::new(current_char_idx + 1));

                    let mut char_width = cursor_rect2.left() - cursor_rect1.left();
                    // If it wraps to next line or is a newline itself, just use a fallback width
                    if char_width < 0.0 || chr == '\n' {
                        char_width = 8.0;
                    }

                    let char_rect = egui::Rect::from_min_size(
                        cursor_rect1.left_top(),
                        egui::vec2(char_width, cursor_rect1.height()),
                    );

                    let screen_rect = char_rect.translate(galley_pos_data.to_vec2());

                    if screen_rect.bottom() >= full_clip_rect.top()
                        && screen_rect.top() <= full_clip_rect.bottom()
                    {
                        // The screen_rect's height spans the full row. Text is rendered at the top.
                        // We center our symbols within the *font* height rather than the full *row* height.
                        let font_h = editor_font_size;
                        let text_center_y = screen_rect.top() + font_h / 2.0;

                        if chr == ' ' {
                            let center_x = screen_rect.center().x;
                            painter.circle_filled(
                                egui::pos2(center_x, text_center_y),
                                1.5,
                                whitespace_color,
                            );
                        } else if chr == '\t' {
                            let y = text_center_y;
                            let px1 = screen_rect.left() + 2.0;
                            let px2 = (screen_rect.right() - 2.0).max(px1 + 8.0);

                            painter.line_segment(
                                [egui::pos2(px1, y), egui::pos2(px2, y)],
                                egui::Stroke::new(1.0, whitespace_color),
                            );
                            let a = 3.0;
                            painter.line_segment(
                                [egui::pos2(px2 - a, y - a), egui::pos2(px2, y)],
                                egui::Stroke::new(1.0, whitespace_color),
                            );
                            painter.line_segment(
                                [egui::pos2(px2 - a, y + a), egui::pos2(px2, y)],
                                egui::Stroke::new(1.0, whitespace_color),
                            );
                        } else if chr == '\n' {
                            // Shift slightly upwards to compensate for font baseline of the return symbol
                            let y = text_center_y - 2.5;
                            painter.text(
                                egui::pos2(screen_rect.left(), y),
                                egui::Align2::LEFT_CENTER,
                                "↵",
                                font_id.clone(),
                                whitespace_color,
                            );
                        }
                    }
                }
            }
        }
        // Restore animation time
        ui.style_mut().animation_time = original_animation_time;
    }

    fn handle_tab_key(
        ui: &egui::Ui,
        id: egui::Id,
        unindent: bool,
        text: &mut String,
        settings: &crate::settings::Settings,
        is_modified: &mut bool,
    ) {
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
                        state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::one(
                                egui::text::CCursor::new(start_char + chars_added),
                            )));
                        state.store(ui.ctx(), id);
                        *is_modified = true;
                    }
                } else {
                    // Multi-line indent/unindent or single line unindent
                    if unindent && start_char == end_char {
                        let text_ref = &*text;
                        // Find line start/end
                        let line_start = text_ref[..start_byte]
                            .rfind('\n')
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        let line_end = text_ref[start_byte..]
                            .find('\n')
                            .map(|i| start_byte + i)
                            .unwrap_or(text.len());
                        let line = &text_ref[line_start..line_end];

                        let tab_len = if settings.use_spaces_for_tabs {
                            settings.tab_size
                        } else {
                            1
                        };

                        // Check for indentation
                        let leading_spaces = line.chars().take_while(|c| *c == ' ').count();

                        if leading_spaces >= tab_len && settings.use_spaces_for_tabs {
                            // Remove one level of spaces
                            text.replace_range(line_start..line_start + tab_len, "");
                            let new_cursor_byte =
                                start_byte.saturating_sub(tab_len).max(line_start);
                            let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(new_cursor_char),
                                )));
                            state.store(ui.ctx(), id);
                            *is_modified = true;
                        } else if leading_spaces > 0 {
                            // Remove leftover spaces
                            text.replace_range(line_start..line_start + leading_spaces, "");
                            let new_cursor_byte =
                                start_byte.saturating_sub(leading_spaces).max(line_start);
                            let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(new_cursor_char),
                                )));
                            state.store(ui.ctx(), id);
                            *is_modified = true;
                        } else if line.starts_with('\t') {
                            // Remove tab
                            text.replace_range(line_start..line_start + 1, "");
                            let new_cursor_byte = start_byte.saturating_sub(1).max(line_start);
                            let new_cursor_char = byte_to_char_idx(text, new_cursor_byte);
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(new_cursor_char),
                                )));
                            state.store(ui.ctx(), id);
                            *is_modified = true;
                        }
                    }
                }
            }
        }
    }

    fn handle_copy_key(ui: &egui::Ui, id: egui::Id, text: &str) {
        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                let start_byte =
                    char_to_byte_idx(text, range.primary.index.min(range.secondary.index));
                let end_byte =
                    char_to_byte_idx(text, range.primary.index.max(range.secondary.index));

                if start_byte != end_byte && end_byte <= text.len() {
                    let selected_text = &text[start_byte..end_byte];
                    ui.ctx().copy_text(selected_text.to_string());
                }
            }
        }
    }

    fn handle_cut_key(ui: &egui::Ui, id: egui::Id, text: &mut String, is_modified: &mut bool) {
        if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(range) = state.cursor.char_range() {
                let start_char = range.primary.index.min(range.secondary.index);
                let end_char = range.primary.index.max(range.secondary.index);
                let start_byte = char_to_byte_idx(text, start_char);
                let end_byte = char_to_byte_idx(text, end_char);

                if start_byte != end_byte && end_byte <= text.len() {
                    let selected_text = &text[start_byte..end_byte];
                    ui.ctx().copy_text(selected_text.to_string());

                    // Delete selection
                    text.replace_range(start_byte..end_byte, "");
                    *is_modified = true;

                    // Update cursor
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(
                            egui::text::CCursor::new(start_char),
                        )));
                    state.store(ui.ctx(), id);
                }
            }
        }
    }
}
