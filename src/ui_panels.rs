use crate::app_state::{KeyStatus, LogLevel};
use crate::history::HistoryEntry;
use crate::EditorApp;
use eframe::egui;
use std::path::{Path, PathBuf};
impl EditorApp {
    /// Render settings panel
    pub(crate) fn render_settings_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
if !self.settings.hide_panel_headers {
crate::app_helpers::center_row(ui, |ui| {
ui.heading(rust_i18n::t!("settings.settings"));
});
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
ui.separator();
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
ui.heading(rust_i18n::t!("settings.security"));
crate::app_helpers::center_row(ui, |ui| {
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
crate::app_helpers::center_row(ui, |ui| {
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
crate::app_helpers::center_row(ui, |ui| {
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
ui.heading(rust_i18n::t!("settings.workspace"));
// Starting directory setting
ui.add_space(4.0);
ui.horizontal_wrapped(|ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.starting_dir")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
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
ui.heading(rust_i18n::t!("settings.editor"));
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
ui.heading(rust_i18n::t!("settings.reliability"));
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
ui.heading(rust_i18n::t!("settings.appearance"));
// Theme selection
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.theme")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.ui_font")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.ui_font_size")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.editor_font")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.editor_font_size")).selectable(false));
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.line_height")).selectable(false));
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
// Global Scroll Speed
crate::app_helpers::center_row(ui, |ui| {
    ui.add(egui::Label::new(rust_i18n::t!("settings.scroll_speed")).selectable(false));
    let mut mult = self.settings.scroll_speed_multiplier;
    if ui
        .add(
            egui::DragValue::new(&mut mult)
                .speed(0.1)
                .range(1.0..=10.0)
                .clamp_existing_to_range(true),
        )
        .on_hover_text(rust_i18n::t!("settings.scroll_speed_tooltip"))
        .changed()
    {
        self.settings.scroll_speed_multiplier = mult;
        let _ = self.settings.save();
    }
});
// Toolbar icon size
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.toolbar_icon_size")).selectable(false));
if ui
.add(
egui::DragValue::new(&mut self.settings.toolbar_icon_size)
.speed(1.0)
.range(12.0..=96.0)
.clamp_existing_to_range(true),
)
.changed()
{
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
crate::app_helpers::center_row(ui, |ui| {
ui.add(egui::Label::new(rust_i18n::t!("settings.language")).selectable(false));
let current_lang = self.settings.language.clone();
let mut changed = false;
egui::ComboBox::from_id_salt("language_selector")
.selected_text(match current_lang.as_str() {
"pl" => "Polski",
"de" => "Deutsch",
_ => "English",
})
.show_ui(ui, |ui| {
if ui.selectable_value(&mut self.settings.language, "en".to_string(), "English").clicked() { changed = true; }
if ui.selectable_value(&mut self.settings.language, "pl".to_string(), "Polski").clicked() { changed = true; }
if ui.selectable_value(&mut self.settings.language, "de".to_string(), "Deutsch").clicked() { changed = true; }
});
if changed {
rust_i18n::set_locale(&self.settings.language);
let _ = self.settings.save();
}
});
if ui
.checkbox(&mut self.settings.hide_panel_headers, rust_i18n::t!("settings.hide_panel_headers"))
.on_hover_text(rust_i18n::t!("settings.hide_panel_headers_tooltip")) // I should add this key or omit tooltip if not in yaml, wait I'll check yaml
.changed()
{
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
                        ui.heading(rust_i18n::t!("settings.window_panels"));
                        
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
                ui.heading(rust_i18n::t!("history.title"));
            }
            crate::app_helpers::center_row(ui, |ui| {
                ui.add(egui::Label::new(rust_i18n::t!("history.max_limit")).selectable(false));
                let mut temp_limit = doc_max_limit;
                if ui
                    .add(
                        egui::DragValue::new(&mut temp_limit)
                            .speed(0.5)
                            .range(1..=1000)
                            .clamp_existing_to_range(true),
                    )
                    .changed()
                {
                    self.document.set_max_history_length(temp_limit);
                    self.is_modified = true;
                    self.log_info(rust_i18n::t!("history.log_limit_set", limit = temp_limit));
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
                    } else if ui.button(format!("🗑 {}", rust_i18n::t!("history.clear_all"))).clicked() {
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
                                                "▶",
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
                                                let del = ui.button("🗑").on_hover_text(rust_i18n::t!("history.delete_entry")).clicked();
                                                let rev = ui.button("⏪").on_hover_text(rust_i18n::t!("history.revert_entry")).clicked();
                                                
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
    }
    /// Render debug panel
    pub(crate) fn render_debug_panel(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading(rust_i18n::t!("debug.title"));
            }
            crate::app_helpers::center_row(ui, |ui| {
                if ui.button(rust_i18n::t!("debug.clear")).clicked() {
                    self.debug_log.clear();
                }
                
                let mut changed = false;
                changed |= ui.checkbox(&mut self.settings.debug_show_info, rust_i18n::t!("debug.filter_info")).changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_success, rust_i18n::t!("debug.filter_success")).changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_warning, rust_i18n::t!("debug.filter_warning")).changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_error, rust_i18n::t!("debug.filter_error")).changed();
                
                if changed {
                    let _ = self.settings.save();
                }
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
                                LogLevel::Success => self.current_theme.colors.success_color(),
                                LogLevel::Warning => self.current_theme.colors.warning_color(),
                                LogLevel::Error => self.current_theme.colors.error_color(),
                            };
                            ui.colored_label(color, entry.display());
                        }
                    }
                    if visible_count == 0 && !self.debug_log.is_empty() {
                        ui.add(egui::Label::new(egui::RichText::new(rust_i18n::t!("debug.all_filtered")).italics().weak()).selectable(false));
                    }
                });
                });
        });
    }
    /// Render file tree panel
    pub(crate) fn render_file_tree(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.vertical(|ui| {
            ui.set_min_width(ui.available_width());
            if !self.settings.hide_panel_headers {
                ui.heading(rust_i18n::t!("file_tree.title"));
            }
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                                .show(ui, |ui| {
                    egui::Frame::NONE
                        .inner_margin(egui::Margin {
                            left: 4,
                            right: 0,
                            top: 0,
                            bottom: 0,
                        })
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
                        let mut dir_stack: Vec<(PathBuf, usize)> = Vec::new();

                        for (_i, entry) in entries.iter().enumerate() {
                            // Detect end of directories before processing the next entry
                            while let Some((dir_path, _depth)) = dir_stack.last() {
                                if !entry.path.starts_with(dir_path) {
                                    let (finished_dir, finished_depth) = dir_stack.pop().unwrap();
                                    self.render_scanning_spinner_if_needed(ui, &finished_dir, finished_depth, tree_on, tree_indent);
                                } else {
                                    break;
                                }
                            }

                            if entry.is_dir {
                                dir_stack.push((entry.path.clone(), entry.depth));
                            }

                            let path = &entry.path;
                            let raw_filename = path.file_name().unwrap_or_default().to_string_lossy();
                            let filename = if self.settings.capitalize_tree_names {
                                raw_filename.to_uppercase()
                            } else {
                                raw_filename.to_string()
                            };

                            // Hide undecryptable files check
                            if !entry.is_dir && raw_filename.to_lowercase().ends_with(".sen") {
                                let status = self
                                    .file_access_cache
                                    .get(path)
                                    .cloned()
                                    .unwrap_or(KeyStatus::Unknown);

                                if self.settings.hide_undecryptable_files && status != KeyStatus::Decryptable {
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
                                        "📁 ..".to_string()
                                    } else {
                                        format!(
                                            "{} {}",
                                            if entry.is_expanded { "📂" } else { "📁" },
                                            filename
                                        )
                                    };
                                    
                                    let btn_font = egui::TextStyle::Button.resolve(ui.style());
                                    let truncated_name = self.smart_truncate_text(ui, &display_name, btn_font, ui.available_width() - 4.0);
                                    
                                    if ui
                                        .add(egui::Button::new(truncated_name))
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

                                    if raw_filename.to_lowercase().ends_with(".sen") {
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
                                                KeyStatus::Unknown => self.current_theme.colors.warning_color(),
                                                _ => ui.visuals().weak_text_color(),
                                            };

                                            let mut job = egui::text::LayoutJob::default();
                                            let font_id = egui::TextStyle::Button.resolve(ui.style());
                                            
                                            // Measure space taken by dot prefix
                                            let prefix = "  ";
                                            let prefix_w = ui.painter().layout_no_wrap(prefix.to_string(), font_id.clone(), egui::Color32::BLACK).rect.width();
                                            let truncated_name = self.smart_truncate_text(ui, &display_name, font_id.clone(), ui.available_width() - prefix_w - 4.0);

                                            // Add 2 spaces to make room for the dot at the beginning
                                            job.append(prefix, 0.0, egui::text::TextFormat {
                                                font_id: font_id.clone(),
                                                ..Default::default()
                                            });
                                            job.append(&truncated_name, 0.0, egui::text::TextFormat {
                                                color: ui.visuals().text_color(),
                                                font_id,
                                                ..Default::default()
                                            });

                                            let button_resp = ui.add(egui::Button::new(job));
                                            
                                            // Draw the dot manually on top of the button's rectangle
                                            let dot_radius = 4.0;
                                            let dot_center = egui::pos2(
                                                button_resp.rect.left() + 14.0,
                                                button_resp.rect.center().y
                                            );
                                            let pulse_alpha = if self.keyfile_path.is_none() {
                                                (0.1 + 0.9 * (self.start_time.elapsed().as_secs_f32() * 3.0).cos().abs()) as f32
                                            } else {
                                                1.0
                                            };
                                            ui.painter().circle_filled(dot_center, dot_radius, color.gamma_multiply(pulse_alpha));

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
                                            let btn_font = egui::TextStyle::Button.resolve(ui.style());
                                            let truncated_name = self.smart_truncate_text(ui, &display_name, btn_font, ui.available_width() - 20.0); // account for icon
                                            if ui.add(egui::Button::new(truncated_name)).clicked() {
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
                                            let btn_font = egui::TextStyle::Button.resolve(ui.style());
                                            let truncated_name = self.smart_truncate_text(ui, &display_name, btn_font, ui.available_width() - 4.0);
                                            if ui.add(egui::Button::new(truncated_name)).clicked() {
                                            self.open_file(path.clone());
                                        }
                                    }
                                }
                            });
                            if tree_on {
                                let bottom_y = ui.cursor().top();
                                let actual_bottom_y = bottom_y - ui.spacing().item_spacing.y;
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
                            self.render_scanning_spinner_if_needed(ui, &dir_path, depth, tree_on, tree_indent);
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
                        ui.label(rust_i18n::t!("settings.no_dir_opened"));
                        if ui.button(rust_i18n::t!("settings.open_dir")).clicked() {
                            self.open_directory();
                        }
                    }
                });
                });
        });
    }
    /// Render theme editor panel
    pub(crate) fn render_theme_editor_panel(&mut self, ui: &mut egui::Ui) {
        let mut theme_to_save: Option<crate::theme::Theme> = None;
        let mut should_reset = false;
        ui.vertical(|ui| {
            if !self.settings.hide_panel_headers {
                ui.heading(rust_i18n::t!("theme.title"));
            }
            if let Some(theme) = &mut self.editing_theme {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(format!("💾 {}", rust_i18n::t!("theme.save"))).clicked() {
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
                            format!("↺ {}", rust_i18n::t!("theme.reset_default"))
                        } else {
                            format!("↺ {}", rust_i18n::t!("theme.reset_saved"))
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
                if ui.button(format!("➕ {}", rust_i18n::t!("theme.new"))).clicked() {
                    let mut new_theme = self.current_theme.clone();
                    new_theme.name = format!("{} {}", new_theme.name, rust_i18n::t!("theme.copy_suffix"));
                    self.editing_theme = Some(new_theme.clone());
                    self.original_editing_theme = Some(new_theme);
                    self.show_delete_theme_confirmation = false; // Reset confirmation
                }
                // Delete button with confirmation
                if let Some(theme) = &self.editing_theme {
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    if !is_builtin {
                        if !self.show_delete_theme_confirmation {
                            if ui.button(format!("🗑 {}", rust_i18n::t!("theme.delete"))).clicked() {
                                self.show_delete_theme_confirmation = true;
                            }
                        } else {
                            ui.label(egui::RichText::new(rust_i18n::t!("settings.are_you_sure")).color(self.current_theme.colors.error_color()));
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
                let copied_color = &mut self.copied_color;
                let last_copied_id = &mut self.last_copied_id;
                let last_copied_time = &mut self.last_copied_time;
                crate::app_helpers::center_row(ui, |ui| {
                    ui.label(rust_i18n::t!("theme.name_label"));
                    ui.add(egui::TextEdit::singleline(&mut theme.name).desired_width(100.0));
                });
                crate::app_helpers::center_row(ui, |ui| {
                    ui.label(rust_i18n::t!("theme.base_scheme"));
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
                                if !self.settings.hide_panel_headers {
                                    ui.heading(rust_i18n::t!("theme.colors_heading"));
                                    ui.add_space(4.0);
                                }
                        egui::Grid::new("all_theme_colors_grid")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .striped(false)
                            .show(ui, |ui| {

                                // --- HELPER CLOSURES FOR NEW OPTIONAL FIELDS ---
                                let icons = &self.icons;
                                let edit_optional_color = |label: &str, field: &mut Option<[u8; 3]>, default: [u8; 3], id_str: &str, copied_color: &mut Option<[u8; 3]>, last_copied_id: &mut Option<egui::Id>, last_copied_time: &mut f64, ui: &mut egui::Ui| -> bool {
                                    let mut changed = false;
                                    ui.add(egui::Label::new(label).selectable(false));
                                    let mut current = field.unwrap_or(default);
                                    crate::app_helpers::center_row(ui, |ui| {
                                        if ui.color_edit_button_srgb(&mut current).changed() {
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
                                    if let Some(new_color) = render_copy_paste_buttons(
                                        ui,
                                        current,
                                        copied_color,
                                        last_copied_id,
                                        last_copied_time,
                                        egui::Id::new(id_str),
                                        icons,
                                    ) {
                                        *field = Some(new_color);
                                        changed = true;
                                    }
                                    ui.end_row();
                                    changed
                                };

                                let edit_optional_float = |label: &str, field: &mut Option<f32>, default: f32, range: std::ops::RangeInclusive<f32>, speed: f32, ui: &mut egui::Ui| -> bool {
                                    let mut changed = false;
                                    ui.add(egui::Label::new(label).selectable(false));
                                    let mut current = field.unwrap_or(default);
                                    crate::app_helpers::center_row(ui, |ui| {
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
                                    ui.label(""); // Empty label for copy/paste column
                                    ui.end_row();
                                    changed
                                };

                                // --- CORE UI BACKGROUNDS & ACCENTS ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_core")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.bg")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.background, copied_color, last_copied_id, last_copied_time, egui::Id::new("bg_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.selection_bg")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.selection_background, copied_color, last_copied_id, last_copied_time, egui::Id::new("selection_bg_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                
                                let icon_def = if theme.color_scheme == crate::theme::ColorScheme::Dark { [200, 200, 200] } else { [80, 80, 80] };
                                if edit_optional_color(&rust_i18n::t!("theme.icon_default") , &mut theme.colors.icon_color, icon_def, "icon_def_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                ui.add(egui::Label::new(rust_i18n::t!("theme.icon_hover")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.icon_hover, copied_color, last_copied_id, last_copied_time, egui::Id::new("icon_hover_copy"), icons) { theme_changed = true; }
                                ui.end_row();

                                // --- TYPOGRAPHY ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_typography")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.fg")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.foreground, copied_color, last_copied_id, last_copied_time, egui::Id::new("fg_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.headings") , &mut theme.colors.heading_text, [255, 255, 255], "heading_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.labels") , &mut theme.colors.label_text, [220, 220, 220], "label_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.weak_text") , &mut theme.colors.weak_text, [150, 150, 150], "weak_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.strong_text") , &mut theme.colors.strong_text, [255, 255, 255], "strong_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.hyperlinks") , &mut theme.colors.hyperlink, [90, 170, 255], "hyperlink_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }

                                // --- MAIN TEXT EDITOR ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_editor")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.editor_bg") , &mut theme.colors.editor_background, [10, 10, 10], "editor_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.editor_fg") , &mut theme.colors.editor_foreground, theme.colors.foreground, "editor_fg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                ui.add(egui::Label::new(rust_i18n::t!("theme.line_numbers")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.line_number, copied_color, last_copied_id, last_copied_time, egui::Id::new("line_num_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.cursor")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.cursor, copied_color, last_copied_id, last_copied_time, egui::Id::new("cursor_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.search_highlight") , &mut theme.colors.highlight, theme.colors.cursor, "highlight_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.whitespace") , &mut theme.colors.whitespace_symbols, [80,80,80], "whitespace_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }

                                // --- BUTTONS ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_buttons")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.btn_bg") , &mut theme.colors.button_bg, [60, 60, 60], "btn_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_hover") , &mut theme.colors.button_hover_bg, [80, 80, 80], "btn_hover_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_active") , &mut theme.colors.button_active_bg, [100, 100, 100], "btn_active_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_fg") , &mut theme.colors.button_fg, theme.colors.foreground, "btn_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_hover_fg") , &mut theme.colors.button_hover_fg, theme.colors.foreground, "btn_hover_fg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_active_fg") , &mut theme.colors.button_active_fg, theme.colors.foreground, "btn_active_fg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.btn_rounding") , &mut theme.colors.button_rounding, 2.0, 0.0..=20.0, 0.1, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_border") , &mut theme.colors.button_border_color, [100, 100, 100], "btn_border_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_hover_border") , &mut theme.colors.button_hover_border_color, [120, 120, 120], "btn_hover_border_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.btn_active_border") , &mut theme.colors.button_active_border_color, [150, 150, 150], "btn_active_border_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.btn_border_width") , &mut theme.colors.button_border_width, 0.0, 0.0..=5.0, 0.05, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.btn_padding_x") , &mut theme.colors.button_padding_x, 4.0, 0.0..=40.0, 0.5, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.btn_padding_y") , &mut theme.colors.button_padding_y, 2.0, 0.0..=40.0, 0.5, ui) { theme_changed = true; }

                                // --- INPUT ELEMENTS ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_inputs")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.input_bg") , &mut theme.colors.input_bg, [30, 30, 30], "input_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.input_fg") , &mut theme.colors.input_fg, theme.colors.foreground, "input_fg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.input_border") , &mut theme.colors.input_border_color, [100, 100, 100], "input_border_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.input_focus_border") , &mut theme.colors.input_focus_border_color, [100, 150, 255], "input_focus_border_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.input_rounding") , &mut theme.colors.input_rounding, 2.0, 0.0..=20.0, 0.1, ui) { theme_changed = true; }

                                // --- WIDGETS ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_widgets")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color(&rust_i18n::t!("theme.chk_bg") , &mut theme.colors.checkbox_bg, [50, 50, 50], "chk_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.chk_check") , &mut theme.colors.checkbox_check, [200, 200, 200], "chk_check_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.slider_rail") , &mut theme.colors.slider_rail, [60, 60, 60], "slide_rail_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.slider_thumb") , &mut theme.colors.slider_thumb, [180, 180, 180], "slide_thumb_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.scroll_bg") , &mut theme.colors.scrollbar_bg, [30, 30, 30], "scroll_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.scroll_thumb") , &mut theme.colors.scrollbar_thumb, [120, 120, 120], "scroll_thumb_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.tooltip_bg") , &mut theme.colors.tooltip_bg, [20, 20, 20], "tooltip_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.tooltip_text") , &mut theme.colors.tooltip_text, [220, 220, 220], "tooltip_txt_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.text_edit_bg") , &mut theme.colors.text_edit_bg, [15, 15, 15], "text_edit_bg_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.focus_outline") , &mut theme.colors.focus_outline, [100, 150, 255], "focus_outline_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.selection_text") , &mut theme.colors.selection_text, [255, 255, 255], "sel_text_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.separator") , &mut theme.colors.separator, [80, 80, 80], "sep_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.separator_width") , &mut theme.colors.separator_width, 1.0, 0.0..=10.0, 0.1, ui) { theme_changed = true; }

                                // --- WINDOW & PANELS GEOMETRY ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_geometry")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_float(&rust_i18n::t!("theme.window_rounding") , &mut theme.colors.window_rounding, 4.0, 0.0..=20.0, 0.1, ui) { theme_changed = true; }
                                if edit_optional_color(&rust_i18n::t!("theme.shadow_color") , &mut theme.colors.shadow_color, [0, 0, 0], "shadow_copy", copied_color, last_copied_id, last_copied_time, ui) { theme_changed = true; }
                                if edit_optional_float(&rust_i18n::t!("theme.shadow_blur") , &mut theme.colors.shadow_blur, 10.0, 0.0..=50.0, 0.5, ui) { theme_changed = true; }

                                // --- SYNTAX ALERTS ---
                                ui.label(egui::RichText::new(rust_i18n::t!("theme.cat_syntax")).strong()); ui.label(""); ui.label(""); ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.comment")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.comment, copied_color, last_copied_id, last_copied_time, egui::Id::new("comment_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.success_label")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.success, copied_color, last_copied_id, last_copied_time, egui::Id::new("success_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.info_label")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.info, copied_color, last_copied_id, last_copied_time, egui::Id::new("info_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.warning_label")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.warning, copied_color, last_copied_id, last_copied_time, egui::Id::new("warning_copy"), icons) { theme_changed = true; }
                                ui.end_row();
                                ui.add(egui::Label::new(rust_i18n::t!("theme.error_label")).selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.error, copied_color, last_copied_id, last_copied_time, egui::Id::new("error_copy"), icons) { theme_changed = true; }
                                ui.end_row();
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
                    self.status_message = rust_i18n::t!("theme.saved_msg", name = theme.name).to_string();
                    self.log_info(rust_i18n::t!("theme.saved_msg", name = theme.name));
                }
                Err(e) => {
                    self.status_message = rust_i18n::t!("theme.save_error", error = e.to_string()).to_string();
                    self.log_error(rust_i18n::t!("theme.save_error", error = e.to_string()));
                }
            }
        }
    }

    fn render_scanning_spinner_if_needed(&self, ui: &mut egui::Ui, dir_path: &Path, depth: usize, tree_on: bool, tree_indent: f32) {
        if self.is_directory_scanning(dir_path) {
            crate::app_helpers::center_row(ui, |ui| {
                if tree_on {
                    ui.add_space((depth + 1) as f32 * tree_indent);
                } else {
                    ui.add_space(32.0); // Simple view doesn't use tree_indent
                }
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(egui::RichText::new(rust_i18n::t!("theme.verifying")).italics().size(10.0).weak());
            });
        }
    }

    fn is_directory_scanning(&self, dir_path: &Path) -> bool {
        if self.pending_access_checks.is_empty() {
            return false;
        }
        self.pending_access_checks.iter().any(|p| p.starts_with(dir_path))
    }
}

/// Helper for theme editor to render a color row with copy/paste buttons
fn render_color_edit_row(
    ui: &mut egui::Ui,
    color: &mut [u8; 3],
    copied_color: &mut Option<[u8; 3]>,
    last_copied_id: &mut Option<egui::Id>,
    last_copied_time: &mut f64,
    row_id: egui::Id,
    icons: &crate::icons::Icons,
) -> bool {
    let mut changed = false;
    // Column 2: Picker
    if ui.color_edit_button_srgb(color).changed() {
        changed = true;
    }

    // Column 3: Buttons
    if let Some(new_color) = render_copy_paste_buttons(
        ui,
        *color,
        copied_color,
        last_copied_id,
        last_copied_time,
        row_id,
        icons,
    ) {
        *color = new_color;
        changed = true;
    }
    changed
}

/// Helper for rendering Copy/Paste buttons with animation
fn render_copy_paste_buttons(
    ui: &mut egui::Ui,
    current_color: [u8; 3],
    copied_color: &mut Option<[u8; 3]>,
    last_copied_id: &mut Option<egui::Id>,
    last_copied_time: &mut f64,
    row_id: egui::Id,
    icons: &crate::icons::Icons,
) -> Option<[u8; 3]> {
    let mut paste_color = None;
    crate::app_helpers::center_row(ui, |ui| {
        let time = ui.input(|i| i.time);
        
        // --- 1. PASTE BUTTON (Left) ---
        // Only visible when a color is in clipboard
        if let Some(c) = *copied_color {
            let paste_btn = egui::Button::image(egui::Image::new(&icons.paste)
                .max_width(16.0)
                .tint(ui.visuals().widgets.inactive.fg_stroke.color))
                .frame(false);
            let paste_res = ui.add(paste_btn).on_hover_text(rust_i18n::t!("theme.paste_color"));
            if paste_res.clicked() {
                paste_color = Some(c);
            }
            ui.add_space(4.0); // Small gap between buttons
        }

        // --- 2. COPY BUTTON (Right) ---
        // Infinite pulse if this is the source of copied color
        let is_source = Some(row_id) == *last_copied_id;
        let mut alpha = 0.0;
        if is_source {
            // Infinite pulse: fade in and out 
            alpha = ((time * 3.0).sin() * 0.5 + 0.5) as f32;
            ui.ctx().request_repaint();
        }

        let tint = if alpha > 0.0 {
            ui.visuals().selection.bg_fill.gamma_multiply(alpha)
        } else {
            ui.visuals().widgets.inactive.fg_stroke.color
        };
        
        let copy_btn = egui::Button::image(egui::Image::new(&icons.copy)
            .max_width(16.0)
            .tint(tint))
            .frame(false);
            
        let copy_res = ui.add(copy_btn).on_hover_text(rust_i18n::t!("theme.copy_color"));

        if copy_res.clicked() {
            *copied_color = Some(current_color);
            *last_copied_id = Some(row_id);
            *last_copied_time = time;
            ui.ctx().request_repaint();
        }
    });
    paste_color
}
