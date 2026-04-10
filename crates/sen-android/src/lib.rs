pub mod android_fs;
pub mod icons;
use eframe::egui;
use sen_core::settings::Settings;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;

#[cfg(target_os = "android")]
#[no_mangle]
#[cfg(target_os = "android")]
fn android_main(app: AndroidApp) {
    let app_clone = app.clone();
    let mut options = eframe::NativeOptions::default();
    options.android_app = Some(app);

    let _ = eframe::run_native(
        "SEN Android",
        options,
        Box::new(|cc| Ok(Box::new(SenAndroidApp::new(cc, app_clone)))),
    );
}

struct SenAndroidApp {
    settings: Settings,
    icons: Option<icons::Icons>,
    text: String,
    show_settings: bool,
    keyfile_loaded: bool,
    
    #[cfg(target_os = "android")]
    android_app: AndroidApp,
}

impl SenAndroidApp {
    #[cfg(target_os = "android")]
    pub fn new(cc: &eframe::CreationContext<'_>, android_app: AndroidApp) -> Self {
        let settings = Settings::default();
        let icons = Some(icons::Icons::load(&cc.egui_ctx));

        Self {
            settings,
            icons,
            text: String::new(),
            show_settings: false,
            keyfile_loaded: false,
            android_app,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Fallback for logic consistency
        Self {
            settings: Settings::default(),
            icons: Some(icons::Icons::load(&cc.egui_ctx)),
            text: String::new(),
            show_settings: false,
            keyfile_loaded: false,
        }
    }
    
    // Abstract logic that will call into AndroidFs / JNI in the future
    #[cfg(target_os = "android")]
    fn trigger_open_file(&mut self) {
        println!("Triggered open file from Rust!");
        // JNI fully implemented later
    }

    #[cfg(not(target_os = "android"))]
    fn trigger_open_file(&mut self) {}

    #[cfg(target_os = "android")]
    fn trigger_save_file(&mut self) {
        println!("Triggered save file from Rust!");
        // JNI fully implemented later
    }

    #[cfg(not(target_os = "android"))]
    fn trigger_save_file(&mut self) {}
}

impl eframe::App for SenAndroidApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Force continuous repaint on Android to prevent idle freezing
        ctx.request_repaint();

        // Add padding for mobile touch targets
        let mut style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(12.0, 12.0);
        ctx.set_style(style);

        let mut clicked_settings = false;
        let mut clicked_open = false;
        let mut clicked_save = false;

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            // Provide massive padding to clear Android Status Bar / Notch
            ui.add_space(48.0);
            ui.horizontal(|ui| {
                if let Some(icons) = &self.icons {
                    let btn_size = egui::vec2(28.0, 28.0);
                    
                    if ui.add(egui::ImageButton::new(egui::Image::new(&icons.settings).fit_to_exact_size(btn_size))).clicked() {
                        clicked_settings = true;
                    }
                    ui.separator();
                    if ui.add(egui::ImageButton::new(egui::Image::new(&icons.open).fit_to_exact_size(btn_size))).clicked() {
                        clicked_open = true;
                    }
                    if ui.add(egui::ImageButton::new(egui::Image::new(&icons.save).fit_to_exact_size(btn_size))).clicked() {
                        clicked_save = true;
                    }
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.heading("SEN");
                });
            });
            ui.add_space(4.0);
        });

        if clicked_settings { self.show_settings = !self.show_settings; }
        if clicked_open { self.trigger_open_file(); }
        if clicked_save { self.trigger_save_file(); }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.show_settings {
                ui.heading("Settings");
                ui.separator();
                ui.label("Theme and settings customization will be ported here.");
                // We will link to the core theme generation here
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.text)
                            .frame(false)
                            .desired_width(f32::INFINITY)
                            .hint_text("Tap here to type..."),
                    );
                });
            }
        });
    }
}
