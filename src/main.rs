#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod crypto;
mod fonts;
mod history;
mod icons;
mod settings;
mod theme;

// Deklaracje modułów dla app
mod app;
mod app_actions;
mod app_helpers;
mod app_state;
mod config_crypto;
mod ui_dialogs;
mod ui_editor;
mod ui_panels;
mod ui_search;
mod ui_toolbar;
mod ui_batch;

use app::EditorApp;

fn main() -> Result<(), eframe::Error> {
    let settings = crate::settings::Settings::load();

    // Always set the last known non-maximized size as fallback geometry.
    // Do NOT use with_maximized(true) here — it has a race condition on Windows 10/11
    // where the window doesn't reliably start maximized. Instead, we use the
    // "first-frame ViewportCommand" trick in EditorApp::update().
    // Load application icon for window/taskbar
    let app_icon = crate::icons::Icons::load_app_icon();

    let mut viewport_builder = eframe::egui::ViewportBuilder::default()
        .with_inner_size([settings.window_width, settings.window_height])
        .with_icon(app_icon)
        .with_min_inner_size([800.0, 600.0]);

    // Always apply saved position (even when start_maximized is true) so the window
    // has sensible geometry if the user later unmaximizes.
    if settings.window_pos_x >= 0.0 && settings.window_pos_y >= 0.0 {
        viewport_builder = viewport_builder.with_position(eframe::egui::pos2(
            settings.window_pos_x,
            settings.window_pos_y,
        ));
    }

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Secure Encrypted Notepad",
        options,
        Box::new(move |cc| Ok(Box::new(EditorApp::new(cc, settings)))),
    )
}
