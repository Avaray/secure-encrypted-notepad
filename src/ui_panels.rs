use crate::app_state::{KeyStatus, LogLevel};
use crate::history::HistoryEntry;
use crate::EditorApp;
use eframe::egui;
use std::path::{Path, PathBuf};
impl EditorApp {
    /// Render settings panel
    pub(crate) fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);
        ui.vertical(|ui| {
            let h = ls.get_height("settings_header");
            if self.render_panel_header(ui, &rust_i18n::t!("settings.settings"), None, true, h) {
                self.show_settings_panel = false;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: 8,
                            right: 20,
                            top: 0,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
crate::app_helpers::center_row(ui, |ui| {
if ui.button(rust_i18n::t!("settings.open_folder")).on_hover_text(rust_i18n::t!("settings.open_folder_tooltip")).clicked() {
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
if ui.button(rust_i18n::t!("settings.reset_settings")).on_hover_text(rust_i18n::t!("settings.reset_settings_tooltip")).clicked() {
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
self.render_heading(ui, rust_i18n::t!("settings.security"));
crate::app_helpers::stateful_center_row(ui, ls.get_height("sec_pick_btns"), |ui| {
if ui.button(rust_i18n::t!("settings.set_global_keyfile")).clicked() {
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
            if ui.button(rust_i18n::t!("settings.use_now")).on_hover_text(rust_i18n::t!("settings.use_now_tooltip")).clicked() {
                self.keyfile_path = Some(path);
                self.refresh_file_tree();
                self.log_info("Global keyfile applied");
            }
        }
        if self.settings.global_keyfile_path.is_some() {
            if !self.show_clear_keyfile_confirmation {
                if ui.button(rust_i18n::t!("settings.clear")).clicked() {
                    self.show_clear_keyfile_confirmation = true;
                }
            } else {
                ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.are_you_sure")).color(self.current_theme.colors.error_color())).selectable(false));
                if ui.button(rust_i18n::t!("settings.yes")).clicked() {
                    self.settings.global_keyfile_path = None;
                    self.settings.keyfile_path_encrypted = None;
                    let _ = self.settings.save();
                    self.refresh_file_tree();
                    self.show_clear_keyfile_confirmation = false;
                    self.log_info("Global keyfile cleared");
                }
                if ui.button(rust_i18n::t!("settings.no")).clicked() {
                    self.show_clear_keyfile_confirmation = false;
                }
            }
        }
});
crate::app_helpers::stateful_center_row(ui, ls.get_height("sec_cur_path"), |ui| {
                ui.add(egui::Label::new(rust_i18n::t!("settings.current")).selectable(false));
if let Some(path) = &self.settings.global_keyfile_path {
if self.settings.show_keyfile_paths {
ui.add(egui::Label::new(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color())).selectable(false));
} else {
ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.secured")).color(self.current_theme.colors.success_color())).selectable(false));
}
} else {
    ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.none")).color(self.current_theme.colors.info_color())).selectable(false));
}
});
if ui
.checkbox(
&mut self.settings.use_global_keyfile,
rust_i18n::t!("settings.use_global_keyfile"),
)
.changed()
{
let _ = self.settings.save();
}
if ui.checkbox(&mut self.settings.show_keyfile_paths, rust_i18n::t!("settings.show_keyfile_paths"))
.on_hover_text(rust_i18n::t!("settings.show_keyfile_paths_tooltip"))
.changed() {
let _ = self.settings.save();
}
if ui.checkbox(&mut self.settings.show_directory_paths, rust_i18n::t!("settings.show_directory_paths"))
.on_hover_text(rust_i18n::t!("settings.show_directory_paths_tooltip"))
.changed() {
let _ = self.settings.save();
}

ui.add_space(8.0);
ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.auto_backup")).strong()).selectable(false));
if ui.checkbox(&mut self.settings.auto_backup_enabled, rust_i18n::t!("settings.auto_backup_enable"))
    .on_hover_text(rust_i18n::t!("settings.auto_backup_enable_tooltip"))
    .changed() {
    let _ = self.settings.save();
}
crate::app_helpers::center_row(ui, |ui| {
    if ui.button(rust_i18n::t!("settings.set_backup_dir")).clicked() {
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            self.settings.auto_backup_dir = Some(dir.clone());
            let _ = self.settings.save();
            self.log_info("Auto-backup directory set");
        }
    }

    if self.settings.auto_backup_dir.is_some() {
        if !self.show_clear_backup_dir_confirmation {
            if ui.button(rust_i18n::t!("settings.clear")).clicked() {
                self.show_clear_backup_dir_confirmation = true;
            }
        } else {
            ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.are_you_sure")).color(self.current_theme.colors.error_color())).selectable(false));
            if ui.button(rust_i18n::t!("settings.yes")).clicked() {
                self.settings.auto_backup_dir = None;
                self.settings.auto_backup_dir_encrypted = None;
                let _ = self.settings.save();
                self.show_clear_backup_dir_confirmation = false;
                self.log_info("Auto-backup directory cleared");
            }
            if ui.button(rust_i18n::t!("settings.no")).clicked() {
                self.show_clear_backup_dir_confirmation = false;
            }
        }
    }
});
crate::app_helpers::stateful_center_row(ui, ls.get_height("bkp_cur_path"), |ui| {
    ui.add(egui::Label::new(rust_i18n::t!("settings.current")).selectable(false));
    if let Some(path) = &self.settings.auto_backup_dir {
        if self.settings.show_directory_paths {
            ui.add(egui::Label::new(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color())).selectable(false));
        } else {
            ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.secured")).color(self.current_theme.colors.success_color())).selectable(false));
        }
    } else {
        ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.none")).color(self.current_theme.colors.info_color())).selectable(false));
    }
});

#[cfg(target_os = "windows")]
{
    ui.add_space(8.0);
    if ui.checkbox(&mut self.settings.screen_capture_protection, rust_i18n::t!("settings.screen_capture"))
        .on_hover_text(rust_i18n::t!("settings.screen_capture_tooltip"))
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
self.render_heading(ui, rust_i18n::t!("settings.workspace"));
// Starting directory setting
ui.add_space(4.0);
                    let h = ls.get_height("ws_start_dir");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.starting_dir"), h, |ui| {
if let Some(ref dir) = self.settings.file_tree_starting_dir {
if self.settings.show_directory_paths {
ui.add(egui::Label::new(
egui::RichText::new(dir.display().to_string())
.color(self.current_theme.colors.warning_color())
).selectable(false));
} else {
ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.secured")).color(self.current_theme.colors.success_color())).selectable(false));
}
} else {
ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.not_set")).color(self.current_theme.colors.info_color())).selectable(false));
}
});
crate::app_helpers::stateful_center_row(ui, ls.get_height("ws_pick_btns"), |ui| {
if ui.button(rust_i18n::t!("settings.set_starting_dir")).clicked() {
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
                    if ui.button(rust_i18n::t!("settings.clear")).clicked() {
                        self.show_clear_workspace_confirmation = true;
                    }
                } else {
                    ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.are_you_sure")).color(self.current_theme.colors.error_color())).selectable(false));
                    if ui.button(rust_i18n::t!("settings.yes")).clicked() {
                        self.settings.file_tree_starting_dir = None;
                        self.settings.file_tree_dir_encrypted = None;
                        let _ = self.settings.save();
                        self.log_info("Starting directory cleared");
                        self.show_clear_workspace_confirmation = false;
                    }
                    if ui.button(rust_i18n::t!("settings.no")).clicked() {
                        self.show_clear_workspace_confirmation = false;
                    }
                }
            }
});
if ui
.checkbox(&mut self.settings.show_subfolders, rust_i18n::t!("settings.show_subfolders"))
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}
if ui
.checkbox(&mut self.settings.hide_sen_extension, rust_i18n::t!("settings.hide_sen_ext"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.hide_undecryptable_files, rust_i18n::t!("settings.hide_undecryptable"))
.on_hover_text(rust_i18n::t!("settings.hide_undecryptable_tooltip"))
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}

if ui
.checkbox(&mut self.settings.hide_filename_in_title, rust_i18n::t!("settings.hide_filename_title"))
.on_hover_text(rust_i18n::t!("settings.hide_filename_title_tooltip"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.capitalize_tree_names, rust_i18n::t!("settings.capitalize_names"))
.on_hover_text(rust_i18n::t!("settings.capitalize_names_tooltip"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.hide_hidden_files, rust_i18n::t!("settings.hide_hidden"))
.on_hover_text(rust_i18n::t!("settings.hide_hidden_tooltip"))
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}
if ui
.checkbox(&mut self.settings.tree_style_file_tree, rust_i18n::t!("settings.tree_view"))
.on_hover_text(rust_i18n::t!("settings.tree_view_tooltip"))
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
self.setup_watcher();
}
ui.add_space(8.0);
ui.separator();
ui.add_space(8.0);
// =========================================================================
// 3. EDITOR
// =========================================================================
self.render_heading(ui, rust_i18n::t!("settings.editor"));
if ui
.checkbox(&mut self.settings.show_line_numbers, rust_i18n::t!("settings.show_line_numbers"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.show_whitespace, rust_i18n::t!("settings.show_whitespace"))
.on_hover_text(rust_i18n::t!("settings.show_whitespace_tooltip"))
.changed()
{
let _ = self.settings.save();
}
// Cursor settings
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.cursor_shape")).selectable(false));
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
if ui.checkbox(&mut self.settings.cursor_blink, rust_i18n::t!("settings.cursor_blink")).changed() {
let _ = self.settings.save();
self.style_dirty = true;
}
if ui.checkbox(&mut self.settings.word_wrap, rust_i18n::t!("settings.word_wrap")).changed() {
let _ = self.settings.save();
}
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.tab_size")).selectable(false));
                if ui
                    .add(
                        egui::DragValue::new(&mut self.settings.tab_size)
                            .range(2..=8)
                            .clamp_existing_to_range(true),
                    )
.changed()
{
let _ = self.settings.save();
}
});
if ui
.checkbox(&mut self.settings.use_spaces_for_tabs, rust_i18n::t!("settings.spaces_for_tabs"))
.changed()
{
let _ = self.settings.save();
}

crate::app_helpers::center_row(ui, |ui| {
    ui.add(egui::Label::new(rust_i18n::t!("settings.comment_prefix")).selectable(false));
    let mut changed = false;
    let response = ui.add(
        egui::TextEdit::singleline(&mut self.settings.comment_prefix)
            .desired_width(50.0)
            .margin(ui.spacing().button_padding)
    );

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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.max_lines")).selectable(false));
let mut limit_val = self.settings.max_lines;
if ui
.add(
                        egui::DragValue::new(&mut limit_val)
                            .speed(10.0)
                            .range(0..=1000000)
                            .clamp_existing_to_range(true),
)
.changed()
{
self.settings.max_lines = limit_val;
let _ = self.settings.save();
}
if self.settings.max_lines == 0 {
ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.no_limit")).italics().weak()).selectable(false));
}
})
.response
.on_hover_text(rust_i18n::t!("settings.max_lines_tooltip"));
// History capacity
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.history_limit")).selectable(false));
if ui
.add(
                        egui::DragValue::new(&mut self.settings.max_history_length)
                            .speed(0.5)
                            .range(1..=1000)
                            .clamp_existing_to_range(true),
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
self.render_heading(ui, rust_i18n::t!("settings.reliability"));
ui.vertical(|ui| {
ui.set_min_width(ui.available_width());
ui.group(|ui| {
ui.set_min_width(ui.available_width());
ui.add(egui::Label::new(rust_i18n::t!("settings.auto_save")).selectable(false));
if ui
.checkbox(&mut self.settings.auto_save_on_focus_loss, rust_i18n::t!("settings.auto_save_focus"))
.on_hover_text(rust_i18n::t!("settings.auto_save_focus_tooltip"))
.changed()
{
let _ = self.settings.save();
}
                if ui
                    .checkbox(&mut self.settings.auto_save_enabled, rust_i18n::t!("settings.auto_save_debounce"))
                    .on_hover_text(rust_i18n::t!("settings.auto_save_debounce_tooltip"))
                    .changed()
{
let _ = self.settings.save();
}
                crate::app_helpers::center_row(ui, |ui| {
                    ui.add(egui::Label::new(rust_i18n::t!("settings.inactivity_secs")).selectable(false));
                    if ui
                        .add(
                            egui::DragValue::new(&mut self.settings.auto_save_debounce_secs)
                                .speed(1.0)
                                .range(1..=3600)
                                .clamp_existing_to_range(true),
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
self.render_heading(ui, rust_i18n::t!("settings.appearance"));
// Theme selection
                    let h = ls.get_height("set_theme");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.theme"), h, |ui| {
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
if ui.button(rust_i18n::t!("settings.refresh")).clicked() {
self.themes = crate::theme::load_themes();
self.log_info("Themes refreshed");
}
});
ui.separator();
// UI font family with keyboard navigation
                    let h = ls.get_height("set_ui_font");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.ui_font"), h, |ui| {
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
self.fonts_dirty = true;
self.log_info(format!(
"UI font changed to: {}",
self.settings.ui_font_family
));
}
});
});
                    let h = ls.get_height("set_ui_font_sz");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.ui_font_size"), h, |ui| {
let response = ui.add(
egui::DragValue::new(&mut self.settings.ui_font_size)
.speed(0.5)
.range(8.0..=128.0),
);
if response.changed() {
self.settings.validate_font_sizes();
self.style_dirty = true;
}
if response.drag_stopped() || (response.changed() && response.lost_focus()) {
let _ = self.settings.save();
}
});
ui.separator();
                    let h = ls.get_height("set_ed_font");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.editor_font"), h, |ui| {
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
self.fonts_dirty = true;
self.log_info(format!(
"Editor font changed to: {}",
self.settings.editor_font_family
));
}
});
});
                    let h = ls.get_height("set_ed_font_sz");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.editor_font_size"), h, |ui| {
let response = ui.add(
egui::DragValue::new(&mut self.settings.editor_font_size)
.speed(0.5)
.range(8.0..=128.0),
);
if response.changed() {
self.settings.validate_font_sizes();
self.style_dirty = true;
}
if response.drag_stopped() || (response.changed() && response.lost_focus()) {
let _ = self.settings.save();
}
});
                    let h = ls.get_height("set_line_h");
                    crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("settings.line_height"), h, |ui| {
            let response = ui.add(
                egui::DragValue::new(&mut self.settings.line_height)
                    .speed(0.05)
                    .range(1.0..=2.5)
                    .max_decimals(2)
            );

            if response.drag_stopped() || (response.changed() && response.lost_focus()) {
                let _ = self.settings.save();
            }
});
// Global Scroll Speed
crate::app_helpers::center_row(ui, |ui| {
    ui.add(egui::Label::new(rust_i18n::t!("settings.scroll_speed")).selectable(false));
    let mut mult = self.settings.scroll_speed_multiplier;
    let response = ui.add(
        egui::DragValue::new(&mut mult)
            .speed(0.1)
            .range(1.0..=10.0)
            .clamp_existing_to_range(true),
    )
    .on_hover_text(rust_i18n::t!("settings.scroll_speed_tooltip"));

    if response.changed() {
        self.settings.scroll_speed_multiplier = mult;
    }
    if response.drag_stopped() || (response.changed() && response.lost_focus()) {
        let _ = self.settings.save();
    }
});
// Toolbar icon size
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.toolbar_icon_size")).selectable(false));
let response = ui.add(
egui::DragValue::new(&mut self.settings.toolbar_icon_size)
.speed(1.0)
.range(12.0..=96.0)
.clamp_existing_to_range(true),
);

if response.drag_stopped() || (response.changed() && response.lost_focus()) {
let _ = self.settings.save();
}
});
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.toolbar_position")).selectable(false));
let mut changed = false;
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Top, rust_i18n::t!("settings.toolbar_top")).changed();
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Left, rust_i18n::t!("settings.toolbar_left")).changed();
changed |= ui.radio_value(&mut self.settings.toolbar_position, crate::settings::ToolbarPosition::Right, rust_i18n::t!("settings.toolbar_right")).changed();
if changed {
let _ = self.settings.save();
}
});
// Language Selector
            let current_lang = self.settings.language.clone();
            let mut changed = false;
            let (current_label, current_icon) = match current_lang.as_str() {
                "pl" => ("Polski", &self.icons.flag_pl),
                "de" => ("Deutsch", &self.icons.flag_de),
                _ => ("English", &self.icons.flag_en),
            };

            let text_height = ui.text_style_height(&egui::TextStyle::Body);

            crate::app_helpers::center_row(ui, |ui| {
                ui.add(
                    egui::Button::new(rust_i18n::t!("settings.language"))
                        .frame(false) // Disables background and border - looks like plain text!
                        .sense(egui::Sense::hover()) // Makes it non-clickable
                );

                // We use text_height for the flag
                ui.add(egui::Image::new(current_icon).max_height(text_height).maintain_aspect_ratio(true));

                egui::ComboBox::from_id_salt("language_selector")
                    .selected_text(current_label)
                    .show_ui(ui, |ui| {
                        let mut lang_row = |ui: &mut egui::Ui, code: &str, label: &str, icon: &egui::TextureHandle| {
                            let is_selected = self.settings.language == code;
                            let mut clicked = false;
                            crate::app_helpers::flex_row(ui, |ui| {
                                // We also use text_height here
                                ui.add(egui::Image::new(icon).max_height(text_height).maintain_aspect_ratio(true));
                                if ui.selectable_label(is_selected, label).clicked() {
                                    self.settings.language = code.to_string();
                                    clicked = true;
                                }
                            });
                            clicked
                        };

                        if lang_row(ui, "en", "English", &self.icons.flag_en) { changed = true; }
                        if lang_row(ui, "pl", "Polski", &self.icons.flag_pl) { changed = true; }
                        if lang_row(ui, "de", "Deutsch", &self.icons.flag_de) { changed = true; }
                    });
            });
if changed {
rust_i18n::set_locale(&self.settings.language);
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.preserve_all_panels, rust_i18n::t!("settings.preserve_panels"))
.on_hover_text(rust_i18n::t!("settings.preserve_panels_tooltip"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.start_maximized, rust_i18n::t!("settings.start_maximized"))
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.remember_zen_mode, rust_i18n::t!("settings.remember_zen"))
.on_hover_text(rust_i18n::t!("settings.remember_zen_tooltip"))
.changed()
{
                            let _ = self.settings.save();
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        // =========================================================================
                        // 5. SYSTEM
                        // =========================================================================
                        self.render_heading(ui, rust_i18n::t!("settings.window_panels"));

                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        {
                            ui.add_space(4.0);
                            if ui.button(rust_i18n::t!("settings.associate_sen"))
                                .on_hover_text(rust_i18n::t!("settings.associate_sen_tooltip"))
                                .clicked() {
                                self.associate_sen_files();
                            }
                            ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.assoc_warning")).small().weak()).selectable(false));
                        }

                        #[cfg(target_os = "macos")]
                        {
                            ui.add_space(4.0);
                            ui.add(egui::Label::new(rust_i18n::t!("settings.assoc_macos")).weak());
                        }

                        ui.add_space(8.0);

                        if ui
                            .checkbox(&mut self.settings.single_instance, rust_i18n::t!("settings.single_instance"))
                            .on_hover_text(rust_i18n::t!("settings.single_instance_tooltip"))
                            .changed()
                        {
                            let _ = self.settings.save();
                        }

                            ui.add_space(4.0);
                        });
                });
        });
        self.layout_state = ls;
    }
    /// Render history panel
    pub(crate) fn render_history_panel(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);
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
                let h = ls.get_height("history_header");
                if self.render_panel_header(ui, &rust_i18n::t!("history.title"), None, true, h) {
                    self.show_history_panel = false;
                }
            crate::app_helpers::center_row(ui, |ui| {
                ui.add(egui::Label::new(rust_i18n::t!("history.max_limit")).selectable(false));
                let mut temp_limit = doc_max_limit;
                let response = ui.add(
                    egui::DragValue::new(&mut temp_limit)
                        .speed(0.5)
                        .range(1..=1000)
                        .clamp_existing_to_range(true),
                );

                if response.changed() {
                    self.document.set_max_history_length(temp_limit);
                    self.is_modified = true;
                }
            });
            let history_status_color = if history_len > doc_max_limit {
                self.current_theme.colors.warning_color()
            } else {
                ui.visuals().widgets.noninteractive.fg_stroke.color // Default weak color
            };

            ui.add(egui::Label::new(
                egui::RichText::new(rust_i18n::t!("history.current_entries", current = history_len, max = doc_max_limit))
                .color(history_status_color),
            ).selectable(false));

            if history_len > doc_max_limit {
                let to_delete = history_len - doc_max_limit;
                ui.add(egui::Label::new(
                    egui::RichText::new(rust_i18n::t!("history.will_be_deleted", count = to_delete))
                        .color(self.current_theme.colors.warning_color())
                        .small(),
                ).selectable(false));
            }

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

                crate::app_helpers::center_row(ui, |ui| {
                    if self.show_clear_history_confirmation {
                        ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("settings.are_you_sure")).color(self.current_theme.colors.error_color())).selectable(false));
                        if ui.button(rust_i18n::t!("settings.yes")).clicked() {
                            self.clear_all_history();
                            self.loaded_history_index = None;
                            self.show_clear_history_confirmation = false;
                        }
                        if ui.button(rust_i18n::t!("settings.no")).clicked() {
                            self.show_clear_history_confirmation = false;
                        }
                    } else {
                        ui.horizontal(|ui| {
                            if ui.button(rust_i18n::t!("history.clear_all")).clicked() {
                                self.show_clear_history_confirmation = true;
                            }

                            let history_changed = self.document.history.iter().any(|e| e.deleted)
                                || self.document.history.len() != self.initial_history_len
                                || self.document.max_history_length != self.initial_max_history_length;

                            if history_changed {
                                if ui.button(rust_i18n::t!("history.revert_changes"))
                                    .on_hover_text(rust_i18n::t!("history.log_reverted"))
                                    .clicked()
                                {
                                    self.revert_history_changes();
                                }
                            }
                        });
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
                        .auto_shrink([false, false])
                                                .show(ui, |ui| {
                            egui::Frame::NONE
                                .inner_margin(egui::Margin {
                                    left: 4,
                                    right: 16,
                                    top: 0,
                                    bottom: 0,
                                })
                                .show(ui, |ui| {
                                    if history_len == 0 {
                                ui.add(egui::Label::new(rust_i18n::t!("history.no_history")).selectable(false));
                            } else {
                                let to_delete_count = if history_len > doc_max_limit {
                                    history_len - doc_max_limit
                                } else {
                                    0
                                };

                                for (v_idx, (original_index, entry)) in visible_history.iter().enumerate().rev() {
                                    let is_loaded = self.loaded_history_index == Some(*original_index);
                                    let will_be_deleted = v_idx < to_delete_count;

                                    crate::app_helpers::center_row(ui, |ui| {
                                        ui.spacing_mut().item_spacing.x = 4.0;

                                        // 1. Arrow column — fixed width, painted manually so it
                                        //    never affects the layout width regardless of visibility.
                                        let row_height = ui.text_style_height(&egui::TextStyle::Body);
                                        let arrow_size = egui::vec2(16.0, row_height);
                                        let (arrow_rect, _) = ui.allocate_exact_size(arrow_size, egui::Sense::hover());
                                        if is_loaded {
                                            ui.painter().text(
                                                arrow_rect.center(),
                                                egui::Align2::CENTER_CENTER,
                                                ">",
                                                egui::FontId::proportional(row_height),
                                                self.current_theme.colors.success_color(),
                                            );
                                        }

                                        let text = entry.display_timestamp().to_string();
                                        let mut rich_text = egui::RichText::new(text);
                                        if will_be_deleted {
                                            rich_text = rich_text.color(self.current_theme.colors.warning_color());
                                        }

                                        // right_to_left places the buttons on the far right first, then the nested
                                        // left_to_right layout fills all remaining space with the label (text stays left-aligned).
                                        let (label_res, delete_clicked, revert_clicked) = ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                let del = ui.button("X").on_hover_text(rust_i18n::t!("history.delete_entry")).clicked();
                                                let rev = ui.button("R").on_hover_text(rust_i18n::t!("history.revert_entry")).clicked();

                                                let lbl = ui.with_layout(
                                                    egui::Layout::left_to_right(egui::Align::Center),
                                                    |ui| {
                                                        // Prevent text from wrapping to a new line and pushing the layout down
                                                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                                                        ui.selectable_label(is_loaded, rich_text)
                                                    },
                                                ).inner;

                                                (lbl, del, rev)
                                            },
                                        ).inner;

                                        if label_res.clicked() {
                                            self.load_history_version(*original_index);
                                            self.loaded_history_index = Some(*original_index);
                                            ui.memory_mut(|mem| mem.request_focus(history_area_id));
                                        }

                                        if is_loaded && has_focus {
                                            label_res.scroll_to_me(None);
                                        }

                                        if delete_clicked {
                                            self.delete_history_entry(*original_index);
                                            if self.loaded_history_index == Some(*original_index) {
                                                self.loaded_history_index = None;
                                            }
                                        }

                                        if revert_clicked {
                                            self.revert_to_history_version(*original_index);
                                            ui.memory_mut(|mem| mem.request_focus(history_area_id));
                                        }
                                    });
                                    ui.add_space(2.0);
                                }
                            }
                        });
                });
                });
        });
        self.layout_state = ls;
    }
    /// Render debug panel
    pub(crate) fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);
        ui.vertical(|ui| {
            let h = ls.get_height("debug_header");
            if self.render_panel_header(ui, &rust_i18n::t!("debug.title"), None, true, h) {
                self.show_debug_panel = false;
            }
            egui::Frame::NONE
                .inner_margin(egui::Margin {
                    left: 8,
                    right: 12,
                    top: 0,
                    bottom: 0,
                })
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;

                        // Clear button on the left
                        if ui.button(rust_i18n::t!("debug.clear")).clicked() {
                            self.debug_log.clear();
                        }

                        // Toggles on the right
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center).with_main_wrap(true),
                            |ui| {
                                let mut changed = false;

                                // 4. Error
                                let mut job = egui::text::LayoutJob::default();
                                let error_uline = if self.settings.debug_show_error {
                                    self.current_theme.colors.error_color()
                                } else {
                                    ui.visuals().widgets.noninteractive.bg_stroke.color
                                };
                                job.append(
                                    &rust_i18n::t!("debug.filter_error"),
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: egui::TextStyle::Button.resolve(ui.style()),
                                        color: ui.visuals().text_color(),
                                        underline: egui::Stroke::new(2.0, error_uline),
                                        ..Default::default()
                                    },
                                );
                                changed |= ui
                                    .checkbox(&mut self.settings.debug_show_error, job)
                                    .changed();

                                // 3. Warning
                                let mut job = egui::text::LayoutJob::default();
                                let warning_uline = if self.settings.debug_show_warning {
                                    self.current_theme.colors.warning_color()
                                } else {
                                    ui.visuals().widgets.noninteractive.bg_stroke.color
                                };
                                job.append(
                                    &rust_i18n::t!("debug.filter_warning"),
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: egui::TextStyle::Button.resolve(ui.style()),
                                        color: ui.visuals().text_color(),
                                        underline: egui::Stroke::new(2.0, warning_uline),
                                        ..Default::default()
                                    },
                                );
                                changed |= ui
                                    .checkbox(&mut self.settings.debug_show_warning, job)
                                    .changed();

                                // 2. Success
                                let mut job = egui::text::LayoutJob::default();
                                let success_uline = if self.settings.debug_show_success {
                                    self.current_theme.colors.success_color()
                                } else {
                                    ui.visuals().widgets.noninteractive.bg_stroke.color
                                };
                                job.append(
                                    &rust_i18n::t!("debug.filter_success"),
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: egui::TextStyle::Button.resolve(ui.style()),
                                        color: ui.visuals().text_color(),
                                        underline: egui::Stroke::new(2.0, success_uline),
                                        ..Default::default()
                                    },
                                );
                                changed |= ui
                                    .checkbox(&mut self.settings.debug_show_success, job)
                                    .changed();

                                // 1. Info
                                let mut job = egui::text::LayoutJob::default();
                                let info_uline = if self.settings.debug_show_info {
                                    self.current_theme.colors.info_color()
                                } else {
                                    ui.visuals().widgets.noninteractive.bg_stroke.color
                                };
                                job.append(
                                    &rust_i18n::t!("debug.filter_info"),
                                    0.0,
                                    egui::text::TextFormat {
                                        font_id: egui::TextStyle::Button.resolve(ui.style()),
                                        color: ui.visuals().text_color(),
                                        underline: egui::Stroke::new(2.0, info_uline),
                                        ..Default::default()
                                    },
                                );
                                changed |= ui
                                    .checkbox(&mut self.settings.debug_show_info, job)
                                    .changed();

                                if changed {
                                    let _ = self.settings.save();
                                }
                            },
                        );
                    });
                });
            ui.separator();
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: 4,
                            right: 16,
                            top: 0,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            let mut visible_count = 0;
                            for entry in &self.debug_log {
                                let show = match entry.level {
                                    LogLevel::Info => self.settings.debug_show_info,
                                    LogLevel::Success => self.settings.debug_show_success,
                                    LogLevel::Warning => self.settings.debug_show_warning,
                                    LogLevel::Error => self.settings.debug_show_error,
                                };

                                if show {
                                    visible_count += 1;
                                    let color = match entry.level {
                                        LogLevel::Info => self.current_theme.colors.info_color(),
                                        LogLevel::Success => {
                                            self.current_theme.colors.success_color()
                                        }
                                        LogLevel::Warning => {
                                            self.current_theme.colors.warning_color()
                                        }
                                        LogLevel::Error => self.current_theme.colors.error_color(),
                                    };
                                    ui.colored_label(color, entry.display());
                                }
                            }
                            if visible_count == 0 {
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(rust_i18n::t!("debug.all_filtered"))
                                            .italics()
                                            .weak(),
                                    )
                                    .selectable(false),
                                );
                            }
                        });
                });
        });
        self.layout_state = ls;
    }
    /// Render file tree panel
    pub(crate) fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);
        let mut _scroll_to_path: Option<PathBuf> = None;
        // ★ FIX: allow panel to shrink below its natural content size
        ui.set_min_width(0.0);
        ui.vertical(|ui| {
            // Aggressively prevent expansion
            ui.set_max_width(ui.available_width());
            ui.set_min_width(0.0);
            let h = ls.get_height("tree_header");
            if self.render_panel_header(ui, &rust_i18n::t!("file_tree.title"), None, true, h) {
                self.show_file_tree = false;
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // ★ FIX: prevent ScrollArea from propagating content min_width to the panel
                    ui.set_min_width(0.0);
                    ui.set_max_width(ui.available_width());
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: 4,
                            right: 12,
                            top: 0,
                            bottom: 0,
                        })
                        .show(ui, |ui| {
                            // ★ FIX: Frame inside ScrollArea must not impose min_width
                            ui.set_min_width(0.0);
                            ui.set_max_width(ui.available_width());
                            if let Some(dir) = &self.file_tree_dir {
                                if self.settings.show_directory_paths {
                                    let sub_font = egui::TextStyle::Body.resolve(ui.style());
                                    let truncated_dir = self.smart_truncate_text(
                                        ui,
                                        &dir.display().to_string(),
                                        sub_font,
                                        ui.available_width() - 12.0,
                                    );
                                    ui.label(truncated_dir);
                                    ui.separator();
                                }
                                let available_width = ui.available_width();
                                ui.set_max_width(available_width);
                                let entries = self.file_tree_entries.clone();
                                let tree_on = self.settings.tree_style_file_tree;
                                let tree_indent: f32 = if tree_on { 24.0 } else { 0.0 };
                                let panel_left = ui.cursor().left();
                                let line_color =
                                    self.current_theme.colors.tree_line_color(ui.visuals());
                                let stroke = egui::Stroke::new(1.0, line_color);

                                // Collect mid_y positions for tree lines (two-pass to avoid overlap)
                                struct RowData {
                                    depth: usize,
                                    is_dir: bool,

                                    bottom_y: f32,
                                    mid_y: f32,
                                }

                                let mut row_infos = Vec::new();
                                let mut dir_stack: Vec<(PathBuf, usize)> = Vec::new();

                                for (_i, entry) in entries.iter().enumerate() {
                                    // Detect end of directories before processing the next entry
                                    while let Some((dir_path, _depth)) = dir_stack.last() {
                                        if !entry.path.starts_with(dir_path) {
                                            let (finished_dir, finished_depth) =
                                                dir_stack.pop().unwrap();
                                            self.render_scanning_spinner_if_needed(
                                                ui,
                                                &finished_dir,
                                                finished_depth,
                                                tree_on,
                                                tree_indent,
                                            );
                                        } else {
                                            break;
                                        }
                                    }

                                    if entry.is_dir {
                                        dir_stack.push((entry.path.clone(), entry.depth));
                                    }

                                    let path = &entry.path;
                                    let raw_filename =
                                        path.file_name().unwrap_or_default().to_string_lossy();
                                    let filename = if self.settings.capitalize_tree_names {
                                        raw_filename.to_uppercase()
                                    } else {
                                        raw_filename.to_string()
                                    };

                                    // Hide undecryptable files check
                                    if !entry.is_dir
                                        && raw_filename.to_lowercase().ends_with(".sen")
                                    {
                                        let status = self
                                            .file_access_cache
                                            .get(path)
                                            .cloned()
                                            .unwrap_or(KeyStatus::Unknown);

                                        if self.settings.hide_undecryptable_files
                                            && status != KeyStatus::Decryptable
                                        {
                                            continue;
                                        }
                                    }

                                    let top_y = ui.cursor().top();
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
                                                "..".to_string()
                                            } else {
                                                filename.to_string()
                                            };

                                            let btn_font =
                                                egui::TextStyle::Button.resolve(ui.style());

                                            let button_resp = {
                                                let icon = if is_parent {
                                                    &self.icons.folder_filled
                                                } else if tree_on && entry.is_expanded {
                                                    &self.icons.folder_open
                                                } else {
                                                    &self.icons.folder_filled
                                                };

                                                let tint = self.current_theme.colors.icon_color();
                                                let icon_size =
                                                    ui.text_style_height(&egui::TextStyle::Button);

                                                let truncated_name = self.smart_truncate_text(
                                                    ui,
                                                    &display_name,
                                                    btn_font,
                                                    (ui.available_width()
                                                        - icon_size
                                                        - ui.spacing().button_padding.x * 2.0
                                                        - ui.spacing().item_spacing.x
                                                        - 8.0)
                                                        .max(20.0),
                                                );

                                                ui.add(egui::Button::image_and_text(
                                                    egui::Image::new(icon)
                                                        .shrink_to_fit()
                                                        .maintain_aspect_ratio(true)
                                                        .fit_to_exact_size(egui::vec2(
                                                            icon_size, icon_size,
                                                        ))
                                                        .tint(tint),
                                                    truncated_name,
                                                ))
                                            };

                                            if button_resp.clicked() {
                                                if is_parent {
                                                    self.change_directory(path.clone());
                                                } else if tree_on {
                                                    // Toggle expansion
                                                    if entry.is_expanded {
                                                        self.expanded_directories.remove(path);
                                                    } else {
                                                        self.expanded_directories
                                                            .insert(path.clone());
                                                    }
                                                    self.refresh_file_tree(); // Re-trigger build
                                                } else {
                                                    self.change_directory(path.clone());
                                                }
                                            }
                                        } else {
                                            // File
                                            let display_name = if self.settings.hide_sen_extension
                                                && filename.to_lowercase().ends_with(".sen")
                                            {
                                                filename[..filename.len() - 4].to_string()
                                            } else {
                                                filename.to_string()
                                            };

                                            if raw_filename.to_lowercase().ends_with(".sen") {
                                                let status = self
                                                    .file_access_cache
                                                    .get(path)
                                                    .cloned()
                                                    .unwrap_or(KeyStatus::Unknown);

                                                let color = match status {
                                                    KeyStatus::Decryptable => {
                                                        self.current_theme.colors.success_color()
                                                    }
                                                    KeyStatus::WrongKey => {
                                                        self.current_theme.colors.error_color()
                                                    }
                                                    KeyStatus::Unknown => {
                                                        self.current_theme.colors.warning_color()
                                                    }
                                                    _ => ui.visuals().weak_text_color(),
                                                };

                                                let pulse_alpha = if self.keyfile_path.is_none() {
                                                    (0.1 + 0.9
                                                        * (self.start_time.elapsed().as_secs_f32()
                                                            * 3.0)
                                                            .cos()
                                                            .abs())
                                                        as f32
                                                } else {
                                                    1.0
                                                };

                                                let icon = match status {
                                                    KeyStatus::Unknown => &self.icons.unknown_file,
                                                    KeyStatus::WrongKey => &self.icons.locked_file,
                                                    KeyStatus::Decryptable => {
                                                        &self.icons.asterisk_file
                                                    }
                                                    _ => {
                                                        if tree_on {
                                                            &self.icons.status_dot
                                                        } else {
                                                            &self.icons.key
                                                        }
                                                    }
                                                };

                                                let font_id =
                                                    egui::TextStyle::Button.resolve(ui.style());
                                                let icon_size =
                                                    ui.text_style_height(&egui::TextStyle::Button);

                                                let truncated_name = self.smart_truncate_text(
                                                    ui,
                                                    &display_name,
                                                    font_id,
                                                    (ui.available_width()
                                                        - icon_size
                                                        - ui.spacing().button_padding.x * 2.0
                                                        - ui.spacing().item_spacing.x
                                                        - 8.0)
                                                        .max(10.0),
                                                );

                                                if ui
                                                    .add(egui::Button::image_and_text(
                                                        egui::Image::new(icon)
                                                            .fit_to_exact_size(egui::vec2(
                                                                icon_size, icon_size,
                                                            ))
                                                            .tint(
                                                                color.gamma_multiply(pulse_alpha),
                                                            ),
                                                        truncated_name,
                                                    ))
                                                    .clicked()
                                                {
                                                    self.open_file(path.clone());
                                                }
                                            } else {
                                                // Non-SEN file
                                                if !tree_on {
                                                    let icon_size = ui
                                                        .text_style_height(&egui::TextStyle::Body);
                                                    ui.allocate_ui(
                                                        egui::vec2(icon_size, icon_size),
                                                        |ui| {
                                                            ui.centered_and_justified(|ui| {
                                                                ui.label("");
                                                            });
                                                        },
                                                    );
                                                }
                                                let btn_font =
                                                    egui::TextStyle::Button.resolve(ui.style());
                                                let truncated_name = self.smart_truncate_text(
                                                    ui,
                                                    &display_name,
                                                    btn_font,
                                                    (ui.available_width()
                                                        - ui.spacing().button_padding.x * 2.0)
                                                        .max(20.0),
                                                );
                                                if ui
                                                    .add(egui::Button::new(truncated_name))
                                                    .clicked()
                                                {
                                                    self.open_file(path.clone());
                                                }
                                            }
                                        }
                                    });
                                    if tree_on {
                                        let bottom_y = ui.cursor().top();
                                        let actual_bottom_y =
                                            bottom_y - ui.spacing().item_spacing.y;
                                        row_infos.push(RowData {
                                            depth: entry.depth,
                                            is_dir: entry.is_dir,
                                            bottom_y: actual_bottom_y,
                                            mid_y: (top_y + actual_bottom_y) / 2.0,
                                        });
                                    }
                                }

                                // Close remaining directories after the loop
                                while let Some((dir_path, depth)) = dir_stack.pop() {
                                    self.render_scanning_spinner_if_needed(
                                        ui,
                                        &dir_path,
                                        depth,
                                        tree_on,
                                        tree_indent,
                                    );
                                }

                                // Pass 2: Draw the tree geometry based on layout positions
                                if tree_on && !row_infos.is_empty() {
                                    let painter = ui.painter();

                                    for i in 0..row_infos.len() {
                                        let row = &row_infos[i];

                                        // Draw horizontal branch
                                        if row.depth > 0 {
                                            let parent_depth = row.depth - 1;
                                            let parent_drop_x = panel_left
                                                + (parent_depth as f32) * tree_indent
                                                + 11.0;
                                            let branch_start_x = parent_drop_x + stroke.width / 2.0;
                                            let branch_end_x =
                                                panel_left + (row.depth as f32) * tree_indent - 1.0;

                                            painter.rect_filled(
                                                egui::Rect::from_min_max(
                                                    egui::pos2(
                                                        branch_start_x,
                                                        row.mid_y - stroke.width / 2.0,
                                                    ),
                                                    egui::pos2(
                                                        branch_end_x,
                                                        row.mid_y + stroke.width / 2.0,
                                                    ),
                                                ),
                                                0.0,
                                                stroke.color,
                                            );
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
                                                let drop_x = panel_left
                                                    + (row.depth as f32) * tree_indent
                                                    + 11.0;
                                                painter.rect_filled(
                                                    egui::Rect::from_min_max(
                                                        egui::pos2(
                                                            drop_x - stroke.width / 2.0,
                                                            row.bottom_y,
                                                        ),
                                                        egui::pos2(
                                                            drop_x + stroke.width / 2.0,
                                                            end_y + stroke.width / 2.0,
                                                        ),
                                                    ),
                                                    0.0,
                                                    stroke.color,
                                                );
                                            }
                                        }
                                    }
                                }
                            } else {
                                ui.label(rust_i18n::t!("settings.no_dir_opened"));
                                if ui.button(rust_i18n::t!("settings.open_dir")).clicked() {
                                    self.open_directory();
                                }
                            }
                        });
                });
        });
        self.layout_state = ls;
    }
    /// Render theme editor panel
    pub(crate) fn render_theme_editor_panel(&mut self, ui: &mut egui::Ui) {
        let mut ls = std::mem::take(&mut self.layout_state);
        let mut theme_to_save: Option<crate::theme::Theme> = None;
        let mut should_reset = false;
        ui.vertical(|ui| {
            let h = ls.get_height("theme_header");
            if self.render_panel_header(ui, &rust_i18n::t!("theme.title"), None, true, h) {
                self.show_theme_editor = false;
            }
            if let Some(theme) = &mut self.editing_theme {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(rust_i18n::t!("theme.save")).clicked() {
                        theme_to_save = Some(theme.clone());
                    }
                    let mut modified = true;
                    if let Some(original) = &self.original_editing_theme {
                        if theme == original {
                            modified = false;
                        }
                    }

                    if modified {
                        let is_builtin = theme.name == "Dark" || theme.name == "Light";
                        let reset_text = if is_builtin {
                            rust_i18n::t!("theme.reset_default").to_string()
                        } else {
                            rust_i18n::t!("theme.reset_saved").to_string()
                        };
                        if ui.button(reset_text).clicked() {
                            should_reset = true;
                        }
                    }
                });
            }

            ui.separator();

            // Top bar: Theme selector and actions
            crate::app_helpers::center_row(ui, |ui| {
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
                                self.original_editing_theme = Some(theme.clone());
                                self.current_theme = theme.clone();
                                self.settings.theme_name = theme.name.clone();
                                self.show_delete_theme_confirmation = false; // Reset confirmation on theme change
                                self.apply_theme(ui.ctx());
                                let _ = self.settings.save();
                            }
                        }
                    });
                if ui.button(rust_i18n::t!("theme.new")).clicked() {
                    let mut new_theme = self.current_theme.clone();
                    new_theme.name =
                        format!("{} {}", new_theme.name, rust_i18n::t!("theme.copy_suffix"));
                    self.editing_theme = Some(new_theme.clone());
                    self.original_editing_theme = Some(new_theme);
                    self.show_delete_theme_confirmation = false; // Reset confirmation
                }
                // Delete button with confirmation
                if let Some(theme) = &self.editing_theme {
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    if !is_builtin {
                        if !self.show_delete_theme_confirmation {
                            if ui.button(rust_i18n::t!("theme.delete")).clicked() {
                                self.show_delete_theme_confirmation = true;
                            }
                        } else {
                            ui.label(
                                egui::RichText::new(rust_i18n::t!("settings.are_you_sure"))
                                    .color(self.current_theme.colors.error_color()),
                            );
                            if ui.button(rust_i18n::t!("settings.yes")).clicked() {
                                let _ = crate::theme::delete_theme(&theme.name);
                                self.themes = crate::theme::load_themes(); // Reload
                                self.editing_theme = Some(crate::theme::Theme::dark());
                                self.show_delete_theme_confirmation = false;
                                // Reset to safe default
                            }
                            if ui.button(rust_i18n::t!("settings.no")).clicked() {
                                self.show_delete_theme_confirmation = false;
                            }
                        }
                    }
                }
            });
            ui.separator();
            if let Some(ref mut theme) = self.editing_theme {
                let mut theme_changed = false;
                crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("theme.name_label"), &mut 0.0, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut theme.name)
                            .desired_width(ui.available_width() - 8.0)
                            .margin(ui.spacing().button_padding),
                    );
                });
                crate::app_helpers::render_settings_row(ui, &rust_i18n::t!("theme.base_scheme"), &mut 0.0, |ui| {
                    egui::ComboBox::from_id_salt("color_scheme_selector")
                        .width(100.0)
                        .selected_text(format!("{:?}", theme.color_scheme))
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Dark),
                                    rust_i18n::t!("theme.dark"),
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Dark;
                                theme_changed = true;
                            }
                            if ui
                                .selectable_label(
                                    matches!(theme.color_scheme, crate::theme::ColorScheme::Light),
                                    rust_i18n::t!("theme.light"),
                                )
                                .clicked()
                            {
                                theme.color_scheme = crate::theme::ColorScheme::Light;
                                theme_changed = true;
                            }
                        });
                });
                ui.separator();
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        egui::Frame::NONE
                            .inner_margin(egui::Margin {
                                left: 4,
                                right: 16,
                                top: 0,
                                bottom: 0,
                            })
                            .show(ui, |ui| {
                                 let head_color = theme.colors.heading_color();
                                 ui.heading(egui::RichText::new(rust_i18n::t!("theme.colors_heading")).color(head_color));
                                ui.add_space(4.0);
                                ui.vertical(|ui| {
                                        // --- HELPER CLOSURES FOR NEW OPTIONAL FIELDS ---
                                        let edit_optional_color =
    |label: &str, field: &mut Option<[u8; 3]>, default: [u8; 3], id_str: &str, ui: &mut egui::Ui| -> bool {
        let mut changed = false;
        ui.add_space(4.0);
        crate::app_helpers::render_settings_row(ui, label, &mut 0.0, |ui| {
            let mut current = field.unwrap_or(default);
            ui.spacing_mut().item_spacing.x = 4.0;
            if custom_color_picker_button(ui, &mut current, egui::Id::new(id_str)) {
                *field = Some(current);
                changed = true;
            }
            if field.is_some() {
                if ui.button("↺").on_hover_text(rust_i18n::t!("theme.reset_tooltip")).clicked() {
                    *field = None;
                    changed = true;
                }
            }
        });
        changed
    };

                                        let edit_optional_float =
    |label: &str, field: &mut Option<f32>, default: f32, range: std::ops::RangeInclusive<f32>, speed: f32, ui: &mut egui::Ui| -> bool {
        let mut changed = false;
        ui.add_space(4.0);
        crate::app_helpers::render_settings_row(ui, label, &mut 0.0, |ui| {
            let mut current = field.unwrap_or(default);
            ui.spacing_mut().item_spacing.x = 4.0;
            if ui.add(egui::DragValue::new(&mut current).speed(speed).range(range)).changed() {
                *field = Some(current);
                changed = true;
            }
            if field.is_some() {
                if ui.button("↺").on_hover_text(rust_i18n::t!("theme.reset_tooltip")).clicked() {
                    *field = None;
                    changed = true;
                }
            }
        });
        changed
    };

                                        // --- CORE UI BACKGROUNDS & ACCENTS ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!("theme.cat_core"))
                                                .strong(),
                                        );
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.bg"), &mut theme.colors.background, egui::Id::new("bg_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.selection_bg"), &mut theme.colors.selection_background, egui::Id::new("selection_bg_copy")) {
                                            theme_changed = true;
                                        }

                                        let icon_def = if theme.color_scheme
                                            == crate::theme::ColorScheme::Dark
                                        {
                                            [200, 200, 200]
                                        } else {
                                            [80, 80, 80]
                                        };
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.icon_default"),
                                            &mut theme.colors.icon_color,
                                            icon_def,
                                            "icon_def_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.icon_hover"), &mut theme.colors.icon_hover, egui::Id::new("icon_hover_copy")) {
                                            theme_changed = true;
                                        }

                                        // --- TYPOGRAPHY ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!(
                                                "theme.cat_typography"
                                            ))
                                            .strong(),
                                        );
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.fg"), &mut theme.colors.foreground, egui::Id::new("fg_copy")) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.headings"),
                                            &mut theme.colors.heading_text,
                                            [255, 255, 255],
                                            "heading_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.labels"),
                                            &mut theme.colors.label_text,
                                            [220, 220, 220],
                                            "label_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.weak_text"),
                                            &mut theme.colors.weak_text,
                                            [150, 150, 150],
                                            "weak_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.strong_text"),
                                            &mut theme.colors.strong_text,
                                            [255, 255, 255],
                                            "strong_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.hyperlinks"),
                                            &mut theme.colors.hyperlink,
                                            [90, 170, 255],
                                            "hyperlink_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }

                                        // --- MAIN TEXT EDITOR ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!("theme.cat_editor"))
                                                .strong(),
                                        );
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.editor_bg"),
                                            &mut theme.colors.editor_background,
                                            [10, 10, 10],
                                            "editor_bg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.editor_fg"),
                                            &mut theme.colors.editor_foreground,
                                            theme.colors.foreground,
                                            "editor_fg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.line_numbers"), &mut theme.colors.line_number, egui::Id::new("line_num_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.cursor"), &mut theme.colors.cursor, egui::Id::new("cursor_copy")) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.search_highlight"),
                                            &mut theme.colors.highlight,
                                            theme.colors.cursor,
                                            "highlight_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.whitespace"),
                                            &mut theme.colors.whitespace_symbols,
                                            [80, 80, 80],
                                            "whitespace_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }

                                        // --- BUTTONS & INPUTS ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!("theme.cat_buttons"))
                                                .strong(),
                                        );
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_bg"),
                                            &mut theme.colors.button_bg,
                                            [60, 60, 60],
                                            "btn_bg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_hover"),
                                            &mut theme.colors.button_hover_bg,
                                            [80, 80, 80],
                                            "btn_hover_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_active"),
                                            &mut theme.colors.button_active_bg,
                                            [100, 100, 100],
                                            "btn_active_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_fg"),
                                            &mut theme.colors.button_fg,
                                            theme.colors.foreground,
                                            "btn_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_hover_fg"),
                                            &mut theme.colors.button_hover_fg,
                                            theme.colors.foreground,
                                            "btn_hover_fg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_active_fg"),
                                            &mut theme.colors.button_active_fg,
                                            theme.colors.foreground,
                                            "btn_active_fg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_hover_border"),
                                            &mut theme.colors.button_hover_border_color,
                                            [120, 120, 120],
                                            "btn_hover_border_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.btn_active_border"),
                                            &mut theme.colors.button_active_border_color,
                                            [150, 150, 150],
                                            "btn_active_border_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }

                                        // --- WIDGETS (unified geometry + specific widget colors) ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!("theme.cat_widgets"))
                                                .strong(),
                                        );
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.widget_rounding"),
                                            &mut theme.colors.widget_rounding,
                                            2.0,
                                            0.0..=20.0,
                                            0.1,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.widget_border"),
                                            &mut theme.colors.widget_border_color,
                                            [100, 100, 100],
                                            "wgt_border_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.widget_border_width"),
                                            &mut theme.colors.widget_border_width,
                                            0.0,
                                            0.0..=5.0,
                                            0.05,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.widget_padding_x"),
                                            &mut theme.colors.widget_padding_x,
                                            4.0,
                                            0.0..=40.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.widget_padding_y"),
                                            &mut theme.colors.widget_padding_y,
                                            2.0,
                                            0.0..=40.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.widget_focus_border"),
                                            &mut theme.colors.widget_focus_border,
                                            [100, 150, 255],
                                            "wgt_focus_border_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.chk_check"),
                                            &mut theme.colors.checkbox_check,
                                            [200, 200, 200],
                                            "chk_check_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.text_edit_bg"),
                                            &mut theme.colors.text_edit_bg,
                                            [15, 15, 15],
                                            "text_edit_bg_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.selection_text"),
                                            &mut theme.colors.selection_text,
                                            [255, 255, 255],
                                            "sel_text_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.separator"),
                                            &mut theme.colors.separator,
                                            [80, 80, 80],
                                            "sep_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.separator_width"),
                                            &mut theme.colors.separator_width,
                                            1.0,
                                            0.0..=10.0,
                                            0.1,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.tree_line"),
                                            &mut theme.colors.tree_line,
                                            [100, 100, 100],
                                            "tree_line_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }

                                        // --- WINDOW & PANELS GEOMETRY ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!(
                                                "theme.cat_geometry"
                                            ))
                                            .strong(),
                                        );
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.window_rounding"),
                                            &mut theme.colors.window_rounding,
                                            4.0,
                                            0.0..=20.0,
                                            0.1,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_color(
                                            &rust_i18n::t!("theme.shadow_color"),
                                            &mut theme.colors.shadow_color,
                                            [0, 0, 0],
                                            "shadow_copy", ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.shadow_blur"),
                                            &mut theme.colors.shadow_blur,
                                            20.0,
                                            0.0..=100.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.shadow_spread"),
                                            &mut theme.colors.shadow_spread,
                                            0.0,
                                            0.0..=100.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.shadow_offset_x"),
                                            &mut theme.colors.shadow_offset_x,
                                            0.0,
                                            -100.0..=100.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }
                                        if edit_optional_float(
                                            &rust_i18n::t!("theme.shadow_offset_y"),
                                            &mut theme.colors.shadow_offset_y,
                                            0.0,
                                            -100.0..=100.0,
                                            0.5,
                                             ui,
                                        ) {
                                            theme_changed = true;
                                        }

                                        // --- SYNTAX ALERTS ---
                                        ui.label(
                                            egui::RichText::new(rust_i18n::t!("theme.cat_syntax"))
                                                .strong(),
                                        );
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.comment"), &mut theme.colors.comment, egui::Id::new("comment_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.success_label"), &mut theme.colors.success, egui::Id::new("success_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.info_label"), &mut theme.colors.info, egui::Id::new("info_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.warning_label"), &mut theme.colors.warning, egui::Id::new("warning_copy")) {
                                            theme_changed = true;
                                        }
                                        ui.add_space(4.0);
                                        if render_color_edit_row(ui, &rust_i18n::t!("theme.error_label"), &mut theme.colors.error, egui::Id::new("error_copy")) {
                                            theme_changed = true;
                                        }
                                    });
                            });
                    });
                // KLUCZOWA ZMIANA: Synchronizuj z current_theme natychmiast!
                if theme_changed {
                    theme.apply(ui.ctx());
                    self.current_theme = theme.clone();
                }
            } else {
                ui.label(rust_i18n::t!("theme.no_theme_editing"));
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
                    self.original_editing_theme = Some(theme.clone());
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.themes = crate::theme::load_themes();
                    self.status_message =
                        rust_i18n::t!("theme.saved_msg", name = theme.name).to_string();
                    self.log_info(rust_i18n::t!("theme.saved_msg", name = theme.name));
                }
                Err(e) => {
                    self.status_message =
                        rust_i18n::t!("theme.save_error", error = e.to_string()).to_string();
                    self.log_error(rust_i18n::t!("theme.save_error", error = e.to_string()));
                }
            }
        }
        self.layout_state = ls;
    }

    fn render_scanning_spinner_if_needed(
        &self,
        ui: &mut egui::Ui,
        dir_path: &Path,
        depth: usize,
        tree_on: bool,
        tree_indent: f32,
    ) {
        if self.is_directory_scanning(dir_path) {
            crate::app_helpers::center_row(ui, |ui| {
                if tree_on {
                    ui.add_space((depth + 1) as f32 * tree_indent);
                } else {
                    ui.add_space(32.0); // Simple view doesn't use tree_indent
                }
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(
                    egui::RichText::new(rust_i18n::t!("theme.verifying"))
                        .italics()
                        .size(10.0)
                        .weak(),
                );
            });
        }
    }

    fn is_directory_scanning(&self, dir_path: &Path) -> bool {
        if self.pending_access_checks.is_empty() {
            return false;
        }
        self.pending_access_checks
            .iter()
            .any(|p| p.starts_with(dir_path))
    }
}

/// Helper for theme editor to render a color row with copy/paste buttons
fn render_color_edit_row(
    ui: &mut egui::Ui,
    label: &str,
    color: &mut [u8; 3],
    row_id: egui::Id,
) -> bool {
    let mut changed = false;
    crate::app_helpers::render_settings_row(ui, label, &mut 0.0, |ui| {
        if custom_color_picker_button(ui, color, row_id.with("color_btn")) {
            changed = true;
        }
    });
    changed
}

// ─── Custom color picker with rectangular 2D field ──────────────────────────

const COLOR_N: u32 = 36; // 6×6 – identical to egui internals

fn picker_contrast(color: egui::Color32) -> egui::Color32 {
    let [r, g, b, _] = color.to_array();
    let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
    if lum < 128.0 {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
}

/// Copy of egui's private `color_slider_1d` (hue / alpha bar).
fn picker_slider_1d(
    ui: &mut egui::Ui,
    value: &mut f32,
    color_at: impl Fn(f32) -> egui::Color32,
) -> egui::Response {
    let desired = egui::vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, resp) = ui.allocate_at_least(desired, egui::Sense::click_and_drag());

    if let Some(p) = resp.interact_pointer_pos() {
        *value = egui::remap_clamp(p.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let vis = ui.style().interact(&resp);
        let mut mesh = egui::epaint::Mesh::default();
        for i in 0..=COLOR_N {
            let t = i as f32 / COLOR_N as f32;
            let c = color_at(t);
            let x = egui::lerp(rect.left()..=rect.right(), t);
            mesh.colored_vertex(egui::pos2(x, rect.top()), c);
            mesh.colored_vertex(egui::pos2(x, rect.bottom()), c);
            if i < COLOR_N {
                let b = 2 * i;
                mesh.add_triangle(b, b + 1, b + 2);
                mesh.add_triangle(b + 1, b + 2, b + 3);
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));
        ui.painter()
            .rect_stroke(rect, 0.0, vis.bg_stroke, egui::StrokeKind::Inside);

        // triangle position indicator
        let x = egui::lerp(rect.left()..=rect.right(), *value);
        let r = rect.height() / 4.0;
        let picked = color_at(*value);
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(x, rect.center().y),
                egui::pos2(x + r, rect.bottom()),
                egui::pos2(x - r, rect.bottom()),
            ],
            picked,
            egui::Stroke::new(vis.fg_stroke.width, picker_contrast(picked)),
        ));
    }
    resp
}

/// Copy of egui's private `color_slider_2d` but with separate `width` and `height`.
/// This allows the field to be a rectangle instead of a square.
fn picker_slider_2d(
    ui: &mut egui::Ui,
    x_value: &mut f32,
    y_value: &mut f32,
    width: f32,
    height: f32,
    color_at: impl Fn(f32, f32) -> egui::Color32,
) -> egui::Response {
    let (rect, resp) =
        ui.allocate_at_least(egui::vec2(width, height), egui::Sense::click_and_drag());

    if let Some(p) = resp.interact_pointer_pos() {
        *x_value = egui::remap_clamp(p.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = egui::remap_clamp(p.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let vis = ui.style().interact(&resp);
        let n = COLOR_N;
        let mut mesh = egui::epaint::Mesh::default();

        for yi in 0..=n {
            for xi in 0..=n {
                let xt = xi as f32 / n as f32;
                let yt = yi as f32 / n as f32;
                let x = egui::lerp(rect.left()..=rect.right(), xt);
                let y = egui::lerp(rect.bottom()..=rect.top(), yt);
                mesh.colored_vertex(egui::pos2(x, y), color_at(xt, yt));

                if xi < n && yi < n {
                    let row = n + 1;
                    let idx = yi * row + xi;
                    mesh.add_triangle(idx, idx + 1, idx + row);
                    mesh.add_triangle(idx + 1, idx + row + 1, idx + row);
                }
            }
        }
        ui.painter().add(egui::Shape::mesh(mesh));
        ui.painter()
            .rect_stroke(rect, 0.0, vis.bg_stroke, egui::StrokeKind::Inside);

        let cx = egui::lerp(rect.left()..=rect.right(), *x_value);
        let cy = egui::lerp(rect.bottom()..=rect.top(), *y_value);
        let picked = color_at(*x_value, *y_value);
        ui.painter().add(egui::Shape::circle_stroke(
            egui::pos2(cx, cy),
            4.0,
            egui::Stroke::new(1.5, picker_contrast(picked)),
        ));
    }
    resp
}

/// Replacement for `egui::color_picker::color_picker_color32`.
///
/// `picker_height` – height of the 2D field in pixels (e.g. 80.0).
/// Width comes from `ui.spacing().slider_width`, so the popup
/// keeps the same width as before – only the 2D field height changes.
pub fn color_picker_color32_wide(
    ui: &mut egui::Ui,
    srgba: &mut egui::Color32,
    alpha: egui::color_picker::Alpha,
    picker_height: f32,
) -> bool {
    use egui::epaint::ecolor::Hsva;

    let mut hsva = Hsva::from(*srgba);
    let mut changed = false;

    // 2D field (saturation × value)
    // NOTE: Response::changed() doesn't work for raw allocations (allocate_at_least),
    // so we track old values and compare after the slider modifies them.
    let width = ui.spacing().slider_width;
    let h = hsva.h;
    let (old_s, old_v) = (hsva.s, hsva.v);
    picker_slider_2d(
        ui,
        &mut hsva.s,
        &mut hsva.v,
        width,
        picker_height,
        |s, v| Hsva { h, s, v, a: 1.0 }.into(),
    );
    if hsva.s != old_s || hsva.v != old_v {
        changed = true;
    }

    // Hue bar
    let old_h = hsva.h;
    picker_slider_1d(ui, &mut hsva.h, |h| {
        Hsva {
            h,
            s: 1.0,
            v: 1.0,
            a: 1.0,
        }
        .into()
    });
    if hsva.h != old_h {
        changed = true;
    }

    // Alpha bar (when needed)
    match alpha {
        egui::color_picker::Alpha::Opaque => {
            hsva.a = 1.0;
        }
        egui::color_picker::Alpha::OnlyBlend | egui::color_picker::Alpha::BlendOrAdditive => {
            let (h, s, v) = (hsva.h, hsva.s, hsva.v);
            let old_a = hsva.a;
            picker_slider_1d(ui, &mut hsva.a, |a| Hsva { h, s, v, a }.into());
            if hsva.a != old_a {
                changed = true;
            }
        }
    }

    // Color swatch + R/G/B fields
    let swatch = egui::vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    egui::color_picker::show_color(ui, *srgba, swatch);

    ui.horizontal(|ui| {
        let speed = 1.0 / 255.0;
        let mut r = srgba.r() as f32 / 255.0;
        let mut g = srgba.g() as f32 / 255.0;
        let mut b = srgba.b() as f32 / 255.0;

        let dr = ui.add(
            egui::DragValue::new(&mut r)
                .speed(speed)
                .range(0.0..=1.0)
                .prefix("R ")
                .custom_formatter(|n, _| format!("{n:.3}")),
        );
        let dg = ui.add(
            egui::DragValue::new(&mut g)
                .speed(speed)
                .range(0.0..=1.0)
                .prefix("G ")
                .custom_formatter(|n, _| format!("{n:.3}")),
        );
        let db = ui.add(
            egui::DragValue::new(&mut b)
                .speed(speed)
                .range(0.0..=1.0)
                .prefix("B ")
                .custom_formatter(|n, _| format!("{n:.3}")),
        );

        if dr.changed() || dg.changed() || db.changed() {
            hsva = Hsva::from(egui::Color32::from_rgb(
                (r * 255.0).round() as u8,
                (g * 255.0).round() as u8,
                (b * 255.0).round() as u8,
            ));
            changed = true;
        }
    });

    if changed {
        *srgba = egui::Color32::from(hsva);
    }
    changed
}

fn custom_color_picker_button(ui: &mut egui::Ui, color: &mut [u8; 3], popup_id: egui::Id) -> bool {
    let mut color32 = egui::Color32::from_rgb(color[0], color[1], color[2]);
    let button_id = popup_id.with("btn");
    let area_id = popup_id.with("area");

    // Calculate natural button size like a button with 8 spaces
    let galley = egui::WidgetText::from("        ").into_galley(
        ui,
        None,
        f32::INFINITY,
        egui::TextStyle::Button,
    );
    let desired_size = galley.size() + ui.spacing().button_padding * 2.0;

    let rect = ui.allocate_exact_size(desired_size, egui::Sense::hover()).0;
    let response = ui.interact(rect, button_id, egui::Sense::click());

    let is_open = ui.data(|d| d.get_temp::<bool>(popup_id).unwrap_or(false));
    if response.clicked() {
        ui.data_mut(|d| d.insert_temp(popup_id, !is_open));
    }

    // Draw button manually to avoid egui default button hover artifacts
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, color32);

        // Enhance border visibility on hover to compensate for the fixed background color
        let mut stroke = visuals.bg_stroke;
        if response.hovered() {
            stroke.width = stroke.width.max(2.0); // Clear feedback using theme's hover border color
        } else {
            stroke.width = stroke.width.max(1.0);
        }

        ui.painter().rect_stroke(
            rect,
            visuals.corner_radius,
            stroke,
            egui::StrokeKind::Inside,
        );
    }

    let mut changed = false;

    if is_open {
        let area = egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .default_pos(response.rect.left_bottom() + egui::vec2(0.0, 4.0))
            .interactable(true);

        let area_response = area.show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                // Two-frame approach: measure the actual popup content width (driven by icon buttons)
                // on frame 1, then set slider_width to match on frame 2+
                let width_key = popup_id.with("measured_w");
                let measured = ui.data(|d| d.get_temp::<f32>(width_key));

                // Use measured width from previous frame, or small default to let buttons determine width
                ui.spacing_mut().slider_width = measured.unwrap_or(100.0);
                if color_picker_color32_wide(
                    ui,
                    &mut color32,
                    egui::color_picker::Alpha::Opaque,
                    80.0, // height of the 2D field in pixels (half of square)
                ) {
                    changed = true;
                    color[0] = color32.r();
                    color[1] = color32.g();
                    color[2] = color32.b();
                }

                // Store measured content width for next frame (stabilizes after 2 frames)
                let actual_width = ui.min_rect().width();
                if measured.is_none() || (actual_width - measured.unwrap_or(0.0)).abs() > 1.0 {
                    ui.data_mut(|d| d.insert_temp(width_key, actual_width));
                    ui.ctx().request_repaint();
                }
            });
        });

        // Close only when clicking outside both button and popup
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !area_response.response.rect.contains(pos) && !response.rect.contains(pos) {
                    ui.data_mut(|d| d.insert_temp(popup_id, false));
                }
            }
        }
    }

    changed
}
