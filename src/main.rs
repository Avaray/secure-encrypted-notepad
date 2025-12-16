mod app;
mod crypto;
mod fonts;
mod history;
mod icons;
mod settings;
mod theme;

use app::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_min_inner_size([1000.0, 700.0])
            .with_fullscreen(false)
            .with_resizable(true)
            .with_decorations(true),
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        vsync: true,
        multisampling: 0,
        ..Default::default()
    };

    eframe::run_native(
        "SED - Secure Encrypted Document Editor",
        options,
        Box::new(|cc| {
            // Configure UI font (system font or default proportional)
            cc.egui_ctx.style_mut(|style| {
                use egui::{FontFamily, FontId, TextStyle};
                style.text_styles = [
                    (
                        TextStyle::Heading,
                        FontId::new(18.0, FontFamily::Proportional),
                    ),
                    (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
                    (
                        TextStyle::Monospace,
                        FontId::new(14.0, FontFamily::Monospace),
                    ),
                    (
                        TextStyle::Button,
                        FontId::new(14.0, FontFamily::Proportional),
                    ),
                    (
                        TextStyle::Small,
                        FontId::new(10.0, FontFamily::Proportional),
                    ),
                ]
                .into();
            });

            Ok(Box::new(EditorApp::new(cc)))
        }),
    )
}
