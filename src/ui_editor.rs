use crate::EditorApp;
use eframe::egui;

impl EditorApp {
    pub(crate) fn render_editor(&mut self, ui: &mut egui::Ui) {
        let text = &mut self.document.current_content;
        let line_count = text.lines().count().max(1);
        let editor_font_size = self.settings.editor_font_size;
        let show_line_numbers = self.settings.show_line_numbers;
        let row_height = editor_font_size * 1.4;
        let highlight_line = self.highlighted_line;

        // Capture colors
        let foreground_color = self.current_theme.colors.foreground_color();
        let selection_bg = self.current_theme.colors.selection_color();
        let line_number_color = self.current_theme.colors.line_number_color();
        let comment_color = self.current_theme.colors.comment_color();
        let highlight_bg = selection_bg.linear_multiply(0.2);

        // Poziomy layout: numery linii + editor
        ui.horizontal_top(|ui| {
            // Panel z numerami linii (opcjonalny)
            if show_line_numbers {
                egui::ScrollArea::vertical()
                    .id_salt("line_numbers")
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            for line_num in 1..=line_count {
                                let text_color = if Some(line_num) == highlight_line {
                                    foreground_color
                                } else {
                                    line_number_color
                                };

                                ui.label(
                                    egui::RichText::new(format!("{:4}", line_num))
                                        .color(text_color)
                                        .monospace()
                                        .size(editor_font_size),
                                );
                            }
                        });
                    });

                ui.separator();
            }

            // Główne pole edytora ze scrollowaniem
            egui::ScrollArea::both()
                .id_salt("editor")
                .auto_shrink(false)
                .show(ui, |ui| {
                    // Custom layouter BEZ numerów linii
                    let layouter = |ui: &egui::Ui, text_str: &str, wrap_width: f32| {
                        let mut layout_job = egui::text::LayoutJob::default();
                        let font_id = egui::FontId::monospace(editor_font_size);

                        for (line_idx, line) in text_str.lines().enumerate() {
                            let line_num = line_idx + 1;

                            // Podświetlenie tła linii
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

                            // Kolor treści (wykrywanie komentarzy)
                            let trimmed = line.trim_start();
                            let is_comment = trimmed.starts_with("//");
                            let content_color = if is_comment {
                                comment_color
                            } else {
                                foreground_color
                            };

                            // Dodaj treść linii
                            layout_job.append(
                                line,
                                0.0,
                                egui::TextFormat {
                                    font_id: font_id.clone(),
                                    color: content_color,
                                    ..Default::default()
                                },
                            );

                            // Nowa linia
                            if line_idx < text_str.lines().count() - 1 {
                                layout_job.append(
                                    "\n",
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font_id.clone(),
                                        ..Default::default()
                                    },
                                );
                            }
                        }

                        ui.fonts(|f| f.layout_job(layout_job))
                    };

                    // TextEdit z nieskończoną szerokością dla scroll poziomego
                    let output = egui::TextEdit::multiline(text)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_width(f32::INFINITY) // KLUCZOWE dla scroll poziomo
                        .desired_rows(line_count.max(20))
                        .frame(false)
                        .lock_focus(true)
                        .layouter(&mut |ui, text_str, wrap_width| {
                            layouter(ui, text_str, wrap_width)
                        })
                        .show(ui);

                    // Track changes
                    if output.response.changed() {
                        self.is_modified = true;
                        self.loaded_history_index = None;
                    }

                    // Aktualizacja podświetlonej linii na kliknięciu
                    if output.response.clicked() {
                        if let Some(state) =
                            egui::TextEdit::load_state(ui.ctx(), output.response.id)
                        {
                            if let Some(cursor_range) = state.cursor.char_range() {
                                let cursor_pos = cursor_range.primary.index;

                                // Oblicz numer linii z pozycji kursora
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
