# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **File Tree Icons Refresh**: Replace current folder and file icons in the file tree panel with high-quality SVG equivalents for better visual consistency.

- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [ ] **Documentation Deep Audit**: Perform a comprehensive documentation audit using a high-level thinking model.
- [ ] **Font Autodetection List Polish**: Refine the preferred font lists (UI and Editor) for smart detection, as the current lists are early versions that need better selection.
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [ ] **Translation Polish**: Review and refine translations throughout the application to ensure consistency and correctness across all supported languages.


## 🐛 Bug Fixes (To Fix)

- [ ] **Theme Editor UI Polish (Copy/Paste & Color Pickers)**: Ensure ideal alignment of grid elements in the theme editor.
    - Fix sizes of copy/paste buttons (should be framed, squared, matching `interact_size.y` height).
    - Replace the native `ui.color_edit_button_srgb` with custom buttons (e.g., `ui.add(egui::Button::image(...).fill(color))`). This allows forcing a square `min_size` matching the input height, eliminating the native selector's rigid dimensions.
    - Use `egui::show_tooltip_for_rect` or `egui::Popup` with `egui::color_picker::color_picker_color32` to display the color adjustment interface only after clicking the custom button.
- [ ] **Settings Slider Crash**: Investigate and fix application crashes occurring when rapidly dragging value-based sliders (e.g., Font Size, Line Height, Transparency) in the settings and theme panels.
