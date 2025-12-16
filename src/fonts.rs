use font_kit::source::SystemSource;
use std::collections::BTreeSet;

/// Get list of available system fonts
pub fn get_system_fonts() -> Vec<String> {
    let mut fonts = BTreeSet::new();

    // Add default egui fonts
    fonts.insert("Proportional (Default)".to_string());
    fonts.insert("Monospace (Default)".to_string());

    // Scan system fonts
    let source = SystemSource::new();
    if let Ok(families) = source.all_families() {
        for family in families {
            fonts.insert(family);
        }
    }

    fonts.into_iter().collect()
}

/// Load custom font from system
pub fn load_font_data(font_name: &str) -> Option<Vec<u8>> {
    // Skip default fonts
    if font_name.contains("(Default)") {
        return None;
    }

    let source = SystemSource::new();

    // Try to find the font family
    if let Ok(handle) = source.select_best_match(
        &[font_kit::family_name::FamilyName::Title(
            font_name.to_string(),
        )],
        &font_kit::properties::Properties::new(),
    ) {
        // Load font data
        if let Ok(font) = handle.load() {
            if let Some(data) = font.copy_font_data() {
                // Zmień Ok na Some
                return Some(data.to_vec());
            }
        }
    }

    None
}
