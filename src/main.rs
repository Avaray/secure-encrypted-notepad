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
mod ui_dialogs;
mod ui_editor;
mod ui_panels;
mod ui_search;
mod ui_toolbar;

use app::EditorApp;

fn main() -> Result<(), eframe::Error> {
    let settings = crate::settings::Settings::load();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_maximized(settings.start_maximized),
        ..Default::default()
    };

    eframe::run_native(
        "Secure Encrypted Document Editor",
        options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}
