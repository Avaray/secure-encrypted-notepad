use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    pub(crate) fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = text.lines().count().max(1);
        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;
        let highlight_line = self.highlighted_line;

        // Capture colors
        let foreground_color = self.current_theme.colors.foreground_color();
        let selection_bg = self.current_theme.colors.selection_color();
        let line_number_color = self.current_theme.colors.line_number_color();
        let comment_color = self.current_theme.colors.comment_color();
        let highlight_bg = selection_bg.linear_multiply(0.2);

        // Symetryczne paddingi wokół numerów
        let line_number_side_padding = 10.0; // Po lewej i prawej od numeru
        let text_left_padding = 10.0; // Odstęp między separatorem a tekstem

        // Oblicz PRECYZYJNĄ szerokość dla numerów linii
        let line_number_width = if show_line_numbers {
            let font_id = egui::FontId::monospace(editor_font_size);

            // Zmierz rzeczywistą szerokość najdłuższego numeru
            let max_line_text = format!("{}", line_count);
            let text_width =
                ui.fonts(|f| f.glyph_width(&font_id, ' ') * max_line_text.len() as f32);

            // Szerokość obszaru: lewy padding + tekst + prawy padding
            line_number_side_padding + text_width + line_number_side_padding
        } else {
            0.0
        };

        egui::ScrollArea::both()
            .id_salt("main_editor")
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    // Zarezerwuj miejsce dla numerów + separator + odstęp do tekstu
                    if show_line_numbers {
                        ui.add_space(line_number_width + text_left_padding);
                    }

                    // Custom layouter BEZ numerów
                    let layouter = |ui: &egui::Ui, text_str: &str, _wrap_width: f32| {
                        let mut layout_job = egui::text::LayoutJob::default();
                        let font_id = egui::FontId::monospace(editor_font_size);

                        for (line_idx, line) in text_str.lines().enumerate() {
                            let line_num = line_idx + 1;

                            // Podświetlenie tła bieżącej linii
                            if Some(line_num) == highlight_line {
                                layout_job.sections.push(egui::text::LayoutSection {
                                    leading_space: 0.0,
                                    byte_range: layout_job.text.len()..layout_job.text.len(),
                                    format: egui::TextFormat {
                                        font_id: font_id.clone(),
                                        background: highlight_bg,
                                        ..Default::default()
                                    },
                                });
                            }

                            // Kolor treści
                            let trimmed = line.trim_start();
                            let is_comment = trimmed.starts_with("//");
                            let content_color = if is_comment {
                                comment_color
                            } else {
                                foreground_color
                            };

                            layout_job.append(
                                line,
                                0.0,
                                egui::TextFormat {
                                    font_id: font_id.clone(),
                                    color: content_color,
                                    ..Default::default()
                                },
                            );

                            layout_job.append(
                                "\n",
                                0.0,
                                egui::TextFormat {
                                    font_id: font_id.clone(),
                                    ..Default::default()
                                },
                            );
                        }

                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    // Renderuj TextEdit
                    let output = egui::TextEdit::multiline(text)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .desired_rows(line_count)
                        .frame(false)
                        .lock_focus(true)
                        .layouter(&mut |ui, text_str, wrap_width| {
                            layouter(ui, text_str, wrap_width)
                        })
                        .show(ui);

                    // Rysuj numery linii
                    if show_line_numbers {
                        let galley = &output.galley;
                        let painter = ui.painter();
                        let font_id = egui::FontId::monospace(editor_font_size);

                        // Pozycja TextEdit
                        let text_rect = output.response.rect;

                        // Separator z odstępem przed tekstem
                        let separator_x = text_rect.min.x - text_left_padding;

                        // Anchor dla numerów: prawy padding przed separatorem
                        let line_num_anchor_x = separator_x - line_number_side_padding;

                        // Pobierz pełną wysokość ScrollArea
                        let scroll_rect = ui.clip_rect();

                        for (row_idx, row) in galley.rows.iter().enumerate() {
                            let line_num = row_idx + 1;

                            // Y na podstawie rzeczywistej pozycji linii
                            let line_y = text_rect.min.y + row.min_y();

                            let text_color = if Some(line_num) == highlight_line {
                                foreground_color
                            } else {
                                line_number_color
                            };

                            // Narysuj numer wyrównany DO PRAWEJ
                            painter.text(
                                egui::pos2(line_num_anchor_x, line_y),
                                egui::Align2::RIGHT_TOP,
                                format!("{}", line_num),
                                font_id.clone(),
                                text_color,
                            );
                        }

                        // Separator
                        painter.vline(
                            separator_x,
                            scroll_rect.top()..=scroll_rect.bottom(),
                            ui.visuals().widgets.noninteractive.bg_stroke,
                        );
                    }

                    // Track changes
                    if output.response.changed() {
                        self.is_modified = true;
                        self.loaded_history_index = None;
                    }

                    // Aktualizacja podświetlonej linii
                    if output.response.clicked() || output.response.has_focus() {
                        if let Some(state) =
                            egui::TextEdit::load_state(ui.ctx(), output.response.id)
                        {
                            if let Some(cursor_range) = state.cursor.char_range() {
                                let cursor_pos = cursor_range.primary.index;
                                let line_num =
                                    text[..cursor_pos.min(text.len())].lines().count().max(1);

                                self.highlighted_line = Some(line_num);

                                let start =
                                    cursor_range.primary.index.min(cursor_range.secondary.index);
                                let end =
                                    cursor_range.primary.index.max(cursor_range.secondary.index);
                                self.text_cursor_range = Some(start..end);
                            }
                        }
                    }
                });
            });
    }
}
