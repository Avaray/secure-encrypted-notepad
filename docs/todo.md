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
- [ ] **I18n Synchronization Script**: Create a `/scripts` directory containing helpful scripts. The first script (which can be written in Rust or TypeScript/Bun.sh) should compare `en.yml` against other translation files, identify missing keys, and automatically add them. The script should use an AI/LLM API to translate the missing sentences properly into the target languages.
- [x] **Optimize Compile Times for I18n**: The translation files (`.yml`) drastically increase build times (from a few seconds to tens of seconds). Resolve this by picking one of two solutions (the first is preferred):
    1. **Extract to a separate workspace crate**: Move the all `i18n!` macro usage and translation embeddings into a dedicated tiny crate (e.g., `sen-i18n`). Only this small crate will recompile on `.yml` changes, and the main app will just link it.
    2. **Runtime Loading**: Use `rust-i18n` with a runtime `load_path` mode. This pulls translations from the filesystem at runtime instead of embedding them at compile-time, completely bypassing recompilation on YAML edits.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Vertical Alignment Refinement**: Fix UI layouts for better vertical centering of heterogeneous single-line elements. Comboboxes still do not respect proper vertical alignment.
