mod app;
mod crypto;
mod history;
mod settings;

use app::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // ZAWSZE OTWIERAJ ZMAKSYMALIZOWANE NA WSZYSTKICH PLATFORMACH
            .with_maximized(true)
            // Zachowujemy minimalny rozmiar dla użyteczności
            .with_min_inner_size([1000.0, 700.0])
            // Poprawiona obsługa ikony - bezpieczne ładowanie
            .with_icon({
                // Bezpieczne ładowanie ikony - jeśli plik nie istnieje, użyj domyślnej
                match eframe::icon_data::from_png_bytes(include_bytes!("../LogosCockpit.png")) {
                    Ok(icon) => icon,
                    Err(_) => eframe::icon_data::from_png_bytes(&[]).unwrap_or_default(),
                }
            }),
        // Usuwamy linię z renderer::Auto - jest niepotrzebna i niekompatybilna
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        ..Default::default()
    };

    eframe::run_native(
        "SED v3.0 - Secure Encrypted Document Editor with Version Control",
        options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}
