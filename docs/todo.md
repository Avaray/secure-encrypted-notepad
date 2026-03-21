# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [x] **Internationalization (I18n)**: Add support for multiple application languages. (Implemented with EN, PL, DE support).
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Theme System Refactoring & Expansion**: Refactor the color system and add more granular controls. This includes independent color states for buttons (idle, hover, active) for backgrounds, borders, and text, as well as similar refinements for text inputs and other interactive elements. Consider adding drop-shadow support for components.
- [ ] **Theme Editor Polish**: Fix copy/paste buttons in the theme editor and replace their current icons with high-quality SVG equivalents. The UI should follow these rules:
    - Layout: `Paste` icon on the left, `Copy` icon on the right.
    - Visibility: Only the `Copy` icon should be visible when no color is currently in the "clipboard".
    - Active State: When a color is copied, the active `Copy` icon should have a continuous "pulse" animation (infinite fade-in/fade-out) or another engaging micro-animation to indicate the source of the copied color.
- [ ] **Global Button Styling Consistency**: Ensure that all buttons throughout the application (in all panels and dialogs) strictly respect and apply the styles defined in the current theme.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **File Tree Icons Refresh**: Replace current folder and file icons in the file tree panel with high-quality SVG equivalents for better visual consistency.
- [ ] **Multicursor Support**: Implement multicursor functionality in the text editor.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [x] **System File Association**: Add an option in settings to associate `.sen` files with the application. Implementation order: Windows first, followed by Linux, and lastly macOS.
- [ ] **Documentation Deep Audit**: Perform a comprehensive documentation audit using a high-level thinking model.
- [x] **About Panel (F1)**: Add a full-screen panel displaying program information, repository and bug-report links, author information, and financial support links.
- [ ] **Font Autodetection List Polish**: Refine the preferred font lists (UI and Editor) for smart detection, as the current lists are early versions that need better selection.
- [x] **Single Instance Mode**: Add a "Single instance" option in settings to prevent multiple SEN processes from running simultaneously. When enabled, opening a `.sen` file from the OS file browser while SEN is already running should forward the file path to the existing instance via IPC (e.g., named pipe or socket). The existing instance must handle edge cases gracefully: if the user has unsaved changes, prompt to save before loading the new file; if no keyfile is loaded, prompt for one; if the wrong keyfile is active, show the retry dialog. The overall UX must feel seamless and professional regardless of the scenario.
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [ ] **Automatic System Language Detection**: Implement automatic detection of the system language on startup (all platforms including Android).
- [ ] **Language Selection Flags**: Add mini SVG flags next to language names in the settings dropdown (SVG files will be provided soon).
- [ ] **Translation Polish**: Review and refine translations throughout the application to ensure consistency and correctness across all supported languages.

## 🐛 Bug Fixes (To Fix)
- [ ] **Settings Slider Crash**: Investigate and fix application crashes occurring when rapidly dragging value-based sliders (e.g., Font Size, Line Height, Transparency) in the settings and theme panels.

## 🔄 Batch Converter Improvements
- [x] **Layout Refinement**: Change the batch converter layout, specifically the top info section and close button. Move the main action button to the bottom of the left column.
- [x] **Toolbar Button Integration**: Move the batch converter toolbar button and ensure it highlights when active (like other panels).
- [x] **Left Column Blocks**: Refine headings and button groups in the left panel to behave as cohesive "blocks" with proper wrapping, ensuring a cleaner layout regardless of panel width.
