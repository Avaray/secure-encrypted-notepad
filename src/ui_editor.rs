use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    /// Advanced render with automatic current-line highlighting
    pub(crate) fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = text.lines().count().max(1);
        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;

        // Calculate line height
        let row_height = editor_font_size * 1.4;

        // Determine which line to highlight
        let highlight_line = self.highlighted_line;

        let editor_start = ui.cursor().min;
        let mut clicked_line: Option<usize> = None;
        let mut clicked_below_content = false;

        // Build custom editor with line-by-line rendering
        let mut layoutjob = egui::text::LayoutJob::default();
        let font_id = egui::FontId::monospace(editor_font_size);

        let foreground_color = self.current_theme.colors.foreground_color();
        let selection_bg = self.current_theme.colors.selection_color();
        let line_number_color = self.current_theme.colors.line_number_color();

        for (line_idx, line) in text.lines().enumerate() {
            let line_num = line_idx + 1;

            // Highlight current line background
            if Some(line_num) == highlight_line {
                let highlight_bg = selection_bg.linear_multiply(0.2);
                layoutjob.sections.push(egui::text::LayoutSection {
                    leading_space: 0.0,
                    byte_range: layoutjob.text.len()..layoutjob.text.len(),
                    format: egui::TextFormat {
                        font_id: font_id.clone(),
                        background: highlight_bg,
                        ..Default::default()
                    },
                });
            }

            // Line number
            if show_line_numbers {
                let line_num_str = format!("{:4} ", line_num);
                layoutjob.append(
                    &line_num_str,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color: line_number_color,
                        ..Default::default()
                    },
                );
            }

            // Line content
            layoutjob.append(
                line,
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color: foreground_color,
                    ..Default::default()
                },
            );

            layoutjob.append(
                "\n",
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    ..Default::default()
                },
            );
        }

        let _galley = ui.fonts(|f| f.layout_job(layoutjob));

        let output = egui::TextEdit::multiline(text)
            .font(egui::TextStyle::Monospace)
            .desired_width(f32::INFINITY)
            .desired_rows(line_count.max(20))
            .frame(false)
            .lock_focus(true)
            .show(ui);

        // Track changes
        if output.response.changed() {
            self.is_modified = true;
            self.loaded_history_index = None;
        }

        // Detect clicks below the last line of content
        let editor_content_height = line_count as f32 * row_height;
        let editor_rect = egui::Rect::from_min_size(
            editor_start,
            egui::vec2(
                ui.available_width(),
                ui.available_height().max(editor_content_height),
            ),
        );

        // Check if clicked in the editor area below content
        if ui.rect_contains_pointer(editor_rect) {
            if ui.input(|i| i.pointer.primary_clicked()) {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    let relative_y = pos.y - editor_start.y;
                    if relative_y > editor_content_height {
                        // Clicked below content - select last line
                        clicked_below_content = true;
                        output.response.request_focus();
                    } else {
                        // Calculate which line was clicked
                        let line = ((relative_y / row_height) as usize).min(line_count - 1) + 1;
                        clicked_line = Some(line);
                    }
                }
            }
        }

        // Save cursor range for comment toggling
        if let Some(state) = egui::TextEdit::load_state(ui.ctx(), output.response.id) {
            if let Some(cursor_range) = state.cursor.char_range() {
                let start = cursor_range.primary.index.min(cursor_range.secondary.index);
                let end = cursor_range.primary.index.max(cursor_range.secondary.index);
                self.text_cursor_range = Some(start..end);
            }
        }

        // Update clicked line
        if let Some(line) = clicked_line {
            self.highlighted_line = Some(line);
            self.log_info(format!("Line {} selected", line));
        } else if clicked_below_content {
            self.highlighted_line = Some(line_count);
            self.log_info(format!("Line {} selected (last line)", line_count));
        }

        // Manual snapshot button
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("📸 Create Snapshot").clicked() {
                self.document.add_snapshot(None);
                self.is_modified = true;
                self.status_message = "Snapshot created".to_string();
                self.log_info("Manual snapshot created");
            }

            ui.label(
                egui::RichText::new(format!(
                    "History: {}/{}",
                    self.document.get_history().len(),
                    self.document.get_max_history_length()
                ))
                .small()
                .weak(),
            );
        });
    }
}
