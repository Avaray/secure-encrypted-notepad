mod crypto;
mod settings;
mod history;
mod app;

use app::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([1000.0, 700.0])
            .with_icon(
                // Jeśli masz ikonę w formacie PNG, użyj:
                eframe::icon_data::from_png_bytes(include_bytes!("../LogosAurora.png"))
                    .unwrap_or_default(), // Jeśli nie masz ikony, użyj domyślnej dla Windows:
                                          // eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()
            ),
        ..Default::default()
    };
    
    eframe::run_native(
        "SED v3.0 - Secure Encrypted Document Editor with Version Control",
        options,
        Box::new(|cc| Ok(Box::new(EditorApp::new(cc)))),
    )
}
