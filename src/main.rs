// #![windows_subsystem = "windows"]

mod app;
mod crypto;
mod history;
mod settings;

use app::EditorApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_maximized(true)
            .with_min_inner_size([1000.0, 700.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../LogosCockpit.png"))
                    .unwrap_or_else(|_| {
                        eprintln!("Failed to load icon");
                        egui::IconData::default()
                    }),
            )
            .with_fullscreen(false)
            .with_resizable(true)
            .with_decorations(true),
        hardware_acceleration: eframe::HardwareAcceleration::Preferred,
        vsync: true,
        multisampling: 0,
        ..Default::default()
    };

    eframe::run_native(
        "SED - Secure Notepad",
        options,
        Box::new(|cc| {
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
