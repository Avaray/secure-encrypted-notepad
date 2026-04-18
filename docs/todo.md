# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Debug Panel Log Filter Layout**: Improve the layout of log type toggles in the debug panel. Currently, text wraps and expands vertically, causing layout instability.
- [ ] **Status Bar Activity Feedback**: Add more status bar messages about user actions (e.g., "File saved", "Search finished", etc.).
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **Action-Bar Icon Refresh**: Replace current temporary icons with custom-made icons.
- [ ] **Auto-Updater**: Implement an update check system (leveraging GitHub Releases) that is disabled by default. Consider supporting full automatic updates with binary replacement to streamline the update process for users.
- [x] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [x] **I18n Synchronization Script**: Create a `/scripts` directory containing helpful scripts. The first script (which can be written in Rust or TypeScript/Bun.sh) should compare `en.yml` against other translation files, identify missing keys, and automatically add them. The script should use an AI/LLM API to translate the missing sentences properly into the target languages.
- [x] **Custom 'Combo Box' Widget**: Create a custom 'combo box' (select) widget to replace all standard `egui::ComboBox` instances in the application. The standard ComboBox is problematic as its specifications prevent it from being easily centered vertically in layouts and it often ignores paddings and layout constraints. The custom version must respect paddings, theme colors, and align perfectly with other elements in horizontal rows.
- [x] **Optimize Compile Times for I18n**: The translation files (`.yml`) drastically increase build times (from a few seconds to tens of seconds). Resolve this by picking one of two solutions (the first is preferred):
    1. **Extract to a separate workspace crate**: Move the all `i18n!` macro usage and translation embeddings into a dedicated tiny crate (e.g., `sen-i18n`). Only this small crate will recompile on `.yml` changes, and the main app will just link it.
    2. **Runtime Loading**: Use `rust-i18n` with a runtime `load_path` mode. This pulls translations from the filesystem at runtime instead of embedding them at compile-time, completely bypassing recompilation on YAML edits.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Enforce .sen Extension on Save**: When a user opens a regular text file (non-SEN) and attempts to save it, the application currently defaults to the original filename and extension. This leads to the file being encrypted while retaining its old extension (e.g. `.txt`). Upon reopening, the application treats it as plain text but displays encrypted content. The save dialog should default to the `.sen` extension for these cases.
- [ ] **Atomic Safe-Save Protection Against Corruption**: Ensure file saving is resilient to interrupted writes (e.g., system crash, BSOD, forced app termination). Prevent any scenario where a partially/corruptly written file replaces the original and becomes impossible to open correctly.
