//! Shared UI components for Secure Encrypted Notepad.

use egui;

pub struct Select<'a> {
    selected_text: String,
    min_width: f32,
    icon: Option<egui::Image<'a>>,
    button_builder: Option<Box<dyn FnOnce(egui::Button) -> egui::Button + 'a>>,
}

impl<'a> Select<'a> {
    pub fn new(selected_text: impl Into<String>) -> Self {
        Self {
            selected_text: selected_text.into(),
            min_width: 0.0,
            icon: None,
            button_builder: None,
        }
    }

    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }

    /// Sets an icon (e.g. a flag) to display inside the button, to the left of the text.
    pub fn with_icon(mut self, image: egui::Image<'a>) -> Self {
        self.icon = Some(image);
        self
    }

    /// Calculates min_width from the longest option text so the button never
    /// shrinks below that size, but also never stretches beyond it.
    pub fn with_width_hint(mut self, ui: &egui::Ui, longest_text: impl Into<String>) -> Self {
        let text_with_icon = format!("{} ⏷", longest_text.into());
        let font_id = egui::TextStyle::Button.resolve(ui.style());
        let galley =
            ui.painter()
                .layout(text_with_icon, font_id, egui::Color32::WHITE, f32::INFINITY);
        // Store only the text width. The button will add its own padding on top.
        // Add extra space for the icon if one is set.
        let icon_extra = if self.icon.is_some() { 24.0 } else { 0.0 };
        self.min_width = galley.rect.width().ceil() + icon_extra;
        self
    }

    pub fn button_builder(
        mut self,
        builder: impl FnOnce(egui::Button) -> egui::Button + 'a,
    ) -> Self {
        self.button_builder = Some(Box::new(builder));
        self
    }

    pub fn show_ui<R>(
        self,
        ui: &mut egui::Ui,
        add_contents: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let Self {
            selected_text,
            min_width,
            icon,
            button_builder,
        } = self;

        let button_text = format!("{} ⏷", selected_text);
        let mut button = if let Some(image) = icon {
            egui::Button::image_and_text(image, button_text).wrap_mode(egui::TextWrapMode::Truncate)
        } else {
            egui::Button::new(button_text).wrap_mode(egui::TextWrapMode::Truncate)
        };

        if min_width > 0.0 {
            button = button.min_size(egui::vec2(min_width, 0.0));
        }

        if let Some(builder) = button_builder {
            button = builder(button);
        }

        // ROOT CAUSE OF THE WIDTH BUG:
        // We check the direction of the main layout. If the parent already lays out items
        // horizontally (like right_to_left in the settings panel), we skip creating a new
        // nested horizontal layout. This prevents the panel from stretching to 100% width.
        let response = if ui.layout().main_dir().is_horizontal() {
            ui.add(button)
        } else {
            // If we are in a vertical layout (top_down), we use horizontal
            // to prevent unnatural stretching of the button.
            ui.horizontal(|ui| ui.add(button)).inner
        };

        let button_width = response.rect.width();
        let mut inner_res = None;

        egui::Popup::from_toggle_button_response(&response).show(|ui| {
            // If you want the dropdown list to be able to be narrower than the button
            // (perfectly fitted to the text inside), COMMENT OUT the line below.
            // Leave it if you want to keep the standard look (list at least as wide as the button).
            ui.set_min_width(button_width);

            egui::ScrollArea::vertical()
                .max_height(260.0)
                // KEY CHANGE: [true, true] allows egui to perfectly fit
                // the width (and height) to the content (the longest text).
                .auto_shrink([true, true])
                .show(ui, |ui| {
                    inner_res = Some(add_contents(ui));
                });
        });

        egui::InnerResponse {
            inner: inner_res,
            response,
        }
    }
}
