#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

// Initialize i18n with locale files from ./locales, fallback to English
i18n!("locales", fallback = "en");

/// Custom debug logging macro that only prints to console in debug builds.
/// This prevents sensitive data leakage in release versions.
#[allow(unused_macros)]
macro_rules! sen_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(debug_assertions)]
            eprintln!("[SEN] {}", format!($($arg)*));

            #[cfg(not(debug_assertions))]
            if false {
                let _ = format_args!($($arg)*);
            }
        }
    };
}

pub(crate) use sen_debug;

mod crypto;
mod fonts;
mod history;
mod icons;
mod settings;
mod single_instance;
mod theme;

// Module declarations for app
mod app;
mod app_actions;
mod app_helpers;
mod app_state;
mod config_crypto;
mod ui_batch;
mod ui_dialogs;
mod ui_editor;
mod ui_panels;
mod ui_search;
mod ui_toolbar;

use app::EditorApp;

fn main() -> Result<(), eframe::Error> {
    let start_time = std::time::Instant::now();
    let settings = crate::settings::Settings::load();

    // Set the UI language from saved settings
    rust_i18n::set_locale(&settings.language);

    let mut args = std::env::args();
    let _cmd = args.next(); // Skip executable path
    let file_to_open = args.next().map(std::path::PathBuf::from);

    // Single instance check — must happen before creating the window
    let ipc_queue = if settings.single_instance {
        match single_instance::try_lock(&file_to_open) {
            single_instance::LockResult::Acquired(queue) => Some(queue),
            single_instance::LockResult::AlreadyRunning => {
                // File path was forwarded to existing instance; exit silently.
                std::process::exit(0);
            }
        }
    } else {
        None
    };

    // Always set the last known non-maximized size as fallback geometry.
    // Do NOT use with_maximized(true) here — it has a race condition on Windows 10/11
    // where the window doesn't reliably start maximized. Instead, we use the
    // "first-frame ViewportCommand" trick in EditorApp::update().
    // Load application icon for window/taskbar
    let app_icon = crate::icons::Icons::load_app_icon();

    // `with_visible(false)` is reliable on macOS and Linux (X11 + Wayland).
    // On Windows, DWM can briefly render the window frame before WS_VISIBLE is
    // cleared, causing a one-frame flicker.  The workaround is to create the
    // window off-screen at (-32000, -32000) — the Win32 sentinel value for a
    // minimized window — so any DWM flash is outside the visible desktop.
    // The saved position is restored in EditorApp::update() just before Visible(true).
    //
    // We do NOT use this trick on macOS/Linux:
    //   • macOS clamps window positions to the visible screen area.
    //   • Wayland forbids applications from setting their own position entirely.
    //   • with_visible(false) already works reliably on both platforms.
    #[cfg(target_os = "windows")]
    let initial_position = eframe::egui::pos2(-32000.0, -32000.0);

    #[cfg(not(target_os = "windows"))]
    let initial_position = if settings.window_pos_x >= 0.0 && settings.window_pos_y >= 0.0 {
        eframe::egui::pos2(settings.window_pos_x, settings.window_pos_y)
    } else {
        eframe::egui::pos2(100.0, 100.0)
    };

    let viewport_builder = eframe::egui::ViewportBuilder::default()
        .with_inner_size([settings.window_width, settings.window_height])
        .with_icon(app_icon)
        .with_min_inner_size([800.0, 600.0])
        .with_visible(false)
        .with_position(initial_position);

    let options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        "Secure Encrypted Notepad",
        options,
        Box::new(move |cc| {
            Ok(Box::new(EditorApp::new(
                cc,
                settings,
                file_to_open,
                ipc_queue,
                start_time,
            )))
        }),
    )
}
