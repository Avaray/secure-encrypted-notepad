# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [x] **Theme System Refactoring & Expansion**: Refactor the color system and add more granular controls. This includes independent color states for buttons (idle, hover, active) for backgrounds, borders, and text, as well as similar refinements for text inputs and other interactive elements. Consider adding drop-shadow support for components.
- [x] **Theme Editor Polish**: Fix copy/paste buttons in the theme editor and replace their current icons with high-quality SVG equivalents. The UI should follow these rules:
    - Layout: `Paste` icon on the left, `Copy` icon on the right.
    - Visibility: Only the `Copy` icon should be visible when no color is currently in the "clipboard".
    - Active State: When a color is copied, the active `Copy` icon should have a continuous "pulse" animation (infinite fade-in/fade-out) or another engaging micro-animation to indicate the source of the copied color.
- [x] **Global Button Styling Consistency**: Ensure that all buttons throughout the application (in all panels and dialogs) strictly respect and apply the styles defined in the current theme.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **File Tree Icons Refresh**: Replace current folder and file icons in the file tree panel with high-quality SVG equivalents for better visual consistency.
- [ ] **Multicursor Support**: Implement multicursor functionality in the text editor.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [ ] **Documentation Deep Audit**: Perform a comprehensive documentation audit using a high-level thinking model.
- [ ] **Font Autodetection List Polish**: Refine the preferred font lists (UI and Editor) for smart detection, as the current lists are early versions that need better selection.
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [x] **Language Selection Flags**: Add mini SVG flags next to language names in the settings dropdown (SVG files will be provided soon).
- [x] **Add close buttons to panels**: Implement a close button for each side panel for easier navigation.
- [ ] **Translation Polish**: Review and refine translations throughout the application to ensure consistency and correctness across all supported languages.
- [ ] **Dynamic Panel Header Height**: Adjust the `render_panel_header` component to have a dynamic height that scales with the current font size. Currently, when font sizes are small, headers occupy too much vertical space relative to their content.


## 🐛 Bug Fixes (To Fix)
- [ ] **Settings Slider Crash**: Investigate and fix application crashes occurring when rapidly dragging value-based sliders (e.g., Font Size, Line Height, Transparency) in the settings and theme panels.
