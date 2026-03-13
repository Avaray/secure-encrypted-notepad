use crate::app_state::{KeyStatus, LogLevel};
use crate::history::HistoryEntry;
use crate::EditorApp;
use eframe::egui;
impl EditorApp {
    /// Render settings panel
    pub(crate) fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
if !self.settings.hide_panel_headers {
ui.horizontal(|ui| {
ui.heading("Settings");
});
}
egui::ScrollArea::vertical()
.auto_shrink([false, false])
.show(ui, |ui| {
ui.horizontal(|ui| {
if ui.button("Open Setting Folder").on_hover_text("Open settings folder").clicked() {
if let Some(path) = crate::settings::Settings::get_config_dir() {
#[cfg(target_os = "windows")]
{
let _ = std::process::Command::new("explorer").arg(path).spawn();
}
#[cfg(target_os = "linux")]
{
let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}
#[cfg(target_os = "macos")]
{
let _ = std::process::Command::new("open").arg(path).spawn();
}
}
}
ui.separator();
if ui.button("Reset Settings").on_hover_text("Restore all settings to factory defaults.").clicked() {
self.show_reset_confirmation = true;
self.reset_slider_val = 0.0;
}
});
ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 1. SECURITY
// =========================================================================
ui.heading("Security");
ui.horizontal(|ui| {
if ui.button("Set Global Keyfile").clicked() {
if let Some(path) = rfd::FileDialog::new().pick_file() {
self.settings.global_keyfile_path = Some(path.clone());
if self.settings.use_global_keyfile {
self.keyfile_path = Some(path);
}
let _ = self.settings.save();
self.refresh_file_tree(); // Refresh after setting
self.log_info("Global keyfile set");
}
}
        if let Some(path) = self.settings.global_keyfile_path.clone() {
            if ui.button("Use now").on_hover_text("Apply the global keyfile to the current session.").clicked() {
                self.keyfile_path = Some(path);
                self.refresh_file_tree();
                self.log_info("Global keyfile applied");
            }
        }
        if self.settings.global_keyfile_path.is_some() {
            if !self.show_clear_keyfile_confirmation {
                if ui.button("Clear").clicked() {
                    self.show_clear_keyfile_confirmation = true;
                }
            } else {
                ui.label(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color()));
                if ui.button("Yes").clicked() {
                    self.settings.global_keyfile_path = None;
                    self.settings.keyfile_path_encrypted = None;
                    let _ = self.settings.save();
                    self.refresh_file_tree();
                    self.show_clear_keyfile_confirmation = false;
                    self.log_info("Global keyfile cleared");
                }
                if ui.button("No").clicked() {
                    self.show_clear_keyfile_confirmation = false;
                }
            }
        }
});
ui.horizontal(|ui| {
ui.label("Current:");
if let Some(path) = &self.settings.global_keyfile_path {
if self.settings.show_keyfile_paths {
ui.label(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color()));
} else {
ui.label(egui::RichText::new("Secured").color(self.current_theme.colors.success_color()));
}
} else {
    ui.label(egui::RichText::new("None").color(self.current_theme.colors.info_color()));
}
});
if ui
.checkbox(
&mut self.settings.use_global_keyfile,
"Use global keyfile on startup",
)
.changed()
{
let _ = self.settings.save();
}
if ui.checkbox(&mut self.settings.show_keyfile_paths, "Show full keyfile paths")
.on_hover_text("When disabled, full paths to keyfiles are masked as 'Secured' for privacy.")
.changed() {
let _ = self.settings.save();
}
if ui.checkbox(&mut self.settings.show_directory_paths, "Show full directory paths")
.on_hover_text("When disabled, full directory paths (starting dir, file tree header) are masked as 'Secured'.")
.changed() {
let _ = self.settings.save();
}

ui.add_space(8.0);
ui.label(egui::RichText::new("Auto-Backup").strong());
if ui.checkbox(&mut self.settings.auto_backup_enabled, "Enable Auto-Backup on Save")
    .on_hover_text("Automatically saves an encrypted copy to the backup directory on every successful save.")
    .changed() {
    let _ = self.settings.save();
}
ui.horizontal(|ui| {
    if ui.button("Set Backup Directory").clicked() {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.settings.auto_backup_dir = Some(dir.clone());
            let _ = self.settings.save();
            self.log_info("Auto-backup directory set");
        }
    }
    
    if self.settings.auto_backup_dir.is_some() {
        if !self.show_clear_backup_dir_confirmation {
            if ui.button("Clear").clicked() {
                self.show_clear_backup_dir_confirmation = true;
            }
        } else {
            ui.label(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color()));
            if ui.button("Yes").clicked() {
                self.settings.auto_backup_dir = None;
                self.settings.auto_backup_dir_encrypted = None;
                let _ = self.settings.save();
                self.show_clear_backup_dir_confirmation = false;
                self.log_info("Auto-backup directory cleared");
            }
            if ui.button("No").clicked() {
                self.show_clear_backup_dir_confirmation = false;
            }
        }
    }
});
ui.horizontal(|ui| {
    ui.label("Current:");
    if let Some(path) = &self.settings.auto_backup_dir {
        if self.settings.show_directory_paths {
            ui.label(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color()));
        } else {
            ui.label(egui::RichText::new("Secured").color(self.current_theme.colors.success_color()));
        }
    } else {
        ui.label(egui::RichText::new("None").color(self.current_theme.colors.info_color()));
    }
});

#[cfg(target_os = "windows")]
{
    ui.add_space(8.0);
    ui.label(egui::RichText::new("Screen Capture Protection").strong());
    if ui.checkbox(&mut self.settings.screen_capture_protection, "Prevent screen capture")
        .on_hover_text("Prevents screenshots and screen recordings from capturing this window's content. Requires Windows 10 2004 or later.")
        .changed() {
        let _ = self.settings.save();
        self.apply_screen_capture_protection();
    }
}

ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 2. WORKSPACE / FILE TREE
// =========================================================================
ui.heading("Workspace / File Tree");
// Starting directory setting
ui.add_space(4.0);
ui.horizontal_wrapped(|ui| {
ui.label("Starting directory:");
if let Some(ref dir) = self.settings.file_tree_starting_dir {
if self.settings.show_directory_paths {
ui.label(
egui::RichText::new(dir.display().to_string())
.color(self.current_theme.colors.warning_color()),
);
} else {
ui.label(egui::RichText::new("Secured").color(self.current_theme.colors.success_color()));
}
} else {
ui.label(egui::RichText::new("Not set").color(self.current_theme.colors.info_color()));
}
});
ui.horizontal(|ui| {
if ui.button("Set Starting Directory").clicked() {
if let Some(dir) = rfd::FileDialog::new().pick_folder() {
self.settings.file_tree_starting_dir = Some(dir.clone());
self.file_tree_dir = Some(dir);
let _ = self.settings.save();
self.refresh_file_tree();
self.log_info("Starting directory set");
}
}
            if self.settings.file_tree_starting_dir.is_some() {
                if !self.show_clear_workspace_confirmation {
                    if ui.button("Clear").clicked() {
                        self.show_clear_workspace_confirmation = true;
                    }
                } else {
                    ui.label(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color()));
                    if ui.button("Yes").clicked() {
                        self.settings.file_tree_starting_dir = None;
                        self.settings.file_tree_dir_encrypted = None;
                        let _ = self.settings.save();
                        self.log_info("Starting directory cleared");
                        self.show_clear_workspace_confirmation = false;
                    }
                    if ui.button("No").clicked() {
                        self.show_clear_workspace_confirmation = false;
                    }
                }
            }
});
if ui
.checkbox(&mut self.settings.show_subfolders, "Show subfolders")
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}
if ui
.checkbox(&mut self.settings.hide_sen_extension, "Hide .sen extensions")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.tree_style_file_tree, "Use Tree View (expandable)")
.on_hover_text("Display folders hierarchically and toggle expansion on click")
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}
ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 3. EDITOR
// =========================================================================
ui.heading("Editor");
if ui
.checkbox(&mut self.settings.show_line_numbers, "Show line numbers")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.show_whitespace, "Show special symbols")
.on_hover_text("Displays spaces as dots, tabs as arrows, and returns as a return arrow.")
.changed()
{
let _ = self.settings.save();
}
// Cursor settings
ui.horizontal(|ui| {
ui.label("Cursor Shape:");
egui::ComboBox::from_id_salt("cursor_shape_combo")
.selected_text(format!("{:?}", self.settings.cursor_shape))
.show_ui(ui, |ui| {
if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Bar, "Bar").changed() {
let _ = self.settings.save();
self.style_dirty = true;
}
if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Block, "Block").changed() {
let _ = self.settings.save();
self.style_dirty = true;
}
if ui.selectable_value(&mut self.settings.cursor_shape, crate::settings::CursorShape::Underscore, "Underscore").changed() {
let _ = self.settings.save();
self.style_dirty = true;
}
});
});
if ui.checkbox(&mut self.settings.cursor_blink, "Cursor blinking").changed() {
let _ = self.settings.save();
self.style_dirty = true;
}
if ui.checkbox(&mut self.settings.word_wrap, "Word wrap").changed() {
let _ = self.settings.save();
}
ui.horizontal(|ui| {
ui.label("Tab Size:");
if ui
.add(egui::DragValue::new(&mut self.settings.tab_size).range(2..=8))
.changed()
{
let _ = self.settings.save();
}
});
if ui
.checkbox(&mut self.settings.use_spaces_for_tabs, "Use spaces for tabs")
.changed()
{
let _ = self.settings.save();
}

ui.horizontal(|ui| {
    ui.label("Comment Prefix:");
    let mut changed = false;
    let original_inactive_stroke = ui.visuals().widgets.inactive.bg_stroke;
    ui.visuals_mut().widgets.inactive.bg_stroke = ui.visuals().widgets.hovered.bg_stroke;

    let response = ui.add(
        egui::TextEdit::singleline(&mut self.settings.comment_prefix)
            .desired_width(50.0)
            .margin(egui::vec2(6.0, 4.0)) // Adding some margin helps it stand out like a generic border
    );

    ui.visuals_mut().widgets.inactive.bg_stroke = original_inactive_stroke;
    
    if response.changed() {
        changed = true;
    }
    
    if response.lost_focus() {
        if self.settings.comment_prefix.trim().is_empty() {
            self.settings.comment_prefix = "//".to_string();
        } else {
            self.settings.comment_prefix = self.settings.comment_prefix.trim().to_string();
        }
        changed = true; // Save the trimmed state
    }
    
    if changed {
        let _ = self.settings.save();
        self.style_dirty = true;
    }
});
// Max lines
ui.horizontal(|ui| {
ui.label("Max Lines Limit:");
let mut limit_val = self.settings.max_lines;
if ui
.add(
egui::DragValue::new(&mut limit_val)
.speed(10.0)
.range(0..=1000000),
)
.changed()
{
self.settings.max_lines = limit_val;
let _ = self.settings.save();
}
if self.settings.max_lines == 0 {
ui.label(egui::RichText::new("(No limit)").italics().weak());
}
})
.response
.on_hover_text("Maximum number of lines allowed in the editor. Set to 0 to disable the limit.");
// History capacity
ui.horizontal(|ui| {
ui.label("Default history limit:");
if ui
.add(
egui::DragValue::new(&mut self.settings.max_history_length)
.speed(1.0)
.range(1..=1000),
)
.changed()
{
let _ = self.settings.save();
}
});
ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 4. RELIABILITY
// =========================================================================
ui.heading("Reliability");
ui.vertical(|ui| {
ui.set_min_width(ui.available_width());
ui.group(|ui| {
ui.set_min_width(ui.available_width());
ui.label("Auto Save");
if ui
.checkbox(&mut self.settings.auto_save_on_focus_loss, "Auto-save on focus loss")
.on_hover_text("Automatically saves to .autosave.sen when application loses focus.")
.changed()
{
let _ = self.settings.save();
}
                if ui
                    .checkbox(&mut self.settings.auto_save_enabled, "Enable Debounce Auto-save")
                    .on_hover_text("Automatically saves after a period of inactivity while typing.")
                    .changed()
{
let _ = self.settings.save();
}
                ui.horizontal(|ui| {
                    ui.label("Inactivity (seconds):");
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.settings.auto_save_debounce_secs)
                                .range(1..=3600),
                        )
                        .changed()
                    {
                        let _ = self.settings.save();
                    }
                });
});
});
ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 5. APPEARANCE
// =========================================================================
ui.heading("Appearance");
// Theme selection
ui.horizontal(|ui| {
ui.label("Theme:");
egui::ComboBox::from_id_salt("theme_selector")
.selected_text(&self.current_theme.name)
.show_ui(ui, |ui| {
for theme in &self.themes.clone() {
if ui
.selectable_label(
theme.name == self.current_theme.name,
&theme.name,
)
.clicked()
{
self.current_theme = theme.clone();
self.settings.theme_name = theme.name.clone();
self.editing_theme = Some(theme.clone()); // Sync theme editor
self.apply_theme(ui.ctx());
let _ = self.settings.save();
}
}
});
if ui.button("🔄 Refresh").clicked() {
self.themes = crate::theme::load_themes();
self.log_info("Themes refreshed");
}
});
ui.separator();
// UI font family with keyboard navigation
ui.horizontal(|ui| {
ui.label("UI Font:");
let _response = egui::ComboBox::from_id_salt("ui_font_selector")
.selected_text(&self.available_fonts[self.ui_font_index])
.show_ui(ui, |ui| {
let mut changed = false;
if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
if self.ui_font_index > 0 {
self.ui_font_index -= 1;
changed = true;
}
}
if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
if self.ui_font_index < self.available_fonts.len() - 1 {
self.ui_font_index += 1;
changed = true;
}
}
egui::ScrollArea::vertical()
.max_height(300.0)
.auto_shrink([false, false])
.show(ui, |ui| {
let current_time = ui.input(|i| i.time);
let is_first_frame = ui.ctx().data_mut(|d| {
let last_time = d.get_temp::<f64>(ui.id());
d.insert_temp(ui.id(), current_time);
match last_time {
Some(last) => current_time > last + 0.1,
None => true,
}
});
for (idx, font) in self.available_fonts.iter().enumerate() {
let is_selected = idx == self.ui_font_index;
let response = ui.selectable_label(is_selected, font);
if is_selected && is_first_frame {
response.scroll_to_me(Some(egui::Align::Center));
}
if response.clicked() || (is_selected && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
self.ui_font_index = idx;
changed = true;
ui.close_kind(egui::UiKind::Menu);
}
}
});
if changed {
self.settings.ui_font_family =
self.available_fonts[self.ui_font_index].clone();
let _ = self.settings.save();
self.style_dirty = true;
self.log_info(format!(
"UI font changed to: {}",
self.settings.ui_font_family
));
}
});
});
// UI font size
ui.horizontal(|ui| {
ui.label("UI Font Size:");
if ui
.add(
egui::DragValue::new(&mut self.settings.ui_font_size)
.speed(0.5)
.range(8.0..=128.0),
)
.changed()
{
self.settings.validate_font_sizes();
let _ = self.settings.save();
self.style_dirty = true;
}
});
ui.separator();
// Editor font family
ui.horizontal(|ui| {
ui.label("Editor Font:");
let _response = egui::ComboBox::from_id_salt("editor_font_selector")
.selected_text(&self.available_fonts[self.editor_font_index])
.show_ui(ui, |ui| {
let mut changed = false;
if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
if self.editor_font_index > 0 {
self.editor_font_index -= 1;
changed = true;
}
}
if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
if self.editor_font_index < self.available_fonts.len() - 1 {
self.editor_font_index += 1;
changed = true;
}
}
egui::ScrollArea::vertical()
.max_height(300.0)
.auto_shrink([false, false])
.show(ui, |ui| {
let current_time = ui.input(|i| i.time);
let is_first_frame = ui.ctx().data_mut(|d| {
let last_time = d.get_temp::<f64>(ui.id());
d.insert_temp(ui.id(), current_time);
match last_time {
Some(last) => current_time > last + 0.1,
None => true,
}
});
for (idx, font) in self.available_fonts.iter().enumerate() {
let is_selected = idx == self.editor_font_index;
let response = ui.selectable_label(is_selected, font);
if is_selected && is_first_frame {
response.scroll_to_me(Some(egui::Align::Center));
}
if response.clicked() || (is_selected && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
self.editor_font_index = idx;
changed = true;
ui.close_kind(egui::UiKind::Menu);
}
}
});
if changed {
self.settings.editor_font_family =
self.available_fonts[self.editor_font_index].clone();
let _ = self.settings.save();
self.style_dirty = true;
self.log_info(format!(
"Editor font changed to: {}",
self.settings.editor_font_family
));
}
});
});
// Editor font size
ui.horizontal(|ui| {
ui.label("Editor Font Size:");
if ui
.add(
egui::DragValue::new(&mut self.settings.editor_font_size)
.speed(0.5)
.range(8.0..=128.0),
)
.changed()
{
self.settings.validate_font_sizes();
let _ = self.settings.save();
self.style_dirty = true;
}
});
// Line height multiplier
ui.horizontal(|ui| {
ui.label("Line Height:");
if ui
.add(
egui::Slider::new(&mut self.settings.line_height, 1.0..=2.5)
.step_by(0.05)
.max_decimals(2)
.min_decimals(2)
.text("x"),
)
.changed()
{
let _ = self.settings.save();
}
});
// Toolbar icon size
ui.horizontal(|ui| {
ui.label("Toolbar Icon Size:");
if ui
.add(
egui::DragValue::new(&mut self.settings.toolbar_icon_size)
.speed(1.0)
.range(12.0..=64.0),
)
.changed()
{
let _ = self.settings.save();
}
});
ui.horizontal(|ui| {
ui.label("Toolbar Position:");
let mut changed = false;
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Top, "Top").changed();
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Left, "Left").changed();
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Right, "Right").changed();
if changed {
let _ = self.settings.save();
}
});
if ui
.checkbox(&mut self.settings.hide_panel_headers, "Hide panel headers")
.on_hover_text("If enabled, panel titles (like 'Settings', 'Files', etc.) will be hidden for a cleaner look.")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.preserve_all_panels, "Preserve all panels at launch")
.on_hover_text("If enabled, all open panels (Settings, History, etc.) will be restored on next launch.")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.start_maximized, "Start Maximized")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.remember_zen_mode, "Remember Zen mode")
.on_hover_text("If enabled, Zen mode will be restored on next launch if it was active when closing.")
.changed()
{
let _ = self.settings.save();
}
ui.add_space(4.0);
});
});
    }
    /// Render history panel
    pub(crate) fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        // Zbierz wszystkie dane PRZED closure
        let visible_history: Vec<(usize, HistoryEntry)> = self
            .document
            .get_visible_history()
            .into_iter()
            .map(|(idx, entry)| (idx, entry.clone()))
            .collect();
        let history_len = visible_history.len();
        let doc_max_limit = self.document.get_max_history_length();

        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading("History");
            }
            ui.horizontal(|ui| {
                ui.label("Max History for this file:");
                let mut temp_limit = doc_max_limit;
                if ui
                    .add(
                        egui::DragValue::new(&mut temp_limit)
                            .speed(0.1)
                            .range(1..=1000),
                    )
                    .changed()
                {
                    self.document.set_max_history_length(temp_limit);
                    self.is_modified = true;
                    self.log_info(format!("Document history limit set to {}", temp_limit));
                }
            });
            let history_status_color = if history_len > doc_max_limit {
                self.current_theme.colors.warning_color()
            } else {
                ui.visuals().widgets.noninteractive.fg_stroke.color // Default weak color
            };
            
            ui.label(
                egui::RichText::new(format!(
                    "Current: {}/{} entries",
                    history_len, doc_max_limit
                ))
                .color(history_status_color),
            );

            if history_len > doc_max_limit {
                let to_delete = history_len - doc_max_limit;
                ui.label(
                    egui::RichText::new(format!("{} entries will be deleted upon save", to_delete))
                        .color(self.current_theme.colors.warning_color())
                        .small(),
                );
            }
            ui.separator();

            let history_area_id = ui.id().with("history_focus_area");
            let has_focus = ui.memory(|mem| mem.has_focus(history_area_id));

            if history_len > 0 {
                // keyboard navigation IF focused
                if has_focus {
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)) {
                        if let Some(loaded_idx) = self.loaded_history_index {
                            if let Some(pos) = visible_history.iter().position(|(idx, _)| *idx == loaded_idx) {
                                if pos < visible_history.len() - 1 {
                                    let (new_idx, _) = visible_history[pos + 1];
                                    self.load_history_version(new_idx);
                                    self.loaded_history_index = Some(new_idx);
                                }
                            }
                        } else if let Some((idx, _)) = visible_history.last() {
                            self.load_history_version(*idx);
                            self.loaded_history_index = Some(*idx);
                        }
                    }
                    if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown)) {
                        if let Some(loaded_idx) = self.loaded_history_index {
                            if let Some(pos) = visible_history.iter().position(|(idx, _)| *idx == loaded_idx) {
                                if pos > 0 {
                                    let (new_idx, _) = visible_history[pos - 1];
                                    self.load_history_version(new_idx);
                                    self.loaded_history_index = Some(new_idx);
                                }
                            }
                        }
                    }
                }

                ui.horizontal(|ui| {
                    if self.show_clear_history_confirmation {
                        ui.label(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color()));
                        if ui.button("Yes").clicked() {
                            self.clear_all_history();
                            self.loaded_history_index = None;
                            self.show_clear_history_confirmation = false;
                        }
                        if ui.button("No").clicked() {
                            self.show_clear_history_confirmation = false;
                        }
                    } else if ui.button("🗑 Clear All History").clicked() {
                        self.show_clear_history_confirmation = true;
                    }
                });
            }

            // High-level frame to catch clicks for focus
            let stroke = if has_focus {
                egui::Stroke::new(1.0, self.current_theme.colors.cursor_color())
            } else {
                egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)
            };

            egui::Frame::NONE
                .inner_margin(4.0)
                .stroke(stroke)
                .corner_radius(4.0)
                .show(ui, |ui| {
                    // Interaction to gain focus
                    let rect = ui.available_rect_before_wrap();
                    let response = ui.interact(rect, history_area_id, egui::Sense::click());
                    if response.clicked() {
                        ui.memory_mut(|mem| mem.request_focus(history_area_id));
                    }

                    // Prevent arrow keys from moving focus out of the history panel when focused
                    if has_focus {
                        ui.memory_mut(|mem| {
                            mem.set_focus_lock_filter(
                                history_area_id,
                                egui::EventFilter {
                                    tab: true,
                                    horizontal_arrows: true,
                                    vertical_arrows: true,
                                    escape: false,
                                },
                            );
                        });
                    }

                    egui::ScrollArea::vertical()
                        .id_salt("history_scroll_area")
                        .show(ui, |ui| {
                            if history_len == 0 {
                                ui.label("No history");
                            } else {
                                let to_delete_count = if history_len > doc_max_limit {
                                    history_len - doc_max_limit
                                } else {
                                    0
                                };

                                for (v_idx, (original_index, entry)) in visible_history.iter().enumerate().rev() {
                                    let is_loaded = self.loaded_history_index == Some(*original_index);
                                    let will_be_deleted = v_idx < to_delete_count;
                                    
                                    ui.horizontal(|ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;
                                        if is_loaded {
                                            ui.label(
                                                egui::RichText::new("▶")
                                                    .color(self.current_theme.colors.success_color()),
                                            );
                                        } else {
                                            ui.add_space(8.0);
                                        }
                                        
                                        let text = format!("{} - {}", entry.display_timestamp(), entry.display_size());
                                        let mut rich_text = egui::RichText::new(text);
                                        if will_be_deleted {
                                            rich_text = rich_text.color(self.current_theme.colors.warning_color());
                                        }
                                        
                                        let label_res = ui.selectable_label(is_loaded, rich_text);
                                        
                                        if label_res.clicked() {
                                            self.load_history_version(*original_index);
                                            self.loaded_history_index = Some(*original_index);
                                            ui.memory_mut(|mem| mem.request_focus(history_area_id));
                                        }
                                        
                                        if is_loaded && has_focus {
                                            label_res.scroll_to_me(None);
                                        }
                                        
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("🗑").on_hover_text("Delete").clicked() {
                                                self.delete_history_entry(*original_index);
                                                if self.loaded_history_index == Some(*original_index) {
                                                    self.loaded_history_index = None;
                                                }
                                            }
                                        });
                                    });
                                    ui.add_space(2.0);
                                }
                            }
                        });
                });
        });
    }
    /// Render debug panel
    pub(crate) fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading("Debug Log");
            }
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.debug_log.clear();
                }
                ui.label(format!("Entries: {}", self.debug_log.len()));
            });
            ui.separator();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in &self.debug_log {
                        let color = match entry.level {
                            LogLevel::Info => self.current_theme.colors.info_color(),
                            LogLevel::Success => self.current_theme.colors.success_color(),
                            LogLevel::Warning => self.current_theme.colors.warning_color(),
                            LogLevel::Error => self.current_theme.colors.error_color(),
                        };
                        ui.colored_label(color, entry.display());
                    }
                });
        });
    }
    /// Render file tree panel
    pub(crate) fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading("Files");
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if let Some(dir) = &self.file_tree_dir {
                        if self.settings.show_directory_paths {
                            ui.label(dir.display().to_string());
                            ui.separator();
                        }
                        let available_width = ui.available_width();
                        ui.set_max_width(available_width);
                        let entries = self.file_tree_entries.clone();
                        let tree_on = self.settings.tree_style_file_tree;
                        let tree_indent: f32 = if tree_on { 24.0 } else { 0.0 };
                        let panel_left = ui.cursor().left();
                        let line_color = ui.visuals().weak_text_color();
                        let stroke = egui::Stroke::new(1.0, line_color);

                        // Collect mid_y positions for tree lines (two-pass to avoid overlap)
                        struct RowData {
                            depth: usize,
                            is_dir: bool,

                            bottom_y: f32,
                            mid_y: f32,
                        }
                        
                        let mut row_infos = Vec::new();

                        for (_i, entry) in entries.iter().enumerate() {
                            let top_y = ui.cursor().top();
                            let path = &entry.path;
                            let filename = path.file_name().unwrap_or_default().to_string_lossy();
                            let depth = entry.depth;

                            ui.horizontal(|ui| {
                                if tree_on {
                                    // Spacing for depth
                                    ui.add_space(depth as f32 * tree_indent);
                                }
                                ui.spacing_mut().item_spacing.x = 4.0;
                                
                                if entry.is_dir {
                                    // Directory
                                    let is_parent = self
                                        .file_tree_dir
                                        .as_ref()
                                        .and_then(|d| d.parent())
                                        .map(|p| p == path)
                                        .unwrap_or(false);
                                        
                                    let display_name = if is_parent {
                                        "📁 ..".to_string()
                                    } else {
                                        format!(
                                            "{} {}",
                                            if entry.is_expanded { "📂" } else { "📁" },
                                            filename
                                        )
                                    };
                                    
                                    if ui
                                        .add(egui::Button::new(&display_name).truncate())
                                        .clicked()
                                    {
                                        if is_parent {
                                            self.change_directory(path.clone());
                                        } else if tree_on {
                                            // Toggle expansion
                                            if entry.is_expanded {
                                                self.expanded_directories.remove(path);
                                            } else {
                                                self.expanded_directories.insert(path.clone());
                                            }
                                            self.refresh_file_tree(); // Re-trigger build
                                        } else {
                                            self.change_directory(path.clone());
                                        }
                                    }
                                } else {
                                    // File
                                    let mut display_name = if self.settings.hide_sen_extension
                                        && filename.to_lowercase().ends_with(".sen")
                                    {
                                        filename[..filename.len() - 4].to_string()
                                    } else {
                                        filename.to_string()
                                    };

                                    if filename.ends_with(".sen") {
                                        let status = self
                                            .file_access_cache
                                            .get(path)
                                            .cloned()
                                            .unwrap_or(KeyStatus::Unknown);
                                            
                                        if tree_on {
                                            // Status indicator logic (manually drawn inside button frame)
                                            let color = match status {
                                                KeyStatus::Decryptable => self.current_theme.colors.success_color(),
                                                KeyStatus::WrongKey => self.current_theme.colors.error_color(),
                                                _ => ui.visuals().weak_text_color(),
                                            };

                                            let mut job = egui::text::LayoutJob::default();
                                            let font_id = egui::TextStyle::Button.resolve(ui.style());
                                            
                                            // Add 3 spaces to make room for the dot at the beginning
                                            job.append("   ", 0.0, egui::text::TextFormat {
                                                font_id: font_id.clone(),
                                                ..Default::default()
                                            });
                                            job.append(&display_name, 0.0, egui::text::TextFormat {
                                                color: ui.visuals().text_color(),
                                                font_id,
                                                ..Default::default()
                                            });

                                            let button_resp = ui.add(egui::Button::new(job).truncate());
                                            
                                            // Draw the dot manually on top of the button's rectangle
                                            let dot_radius = 4.0;
                                            let dot_center = egui::pos2(
                                                button_resp.rect.left() + 10.0,
                                                button_resp.rect.center().y
                                            );
                                            ui.painter().circle_filled(dot_center, dot_radius, color);

                                            if button_resp.clicked() {
                                                self.open_file(path.clone());
                                            }
                                        } else {
                                            // Simple View (Icon + Text)
                                            let icon_color = match status {
                                                KeyStatus::Decryptable => self.current_theme.colors.success_color(),
                                                KeyStatus::WrongKey => self.current_theme.colors.error_color(),
                                                _ => ui.visuals().text_color(),
                                            };
                                            let icon_size = ui.text_style_height(&egui::TextStyle::Body);
                                            ui.allocate_ui(egui::vec2(icon_size, icon_size), |ui| {
                                                ui.centered_and_justified(|ui| {
                                                    ui.add(egui::Image::new(&self.icons.key).tint(icon_color).max_width(icon_size));
                                                });
                                            });
                                            if ui.add(egui::Button::new(&display_name).truncate()).clicked() {
                                                self.open_file(path.clone());
                                            }
                                        }
                                    } else {
                                        // Non-SEN file
                                        if !tree_on {
                                            let icon_size = ui.text_style_height(&egui::TextStyle::Body);
                                            ui.allocate_ui(egui::vec2(icon_size, icon_size), |ui| {
                                                ui.centered_and_justified(|ui| {
                                                    ui.label("📄");
                                                });
                                            });
                                        } else {
                                            display_name = format!("📄 {}", display_name);
                                        }
                                        if ui.add(egui::Button::new(&display_name).truncate()).clicked() {
                                            self.open_file(path.clone());
                                        }
                                    }
                                }
                            });

                            if tree_on {
                                let bottom_y = ui.cursor().top();
                                let actual_bottom_y = bottom_y - ui.spacing().item_spacing.y;
                                row_infos.push(RowData {
                                    depth,
                                    is_dir: entry.is_dir,
                                    bottom_y: actual_bottom_y,
                                    mid_y: (top_y + actual_bottom_y) / 2.0,
                                });
                            }
                        }

                        // Pass 2: Draw the tree geometry based on layout positions
                        if tree_on && !row_infos.is_empty() {
                            let painter = ui.painter();
                            
                            for i in 0..row_infos.len() {
                                let row = &row_infos[i];
                                
                                // Draw horizontal branch
                                if row.depth > 0 {
                                    let parent_depth = row.depth - 1;
                                    let branch_start_x = panel_left + (parent_depth as f32) * tree_indent + 11.0;
                                    let branch_end_x = panel_left + (row.depth as f32) * tree_indent - 1.0;
                                    painter.line_segment([egui::pos2(branch_start_x, row.mid_y), egui::pos2(branch_end_x, row.mid_y)], stroke);
                                }
                                
                                // Draw vertical drop line to children
                                if row.is_dir {
                                    // Find the last direct child (depth == row.depth + 1)
                                    let mut last_child_mid_y = None;
                                    for j in (i + 1)..row_infos.len() {
                                        if row_infos[j].depth <= row.depth {
                                            break;
                                        }
                                        if row_infos[j].depth == row.depth + 1 {
                                            last_child_mid_y = Some(row_infos[j].mid_y);
                                        }
                                    }
                                    
                                    if let Some(end_y) = last_child_mid_y {
                                        let drop_x = panel_left + (row.depth as f32) * tree_indent + 11.0;
                                        painter.line_segment([egui::pos2(drop_x, row.bottom_y), egui::pos2(drop_x, end_y)], stroke);
                                    }
                                }
                            }
                        }
                    } else {
                        ui.label("No directory opened");
                        if ui.button("Open Directory").clicked() {
                            self.open_directory();
                        }
                    }
                });
        });
    }
    /// Render theme editor panel
    pub(crate) fn render_theme_editor_panel(&mut self, ui: &mut egui::Ui) {
        let mut theme_to_save: Option<crate::theme::Theme> = None;
        let mut should_reset = false;
        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading("Theme Editor");
            }
            // Top bar: Theme selector and actions
            ui.horizontal(|ui| {
                let current_name = self
                    .editing_theme
                    .as_ref()
                    .map(|t| t.name.clone())
                    .unwrap_or_default();
                egui::ComboBox::from_id_salt("theme_editor_selector")
                    .selected_text(&current_name)
                    .show_ui(ui, |ui| {
                        for theme in &self.themes {
                            if ui
                                .selectable_label(theme.name == current_name, &theme.name)
                                .clicked()
                            {
                                self.editing_theme = Some(theme.clone());
                                self.current_theme = theme.clone();
                                self.settings.theme_name = theme.name.clone();
                                self.apply_theme(ui.ctx());
                                let _ = self.settings.save();
                            }
                        }
                    });
                if ui.button("➕ New").clicked() {
                    let mut new_theme = self.current_theme.clone();
                    new_theme.name = format!("{} (Copy)", new_theme.name);
                    self.editing_theme = Some(new_theme);
                }
                // Delete button with confirmation
                if let Some(theme) = &self.editing_theme {
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    if !is_builtin {
                        if ui.button("🗑 Delete").clicked() {
                            // This actually needs a way to confirm. For now, simple delete.
                            // In a real app we'd show a modal.
                            // Since we don't have modals easily, we'll just delete and select default.
                            let _ = crate::theme::delete_theme(&theme.name);
                            self.themes = crate::theme::load_themes(); // Reload
                            self.editing_theme = Some(crate::theme::Theme::dark());
                            // Reset to safe default
                        }
                    }
                }
            });
            ui.separator();
            if let Some(ref mut theme) = self.editing_theme {
                let mut theme_changed = false;
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut theme.name);
                });
                ui.horizontal(|ui| {
                    ui.label("Base Scheme:");
                    egui::ComboBox::from_id_salt("color_scheme_selector")
                        .selected_text(format!("{:?}", theme.color_scheme))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Dark),
                                    "Dark",
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Dark;
                                theme_changed = true;
                            }
                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Light),
                                    "Light",
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Light;
                                theme_changed = true;
                            }
                        });
                });
                ui.separator();
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(ui.available_height() - 80.0)
                    .show(ui, |ui| {
                        ui.heading("Colors");
                        ui.add_space(4.0);
                        egui::Grid::new("all_theme_colors_grid")
                            .num_columns(2)
                            .spacing([20.0, 4.0])
                            .striped(false)
                            .show(ui, |ui| {
                                // Background
                                ui.label("Background:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Foreground
                                ui.label("Foreground:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.foreground)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Editor Foreground
                                ui.label("Editor Foreground:");
                                ui.horizontal(|ui| {
                                    let mut editor_fg = theme
                                        .colors
                                        .editor_foreground
                                        .unwrap_or(theme.colors.foreground);
                                    if ui.color_edit_button_srgb(&mut editor_fg).changed() {
                                        theme.colors.editor_foreground = Some(editor_fg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.editor_foreground.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to match UI Foreground")
                                            .clicked()
                                        {
                                            theme.colors.editor_foreground = None;
                                            theme_changed = true;
                                        }
                                    }
                                    if theme.colors.editor_foreground.is_none() {
                                        ui.weak("(Same as UI)");
                                    }
                                });
                                ui.end_row();
                                // Panel Background
                                ui.label("Panel Background:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.panel_background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Surface
                                ui.label("Surface:");
                                ui.horizontal(|ui| {
                                    let mut surf = theme
                                        .colors
                                        .surface
                                        .unwrap_or(theme.colors.panel_background);
                                    if ui.color_edit_button_srgb(&mut surf).changed() {
                                        theme.colors.surface = Some(surf);
                                        theme_changed = true;
                                    }
                                    if theme.colors.surface.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.surface = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Surface Highlight
                                ui.label("Surface Highlight:");
                                ui.horizontal(|ui| {
                                    let mut surf_h =
                                        theme.colors.surface_highlight.unwrap_or_else(|| {
                                            theme
                                                .colors
                                                .surface
                                                .unwrap_or(theme.colors.panel_background)
                                        });
                                    if ui.color_edit_button_srgb(&mut surf_h).changed() {
                                        theme.colors.surface_highlight = Some(surf_h);
                                        theme_changed = true;
                                    }
                                    if theme.colors.surface_highlight.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.surface_highlight = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                ui.label("");
                                ui.label("");
                                ui.end_row();
                                // Selection Background
                                ui.label("Selection Background:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.selection_background)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Cursor
                                ui.label("Cursor Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.cursor)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Icon Hover
                                ui.label("Icon Hover Tint:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.icon_hover)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Icon Default (New)
                                ui.label("Icon Default Tint:");
                                let mut icon_def = theme.colors.icon_color.unwrap_or(
                                    if theme.color_scheme == crate::theme::ColorScheme::Dark {
                                        [200, 200, 200]
                                    } else {
                                        [80, 80, 80]
                                    },
                                );
                                if ui.color_edit_button_srgb(&mut icon_def).changed() {
                                    theme.colors.icon_color = Some(icon_def);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Highlight (New)
                                ui.label("Search Highlight:");
                                let mut highlight = theme.colors.highlight.unwrap_or(
                                    theme.colors.cursor, // fallback
                                );
                                ui.horizontal(|ui| {
                                    if ui.color_edit_button_srgb(&mut highlight).changed() {
                                        theme.colors.highlight = Some(highlight);
                                        theme_changed = true;
                                    }
                                    if theme.colors.highlight.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.highlight = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Button Background
                                ui.label("Button Background:");
                                ui.horizontal(|ui| {
                                    let mut bg = theme.colors.button_bg.unwrap_or([60, 60, 60]); // Approx default
                                    if ui.color_edit_button_srgb(&mut bg).changed() {
                                        theme.colors.button_bg = Some(bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_bg.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.button_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Button Hover Background
                                ui.label("Button Hover:");
                                ui.horizontal(|ui| {
                                    let mut h_bg = theme
                                        .colors
                                        .button_hover_bg
                                        .unwrap_or(theme.colors.background); // Fallback
                                    if ui.color_edit_button_srgb(&mut h_bg).changed() {
                                        theme.colors.button_hover_bg = Some(h_bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_hover_bg.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.button_hover_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Button Active Background
                                ui.label("Button Active:");
                                ui.horizontal(|ui| {
                                    let mut a_bg = theme
                                        .colors
                                        .button_active_bg
                                        .unwrap_or(theme.colors.background);
                                    if ui.color_edit_button_srgb(&mut a_bg).changed() {
                                        theme.colors.button_active_bg = Some(a_bg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_active_bg.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.button_active_bg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Button Foreground
                                ui.label("Button Text:");
                                ui.horizontal(|ui| {
                                    let mut fg =
                                        theme.colors.button_fg.unwrap_or(theme.colors.foreground);
                                    if ui.color_edit_button_srgb(&mut fg).changed() {
                                        theme.colors.button_fg = Some(fg);
                                        theme_changed = true;
                                    }
                                    if theme.colors.button_fg.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.button_fg = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Separator
                                ui.label("Separator:");
                                ui.horizontal(|ui| {
                                    let mut sep = theme.colors.separator.unwrap_or([80, 80, 80]);
                                    if ui.color_edit_button_srgb(&mut sep).changed() {
                                        theme.colors.separator = Some(sep);
                                        theme_changed = true;
                                    }
                                    if theme.colors.separator.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.separator = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                // Line Number Color
                                ui.label("Line Numbers:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.line_number)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Comment Color
                                ui.label("Comments:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.comment)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Whitespace Symbols Color
                                ui.label("Whitespace Symbols:");
                                ui.horizontal(|ui| {
                                    let mut ws =
                                        theme.colors.whitespace_symbols.unwrap_or_else(|| {
                                            let c = theme.colors.comment;
                                            [
                                                (c[0] as f32 * 0.4) as u8,
                                                (c[1] as f32 * 0.4) as u8,
                                                (c[2] as f32 * 0.4) as u8,
                                            ]
                                        });
                                    if ui.color_edit_button_srgb(&mut ws).changed() {
                                        theme.colors.whitespace_symbols = Some(ws);
                                        theme_changed = true;
                                    }
                                    if theme.colors.whitespace_symbols.is_some() {
                                        if ui
                                            .button("↺")
                                            .on_hover_text("Reset to Default")
                                            .clicked()
                                        {
                                            theme.colors.whitespace_symbols = None;
                                            theme_changed = true;
                                        }
                                    }
                                });
                                ui.end_row();
                                ui.label("");
                                ui.label("");
                                ui.end_row();
                                // Status colors section
                                ui.label("Info Color:");
                                if ui.color_edit_button_srgb(&mut theme.colors.info).changed() {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.label("Success Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.success)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.label("Warning Color:");
                                if ui
                                    .color_edit_button_srgb(&mut theme.colors.warning)
                                    .changed()
                                {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.label("Error Color:");
                                if ui.color_edit_button_srgb(&mut theme.colors.error).changed() {
                                    theme_changed = true;
                                }
                                ui.end_row();
                            });
                    });
                // KLUCZOWA ZMIANA: Synchronizuj z current_theme natychmiast!
                if theme_changed {
                    theme.apply(ui.ctx());
                    self.current_theme = theme.clone();
                }
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui.button("💾 Save Theme").clicked() {
                        theme_to_save = Some(theme.clone());
                    }
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    let reset_text = if is_builtin {
                        format!("↺ Reset to Default ({:?})", theme.color_scheme)
                    } else {
                        "↺ Reset to Saved".to_string()
                    };
                    if ui.button(reset_text).clicked() {
                        should_reset = true;
                    }
                });
            } else {
                ui.label("No theme being edited");
            }
        });
        // Execute actions
        if should_reset {
            if let Some(ref mut theme) = self.editing_theme {
                let is_builtin = theme.name == "Dark" || theme.name == "Light";
                if is_builtin {
                    // Reset to factory defaults
                    *theme = match theme.color_scheme {
                        crate::theme::ColorScheme::Light => crate::theme::Theme::light(),
                        crate::theme::ColorScheme::Dark => crate::theme::Theme::dark(),
                    };
                } else {
                    // Reset to saved file
                    let all_themes = crate::theme::load_themes();
                    if let Some(saved) = all_themes.iter().find(|t| t.name == theme.name) {
                        *theme = saved.clone();
                    } else {
                        // If not found (e.g. unsaved new theme), maybe reset to parent scheme?
                        // Or just keep as is? Let's reset to scheme default as fallback.
                        *theme = match theme.color_scheme {
                            crate::theme::ColorScheme::Light => crate::theme::Theme::light(),
                            crate::theme::ColorScheme::Dark => crate::theme::Theme::dark(),
                        };
                        theme.name = theme.name.clone(); // Keep the name if it was a new custom theme
                                                         // Actually if it's new and unsaved, "Reset to Saved" is ambiguous.
                                                         // But usually this handles the "I messed up editing 'Ocean', let me revert to what's on disk" case.
                    }
                }
                theme.apply(ui.ctx());
                self.current_theme = theme.clone();
            }
        }
        if let Some(theme) = theme_to_save {
            match crate::theme::save_theme(&theme) {
                Ok(_) => {
                    self.current_theme = theme.clone();
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.themes = crate::theme::load_themes();
                    self.status_message = format!("OK: Theme saved: {}", theme.name);
                    self.log_info(format!("Theme saved successfully: {}", theme.name));
                }
                Err(e) => {
                    self.status_message = format!("Error saving theme: {}", e);
                    self.log_error(format!("Failed to save theme: {}", e));
                }
            }
        }
    }
}
