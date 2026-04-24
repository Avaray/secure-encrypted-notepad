# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Debug Panel Log Filter Layout (V2)**: Redesign the arrangement of action buttons and log filter toggles to be more compact and visually organized, ensuring they adapt well to different panel widths without wasting space.
- [ ] **File Tree Panel Icons Refresh**: Replace current file tree icons with clearer, more professional versions that remain legible at small sizes. Also, ensure all files use updated icons instead of the legacy "status dot" still present in some cases.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **Action-Bar Icon Refresh**: Replace current temporary icons with custom-made icons.
- [ ] **Auto-Updater**: Implement an update check system (leveraging GitHub Releases) that is disabled by default. Consider supporting full automatic updates with binary replacement to streamline the update process for users.
- [ ] **Default Theme Refinement**: Improve and refine the color palettes for the default Light and Dark themes to enhance contrast and visual appeal.
- [ ] **Theme System Refactoring**: Expand the collection of built-in themes and refactor the theme engine to load definitions from a dedicated directory within the project. The default Light and Dark themes should also be migrated to this directory as standalone files while maintaining automatic system theme detection.
- [ ] **Custom File Title (Alias)**: Add the ability to assign a custom title (alias) to a SEN file. When set, this title should take priority over the actual filename in all UI locations where the file name is displayed. This allows users to further disguise files on the filesystem (e.g., naming a file `system_logs.tmp` while labeling it "Employee List" in the UI) to enhance privacy. Support should be included for both standard and stealth modes.
- [ ] **I18n Refinement & Sync (v0.9.0+)**: Proofread and refine all English translations in `en.yml`, then synchronize and re-translate all other supported languages to ensure consistency across the application.
- [ ] **File Tree Status Icon Defaults**: Ensure that `tree_file_stealth` and `tree_file_unlocked` status icons in the file tree panel default to the "success" color if their specific colors are not defined in the current theme.
- [ ] **Search Panel Layout Alignment**: In the search panel, set the right margin (to the right of the close button) to the same value used by panel headers to ensure visual consistency across the UI.
- [ ] **High-Quality Icon Audit**: Audit all application icons (especially in buttons and the 'About Panel') to ensure they are high-quality vector assets (SVG) rather than low-resolution PNGs. All icons should remain crisp and professional when scaled or viewed on high-DPI screens.
- [ ] **Theme Editor Label Scaling**: In the Theme Editor, reduce the font size for section headers (e.g., Typography, Widgets) using a smaller font style (e.g., `egui::TextStyle::Small`) to create a better visual hierarchy and a more compact layout.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Line Height Crash**: Fix application crashes occurring when users rapidly change the 'line height' value in settings.
