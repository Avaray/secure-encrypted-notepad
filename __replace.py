import re

with open('src/ui_panels.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# Find the start of the grid
grid_start = content.find('egui::Grid::new("all_theme_colors_grid")')
if grid_start == -1:
    print('Grid not found')
    exit(1)

# Find the .show(ui, |ui| {
show_idx = content.find('.show(ui, |ui| {', grid_start)
block_start = show_idx + len('.show(ui, |ui| {')

# Find the matching closing brace for .show
brace_count = 1
block_end = block_start
while brace_count > 0 and block_end < len(content):
    if content[block_end] == '{':
        brace_count += 1
    elif content[block_end] == '}':
        brace_count -= 1
    block_end += 1

replacement = r'''
                                // --- HELPER CLOSURES FOR NEW OPTIONAL FIELDS ---
                                let mut edit_optional_color = |label: &str, field: &mut Option<[u8; 3]>, default: [u8; 3], id_str: &str, ui: &mut egui::Ui| -> bool {
                                    let mut changed = false;
                                    ui.add(egui::Label::new(label).selectable(false));
                                    let mut current = field.unwrap_or(default);
                                    ui.horizontal(|ui| {
                                        if ui.color_edit_button_srgb(&mut current).changed() {
                                            *field = Some(current);
                                            changed = true;
                                        }
                                        if field.is_some() {
                                            if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                                *field = None;
                                                changed = true;
                                            }
                                        }
                                    });
                                    if let Some(new_color) = render_copy_paste_buttons(
                                        ui,
                                        current,
                                        copied_color,
                                        last_copied_id,
                                        last_copied_time,
                                        egui::Id::new(id_str),
                                    ) {
                                        *field = Some(new_color);
                                        changed = true;
                                    }
                                    ui.end_row();
                                    changed
                                };

                                let mut edit_optional_float = |label: &str, field: &mut Option<f32>, default: f32, range: std::ops::RangeInclusive<f32>, speed: f32, ui: &mut egui::Ui| -> bool {
                                    let mut changed = false;
                                    ui.add(egui::Label::new(label).selectable(false));
                                    let mut current = field.unwrap_or(default);
                                    ui.horizontal(|ui| {
                                        if ui.add(egui::DragValue::new(&mut current).speed(speed).clamp_range(range)).changed() {
                                            *field = Some(current);
                                            changed = true;
                                        }
                                        if field.is_some() {
                                            if ui.button("↺").on_hover_text("Reset to Default").clicked() {
                                                *field = None;
                                                changed = true;
                                            }
                                        }
                                    });
                                    ui.label(""); // Empty label for copy/paste column
                                    ui.end_row();
                                    changed
                                };

                                // --- CORE UI BACKGROUNDS & ACCENTS ---
                                ui.label(egui::RichText::new("--- CORE UI ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new("UI Background:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.background, copied_color, last_copied_id, last_copied_time, egui::Id::new("bg_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Selection Background:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.selection_background, copied_color, last_copied_id, last_copied_time, egui::Id::new("selection_bg_copy")) { theme_changed = true; }
                                ui.end_row();
                                
                                let icon_def = if theme.color_scheme == crate::theme::ColorScheme::Dark { [200, 200, 200] } else { [80, 80, 80] };
                                if edit_optional_color("Icon Default Tint:", &mut theme.colors.icon_color, icon_def, "icon_def_copy", ui) { theme_changed = true; }
                                ui.add(egui::Label::new("Icon Hover Tint:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.icon_hover, copied_color, last_copied_id, last_copied_time, egui::Id::new("icon_hover_copy")) { theme_changed = true; }
                                ui.end_row();

                                // --- TYPOGRAPHY ---
                                ui.label(egui::RichText::new("--- TYPOGRAPHY ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new("UI Foreground:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.foreground, copied_color, last_copied_id, last_copied_time, egui::Id::new("fg_copy")) { theme_changed = true; }
                                ui.end_row();
                                if edit_optional_color("Headings:", &mut theme.colors.heading_text, [255, 255, 255], "heading_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Labels:", &mut theme.colors.label_text, [220, 220, 220], "label_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Weak Text:", &mut theme.colors.weak_text, [150, 150, 150], "weak_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Strong Text:", &mut theme.colors.strong_text, [255, 255, 255], "strong_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Hyperlinks:", &mut theme.colors.hyperlink, [90, 170, 255], "hyperlink_copy", ui) { theme_changed = true; }

                                // --- MAIN TEXT EDITOR ---
                                ui.label(egui::RichText::new("--- MAIN TEXT EDITOR ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Main Editor BG:", &mut theme.colors.editor_background, [10, 10, 10], "editor_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Editor Foreground:", &mut theme.colors.editor_foreground, theme.colors.foreground, "editor_fg_copy", ui) { theme_changed = true; }
                                ui.add(egui::Label::new("Line Numbers:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.line_number, copied_color, last_copied_id, last_copied_time, egui::Id::new("line_num_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Cursor Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.cursor, copied_color, last_copied_id, last_copied_time, egui::Id::new("cursor_copy")) { theme_changed = true; }
                                ui.end_row();
                                if edit_optional_color("Search Highlight:", &mut theme.colors.highlight, theme.colors.cursor, "highlight_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Whitespace Symbols:", &mut theme.colors.whitespace_symbols, [80,80,80], "whitespace_copy", ui) { theme_changed = true; }

                                // --- BUTTONS ---
                                ui.label(egui::RichText::new("--- BUTTONS ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Button Background:", &mut theme.colors.button_bg, [60, 60, 60], "btn_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Button Hover:", &mut theme.colors.button_hover_bg, theme.colors.background, "btn_hover_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Button Active:", &mut theme.colors.button_active_bg, theme.colors.background, "btn_active_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Button Text:", &mut theme.colors.button_fg, theme.colors.foreground, "btn_text_copy", ui) { theme_changed = true; }
                                if edit_optional_float("Button Rounding:", &mut theme.colors.button_rounding, 2.0, 0.0..=20.0, 0.1, ui) { theme_changed = true; }
                                if edit_optional_color("Button Border Color:", &mut theme.colors.button_border_color, [100, 100, 100], "btn_border_copy", ui) { theme_changed = true; }
                                if edit_optional_float("Button Border Width:", &mut theme.colors.button_border_width, 0.0, 0.0..=5.0, 0.05, ui) { theme_changed = true; }
                                if edit_optional_float("Button Padding X:", &mut theme.colors.button_padding_x, 4.0, 0.0..=40.0, 0.5, ui) { theme_changed = true; }
                                if edit_optional_float("Button Padding Y:", &mut theme.colors.button_padding_y, 2.0, 0.0..=40.0, 0.5, ui) { theme_changed = true; }

                                // --- WIDGETS ---
                                ui.label(egui::RichText::new("--- WIDGETS ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Checkbox BG:", &mut theme.colors.checkbox_bg, [50, 50, 50], "chk_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Checkbox Check:", &mut theme.colors.checkbox_check, [200, 200, 200], "chk_check_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Slider Rail:", &mut theme.colors.slider_rail, [60, 60, 60], "slide_rail_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Slider Thumb:", &mut theme.colors.slider_thumb, [180, 180, 180], "slide_thumb_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Scrollbar BG:", &mut theme.colors.scrollbar_bg, [30, 30, 30], "scroll_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Scrollbar Thumb:", &mut theme.colors.scrollbar_thumb, [120, 120, 120], "scroll_thumb_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Tooltip BG:", &mut theme.colors.tooltip_bg, [20, 20, 20], "tooltip_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Tooltip Text:", &mut theme.colors.tooltip_text, [220, 220, 220], "tooltip_txt_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Text Edit BG:", &mut theme.colors.text_edit_bg, [15, 15, 15], "text_edit_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Focus Outline:", &mut theme.colors.focus_outline, [100, 150, 255], "focus_outline_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Selection Text:", &mut theme.colors.selection_text, [255, 255, 255], "sel_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Separator Color:", &mut theme.colors.separator, [80, 80, 80], "sep_copy", ui) { theme_changed = true; }
                                if edit_optional_float("Separator Width:", &mut theme.colors.separator_width, 1.0, 0.0..=10.0, 0.1, ui) { theme_changed = true; }

                                // --- WINDOW & PANELS GEOMETRY ---
                                ui.label(egui::RichText::new("--- PANELS GEOMETRY ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_float("Window Rounding:", &mut theme.colors.window_rounding, 4.0, 0.0..=20.0, 0.1, ui) { theme_changed = true; }
                                if edit_optional_color("Shadow Color:", &mut theme.colors.shadow_color, [0, 0, 0], "shadow_copy", ui) { theme_changed = true; }

                                // --- SYNTAX ALERTS ---
                                ui.label(egui::RichText::new("--- SYNTAX ALERTS ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new("Comment Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.comment, copied_color, last_copied_id, last_copied_time, egui::Id::new("comment_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Success (Green):").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.success, copied_color, last_copied_id, last_copied_time, egui::Id::new("success_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Info (Blue):").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.info, copied_color, last_copied_id, last_copied_time, egui::Id::new("info_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Warning (Orange):").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.warning, copied_color, last_copied_id, last_copied_time, egui::Id::new("warning_copy")) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new("Error (Red):").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.error, copied_color, last_copied_id, last_copied_time, egui::Id::new("error_copy")) { theme_changed = true; }
                                ui.end_row();
                            '''

new_content = content[:block_start] + '\n' + replacement + content[block_end-1:]

with open('src/ui_panels.rs', 'w', encoding='utf-8') as f:
    f.write(new_content)
print('Success')
