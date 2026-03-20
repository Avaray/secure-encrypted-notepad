import re

with open('src/ui_panels.rs', 'r', encoding='utf-8') as f:
    text = f.read()

# 1. Update the signature of edit_optional_color
text = text.replace(
    'let mut edit_optional_color = |label: &str, field: &mut Option<[u8; 3]>, default: [u8; 3], id_str: &str, ui: &mut egui::Ui| -> bool {',
    'let mut edit_optional_color = |label: &str, field: &mut Option<[u8; 3]>, default: [u8; 3], id_str: &str, copied_color: &mut Option<[u8; 3]>, last_copied_id: &mut Option<egui::Id>, last_copied_time: &mut f64, ui: &mut egui::Ui| -> bool {'
)

def replacer(match):
    return match.group(1) + ', copied_color, last_copied_id, last_copied_time, ui)'

text = re.sub(r'(edit_optional_color\([^)]+?)(,\s*ui\))', replacer, text)

with open('src/ui_panels.rs', 'w', encoding='utf-8') as f:
    f.write(text)
print('Success 2')
