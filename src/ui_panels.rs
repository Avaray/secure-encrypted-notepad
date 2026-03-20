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
ui.horizontal(|ui| {
ui.heading("Settings");
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
ui.add(egui::Label::new(egui::RichText::new("Security").heading()).selectable(false));
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
                ui.add(egui::Label::new(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color())).selectable(false));
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
                ui.add(egui::Label::new("Current:").selectable(false));
if let Some(path) = &self.settings.global_keyfile_path {
if self.settings.show_keyfile_paths {
ui.add(egui::Label::new(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color())).selectable(false));
} else {
ui.add(egui::Label::new(egui::RichText::new("Secured").color(self.current_theme.colors.success_color())).selectable(false));
}
} else {
    ui.add(egui::Label::new(egui::RichText::new("None").color(self.current_theme.colors.info_color())).selectable(false));
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
ui.add(egui::Label::new(egui::RichText::new("Auto-Backup").strong()).selectable(false));
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
            ui.add(egui::Label::new(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color())).selectable(false));
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
    ui.add(egui::Label::new("Current:").selectable(false));
    if let Some(path) = &self.settings.auto_backup_dir {
        if self.settings.show_directory_paths {
            ui.add(egui::Label::new(egui::RichText::new(path.to_string_lossy()).color(self.current_theme.colors.warning_color())).selectable(false));
        } else {
            ui.add(egui::Label::new(egui::RichText::new("Secured").color(self.current_theme.colors.success_color())).selectable(false));
        }
    } else {
        ui.add(egui::Label::new(egui::RichText::new("None").color(self.current_theme.colors.info_color())).selectable(false));
    }
});

#[cfg(target_os = "windows")]
{
    ui.add_space(8.0);
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
ui.add(egui::Label::new(egui::RichText::new("Workspace / File Tree").heading()).selectable(false));
// Starting directory setting
ui.add_space(4.0);
ui.horizontal_wrapped(|ui| {
ui.add(egui::Label::new("Starting directory:").selectable(false));
if let Some(ref dir) = self.settings.file_tree_starting_dir {
if self.settings.show_directory_paths {
ui.add(egui::Label::new(
egui::RichText::new(dir.display().to_string())
.color(self.current_theme.colors.warning_color())
).selectable(false));
} else {
ui.add(egui::Label::new(egui::RichText::new("Secured").color(self.current_theme.colors.success_color())).selectable(false));
}
} else {
ui.add(egui::Label::new(egui::RichText::new("Not set").color(self.current_theme.colors.info_color())).selectable(false));
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
                    ui.add(egui::Label::new(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color())).selectable(false));
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
.checkbox(&mut self.settings.hide_undecryptable_files, "Hide undecryptable files")
.on_hover_text("Hide .sen files that cannot be opened with the currently loaded keyfile.")
.changed()
{
let _ = self.settings.save();
self.refresh_file_tree();
}

if ui
.checkbox(&mut self.settings.hide_filename_in_title, "Hide filename in Window Title")
.on_hover_text("Enhances privacy by not displaying the current filename in the window title or taskbar.")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.capitalize_tree_names, "Capitalize names")
.on_hover_text("Always display file and folder names in UPPERCASE in the file tree.")
.changed()
{
let _ = self.settings.save();
}
if ui
.checkbox(&mut self.settings.tree_style_file_tree, "Use Expandable Tree View")
.on_hover_text("Display folders hierarchically and toggle expansion on click")
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
ui.add(egui::Label::new(egui::RichText::new("Editor").heading()).selectable(false));
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
ui.add(egui::Label::new("Cursor Shape:").selectable(false));
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
ui.add(egui::Label::new("Tab Size:").selectable(false));
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
.checkbox(&mut self.settings.use_spaces_for_tabs, "Use spaces for tabs")
.changed()
{
let _ = self.settings.save();
}

ui.horizontal(|ui| {
    ui.add(egui::Label::new("Comment Prefix:").selectable(false));
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
ui.add(egui::Label::new("Max Lines Limit:").selectable(false));
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
ui.add(egui::Label::new(egui::RichText::new("(No limit)").italics().weak()).selectable(false));
}
})
.response
.on_hover_text("Maximum number of lines allowed in the editor. Set to 0 to disable the limit.");
// History capacity
ui.horizontal(|ui| {
ui.add(egui::Label::new("Default history limit:").selectable(false));
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
ui.add(egui::Label::new(egui::RichText::new("Reliability").heading()).selectable(false));
ui.vertical(|ui| {
ui.set_min_width(ui.available_width());
ui.group(|ui| {
ui.set_min_width(ui.available_width());
ui.add(egui::Label::new("Auto Save").selectable(false));
if ui
.checkbox(&mut self.settings.auto_save_on_focus_loss, "Auto-save on focus loss")
.on_hover_text("Automatically saves inside the .sen file when application loses focus.")
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
                    ui.add(egui::Label::new("Inactivity (seconds):").selectable(false));
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
ui.add(egui::Label::new(egui::RichText::new("Appearance").heading()).selectable(false));
// Theme selection
ui.horizontal(|ui| {
ui.add(egui::Label::new("Theme:").selectable(false));
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
ui.add(egui::Label::new("UI Font:").selectable(false));
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
ui.add(egui::Label::new("UI Font Size:").selectable(false));
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
ui.add(egui::Label::new("Editor Font:").selectable(false));
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
ui.add(egui::Label::new("Editor Font Size:").selectable(false));
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
ui.add(egui::Label::new("Line Height:").selectable(false));
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
ui.add(egui::Label::new("Toolbar Icon Size:").selectable(false));
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
ui.horizontal(|ui| {
ui.add(egui::Label::new("Toolbar Position:").selectable(false));
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

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        
                        // =========================================================================
                        // 5. SYSTEM
                        // =========================================================================
                        ui.add(egui::Label::new(egui::RichText::new("System").heading()).selectable(false));
                        
                        #[cfg(any(target_os = "windows", target_os = "linux"))]
                        {
                            ui.add_space(4.0);
                            if ui.button("🔗 Associate .sen files with this program")
                                .on_hover_text("Associate .sen files with this executable. This allows you to open encrypted files by double-clicking them.")
                                .clicked() {
                                self.associate_sen_files();
                            }
                            ui.add(egui::Label::new(egui::RichText::new("Requires current executable to be in a stable location.").small().weak()).selectable(false));
                        }
                        
                        #[cfg(target_os = "macos")]
                        {
                            ui.add_space(4.0);
                            ui.add(egui::Label::new("File association on macOS must be set manually via 'Get Info' -> 'Open with' -> 'Change All'.").weak());
                        }

                        ui.add_space(8.0);

                        if ui
                            .checkbox(&mut self.settings.single_instance, "Single instance mode")
                            .on_hover_text("Prevent multiple SEN windows. When enabled, opening a .sen file while SEN is already running will load the file in the existing window. Requires restart.")
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
                ui.heading("History");
            }
            ui.horizontal(|ui| {
                ui.add(egui::Label::new("Max History for this file:").selectable(false));
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
                    self.log_info(format!("Document history limit set to {}", temp_limit));
                }
            });
            let history_status_color = if history_len > doc_max_limit {
                self.current_theme.colors.warning_color()
            } else {
                ui.visuals().widgets.noninteractive.fg_stroke.color // Default weak color
            };
            
            ui.add(egui::Label::new(
                egui::RichText::new(format!(
                    "Current: {}/{} entries",
                    history_len, doc_max_limit
                ))
                .color(history_status_color),
            ).selectable(false));

            if history_len > doc_max_limit {
                let to_delete = history_len - doc_max_limit;
                ui.add(egui::Label::new(
                    egui::RichText::new(format!("{} entries will be deleted upon save", to_delete))
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

                ui.horizontal(|ui| {
                    if self.show_clear_history_confirmation {
                        ui.add(egui::Label::new(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color())).selectable(false));
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
                                ui.add(egui::Label::new("No history").selectable(false));
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
                                                let del = ui.button("🗑").on_hover_text("Delete this entry").clicked();
                                                let rev = ui.button("⏪").on_hover_text("Revert (Set as current and delete all newer entries)").clicked();
                                                
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
                ui.heading("Debug Log");
            }
            ui.horizontal(|ui| {
                if ui.button("Clear").clicked() {
                    self.debug_log.clear();
                }
                
                let mut changed = false;
                changed |= ui.checkbox(&mut self.settings.debug_show_info, "Info").changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_success, "Success").changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_warning, "Warning").changed();
                changed |= ui.checkbox(&mut self.settings.debug_show_error, "Error").changed();
                
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
                        ui.add(egui::Label::new(egui::RichText::new("All entries filtered out.").italics().weak()).selectable(false));
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
                ui.heading("Files");
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
                        ui.label("No directory opened");
                        if ui.button("Open Directory").clicked() {
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
                ui.heading("Theme Editor");
            }
            if let Some(theme) = &mut self.editing_theme {
                ui.horizontal_wrapped(|ui| {
                    if ui.button("💾 Save Theme").clicked() {
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
                            "↺ Reset to Default".to_string()
                        } else {
                            "↺ Reset to Saved".to_string()
                        };
                        if ui.button(reset_text).clicked() {
                            should_reset = true;
                        }
                    }
                });
            }

            ui.separator();

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
                                self.original_editing_theme = Some(theme.clone());
                                self.current_theme = theme.clone();
                                self.settings.theme_name = theme.name.clone();
                                self.show_delete_theme_confirmation = false; // Reset confirmation on theme change
                                self.apply_theme(ui.ctx());
                                let _ = self.settings.save();
                            }
                        }
                    });
                if ui.button("➕ New").clicked() {
                    let mut new_theme = self.current_theme.clone();
                    new_theme.name = format!("{} (Copy)", new_theme.name);
                    self.editing_theme = Some(new_theme.clone());
                    self.original_editing_theme = Some(new_theme);
                    self.show_delete_theme_confirmation = false; // Reset confirmation
                }
                // Delete button with confirmation
                if let Some(theme) = &self.editing_theme {
                    let is_builtin = theme.name == "Dark" || theme.name == "Light";
                    if !is_builtin {
                        if !self.show_delete_theme_confirmation {
                            if ui.button("🗑 Delete").clicked() {
                                self.show_delete_theme_confirmation = true;
                            }
                        } else {
                            ui.label(egui::RichText::new("Are you sure?").color(self.current_theme.colors.error_color()));
                            if ui.button("Yes").clicked() {
                                let _ = crate::theme::delete_theme(&theme.name);
                                self.themes = crate::theme::load_themes(); // Reload
                                self.editing_theme = Some(crate::theme::Theme::dark());
                                self.show_delete_theme_confirmation = false;
                                // Reset to safe default
                            }
                            if ui.button("No").clicked() {
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
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.add(egui::TextEdit::singleline(&mut theme.name).desired_width(100.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Base Scheme:");
                    egui::ComboBox::from_id_salt("color_scheme_selector")
                        .width(100.0)
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
                                    ui.heading("Colors");
                                    ui.add_space(4.0);
                                }
                        egui::Grid::new("all_theme_colors_grid")
                            .num_columns(3)
                            .spacing([20.0, 4.0])
                            .striped(false)
                            .show(ui, |ui| {
                                // Background
                                ui.add(egui::Label::new("UI Background:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.background, copied_color, last_copied_id, last_copied_time, egui::Id::new("bg_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Foreground
                                ui.add(egui::Label::new("UI Foreground:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.foreground, copied_color, last_copied_id, last_copied_time, egui::Id::new("fg_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Editor Foreground
                                ui.add(egui::Label::new("Editor Foreground:").selectable(false));
                                let mut editor_fg = theme
                                    .colors
                                    .editor_foreground
                                    .unwrap_or(theme.colors.foreground);
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    editor_fg,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("editor_fg_copy"),
                                ) {
                                    theme.colors.editor_foreground = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Removed Panel Background, Surface, and Surface Highlight
                                // Selection Background
                                ui.add(egui::Label::new("Selection Background:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.selection_background, copied_color, last_copied_id, last_copied_time, egui::Id::new("selection_bg_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Cursor
                                ui.add(egui::Label::new("Cursor Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.cursor, copied_color, last_copied_id, last_copied_time, egui::Id::new("cursor_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Icon Hover
                                ui.add(egui::Label::new("Icon Hover Tint:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.icon_hover, copied_color, last_copied_id, last_copied_time, egui::Id::new("icon_hover_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Icon Default (New)
                                ui.add(egui::Label::new("Icon Default Tint:").selectable(false));
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    icon_def,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("icon_def_copy"),
                                ) {
                                    theme.colors.icon_color = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Highlight (New)
                                ui.add(egui::Label::new("Search Highlight:").selectable(false));
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    highlight,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("highlight_copy"),
                                ) {
                                    theme.colors.highlight = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Button Background
                                ui.add(egui::Label::new("Button Background:").selectable(false));
                                let mut bg = theme.colors.button_bg.unwrap_or([60, 60, 60]); // Approx default
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    bg,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("btn_bg_copy"),
                                ) {
                                    theme.colors.button_bg = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Button Hover Background
                                ui.add(egui::Label::new("Button Hover:").selectable(false));
                                let mut h_bg = theme
                                    .colors
                                    .button_hover_bg
                                    .unwrap_or(theme.colors.background); // Fallback
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    h_bg,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("btn_hover_copy"),
                                ) {
                                    theme.colors.button_hover_bg = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Button Active Background
                                ui.add(egui::Label::new("Button Active:").selectable(false));
                                let mut a_bg = theme
                                    .colors
                                    .button_active_bg
                                    .unwrap_or(theme.colors.background);
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    a_bg,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("btn_active_copy"),
                                ) {
                                    theme.colors.button_active_bg = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Button Foreground
                                ui.add(egui::Label::new("Button Text:").selectable(false));
                                let mut fg =
                                    theme.colors.button_fg.unwrap_or(theme.colors.foreground);
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    fg,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("btn_text_copy"),
                                ) {
                                    theme.colors.button_fg = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Separator
                                ui.add(egui::Label::new("Separator:").selectable(false));
                                let mut sep = theme.colors.separator.unwrap_or([80, 80, 80]);
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    sep,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("sep_copy"),
                                ) {
                                    theme.colors.separator = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Line Number Color
                                ui.add(egui::Label::new("Line Numbers:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.line_number, copied_color, last_copied_id, last_copied_time, egui::Id::new("line_num_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Comment Color
                                ui.add(egui::Label::new("Comments:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.comment, copied_color, last_copied_id, last_copied_time, egui::Id::new("comment_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                // Whitespace Symbols Color
                                ui.add(egui::Label::new("Whitespace Symbols:").selectable(false));
                                let mut ws =
                                    theme.colors.whitespace_symbols.unwrap_or_else(|| {
                                        let c = theme.colors.comment;
                                        [
                                            (c[0] as f32 * 0.4) as u8,
                                            (c[1] as f32 * 0.4) as u8,
                                            (c[2] as f32 * 0.4) as u8,
                                        ]
                                    });
                                ui.horizontal(|ui| {
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
                                if let Some(new_color) = render_copy_paste_buttons(
                                    ui,
                                    ws,
                                    copied_color,
                                    last_copied_id,
                                    last_copied_time,
                                    egui::Id::new("ws_copy"),
                                ) {
                                    theme.colors.whitespace_symbols = Some(new_color);
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.label("");
                                ui.label("");
                                ui.end_row();
                                // Status colors section
                                ui.add(egui::Label::new("Info Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.info, copied_color, last_copied_id, last_copied_time, egui::Id::new("info_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.add(egui::Label::new("Success Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.success, copied_color, last_copied_id, last_copied_time, egui::Id::new("success_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.add(egui::Label::new("Warning Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.warning, copied_color, last_copied_id, last_copied_time, egui::Id::new("warning_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();
                                ui.add(egui::Label::new("Error Color:").selectable(false));
                                if render_color_edit_row(ui, &mut theme.colors.error, copied_color, last_copied_id, last_copied_time, egui::Id::new("error_copy")) {
                                    theme_changed = true;
                                }
                                ui.end_row();

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

                                let mut edit_optional_float = |label: &str, field: &mut Option<f32>, default: f32, range: std::ops::RangeInclusive<f32>, ui: &mut egui::Ui| -> bool {
                                    let mut changed = false;
                                    ui.add(egui::Label::new(label).selectable(false));
                                    let mut current = field.unwrap_or(default);
                                    ui.horizontal(|ui| {
                                        if ui.add(egui::Slider::new(&mut current, range)).changed() {
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

                                ui.label(egui::RichText::new("--- TYPOGRAPHY ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Weak Text:", &mut theme.colors.weak_text, [150, 150, 150], "weak_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Strong Text:", &mut theme.colors.strong_text, [255, 255, 255], "strong_text_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Hyperlinks:", &mut theme.colors.hyperlink, [90, 170, 255], "hyperlink_copy", ui) { theme_changed = true; }

                                ui.label(egui::RichText::new("--- WIDGETS ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Checkbox BG:", &mut theme.colors.checkbox_bg, [50, 50, 50], "chk_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Checkbox Check:", &mut theme.colors.checkbox_check, [200, 200, 200], "chk_check_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Slider Rail:", &mut theme.colors.slider_rail, [60, 60, 60], "slide_rail_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Slider Thumb:", &mut theme.colors.slider_thumb, [180, 180, 180], "slide_thumb_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Scrollbar BG:", &mut theme.colors.scrollbar_bg, [30, 30, 30], "scroll_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Scrollbar Thumb:", &mut theme.colors.scrollbar_thumb, [120, 120, 120], "scroll_thumb_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Tooltip BG:", &mut theme.colors.tooltip_bg, [20, 20, 20], "tooltip_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Tooltip Text:", &mut theme.colors.tooltip_text, [220, 220, 220], "tooltip_txt_copy", ui) { theme_changed = true; }

                                ui.label(egui::RichText::new("--- TEXT EDIT ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_color("Text Edit BG:", &mut theme.colors.text_edit_bg, [15, 15, 15], "text_edit_bg_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Focus Outline:", &mut theme.colors.focus_outline, [100, 150, 255], "focus_outline_copy", ui) { theme_changed = true; }
                                if edit_optional_color("Selection Text:", &mut theme.colors.selection_text, [255, 255, 255], "sel_text_copy", ui) { theme_changed = true; }

                                ui.label(egui::RichText::new("--- GEOMETRY ---").strong()); ui.label(""); ui.label(""); ui.end_row();
                                if edit_optional_float("Window Rounding:", &mut theme.colors.window_rounding, 4.0, 0.0..=20.0, ui) { theme_changed = true; }
                                if edit_optional_float("Button Rounding:", &mut theme.colors.button_rounding, 2.0, 0.0..=20.0, ui) { theme_changed = true; }
                                if edit_optional_float("Button Border Width:", &mut theme.colors.button_border_width, 0.0, 0.0..=5.0, ui) { theme_changed = true; }
                                if edit_optional_color("Button Border Color:", &mut theme.colors.button_border_color, [100, 100, 100], "btn_border_copy", ui) { theme_changed = true; }
                                if edit_optional_float("Separator Width:", &mut theme.colors.separator_width, 1.0, 0.0..=10.0, ui) { theme_changed = true; }
                                if edit_optional_color("Shadow Color:", &mut theme.colors.shadow_color, [0, 0, 0], "shadow_copy", ui) { theme_changed = true; }
                            });
                    });
                });
                // KLUCZOWA ZMIANA: Synchronizuj z current_theme natychmiast!
                if theme_changed {
                    theme.apply(ui.ctx());
                    self.current_theme = theme.clone();
                }
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
                    self.original_editing_theme = Some(theme.clone());
                    self.settings.theme_name = theme.name.clone();
                    let _ = self.settings.save();
                    self.themes = crate::theme::load_themes();
                    self.status_message = format!("Theme saved: {}", theme.name);
                    self.log_info(format!("Theme saved successfully: {}", theme.name));
                }
                Err(e) => {
                    self.status_message = format!("Error saving theme: {}", e);
                    self.log_error(format!("Failed to save theme: {}", e));
                }
            }
        }
    }

    fn render_scanning_spinner_if_needed(&self, ui: &mut egui::Ui, dir_path: &Path, depth: usize, tree_on: bool, tree_indent: f32) {
        if self.is_directory_scanning(dir_path) {
            ui.horizontal(|ui| {
                if tree_on {
                    ui.add_space((depth + 1) as f32 * tree_indent);
                } else {
                    ui.add_space(32.0); // Simple view doesn't use tree_indent
                }
                ui.add(egui::Spinner::new().size(12.0));
                ui.label(egui::RichText::new("verifying...").italics().size(10.0).weak());
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
) -> Option<[u8; 3]> {
    let mut paste_color = None;
    ui.horizontal(|ui| {
        // Animation logic
        let duration = 0.5;
        let time = ui.input(|i| i.time);
        let mut alpha = 0.0;
        if Some(row_id) == *last_copied_id {
            let elapsed = time - *last_copied_time;
            if elapsed < duration {
                // Flash/Fade effect: fast pulse then fade
                alpha = (((elapsed * 20.0).sin() * 0.5 + 0.5) * (1.0 - elapsed / duration)).clamp(0.0, 1.0);
                ui.ctx().request_repaint();
            } else {
                *last_copied_id = None;
            }
        }

        let copy_btn = egui::Button::new("📋");

        let copy_res = if alpha > 0.0 {
            let bg_color = ui.visuals().selection.bg_fill.gamma_multiply(alpha as f32);
            ui.add(copy_btn.fill(bg_color))
        } else {
            ui.add(copy_btn.frame(false))
        }
        .on_hover_text("Copy Color");

        if copy_res.clicked() {
            *copied_color = Some(current_color);
            *last_copied_id = Some(row_id);
            *last_copied_time = time;
            ui.ctx().request_repaint();
        }

        if let Some(c) = *copied_color {
            let paste_res = ui
                .add(egui::Button::new("📥").frame(false))
                .on_hover_text("Paste Color");
            if paste_res.clicked() {
                paste_color = Some(c);
            }
        }
    });
    paste_color
}
