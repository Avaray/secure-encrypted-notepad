# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages. Consider using the `egui-i18n` crate for implementation.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **File Tree Icons Refresh**: Replace current folder and file icons in the file tree panel with high-quality SVG equivalents for better visual consistency.
- [ ] **Multicursor Support**: Implement multicursor functionality in the text editor.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [ ] **System File Association**: Add an option in settings to associate `.sen` files with the application. Implementation order: Windows first, followed by Linux, and lastly macOS.
- [ ] **File Tree Real-time Monitoring**: Implement a file system watcher (e.g., using the `notify` crate, which is already in dependencies) for the file tree panel. This should automatically update the list of files when they are added, removed, or renamed in the currently open directory. Note: In "expandable" mode, this requires recursive watching of multiple directories simultaneously.
- [x] **Documentation Audit**: Verify the correctness and completeness of the instructions in `docs/development.md` and `docs/encryption_architecture.md`.
- [x] **Batch Converter Spinner**: Show a spinner (e.g., using `egui::Spinner`) on the bottom action button while conversion (encrypt, decrypt, or rotate) is in progress to provide visual feedback.
- [x] **File Tree Background Scanner Indicator**: Add a spinner at the end of each directory in the file tree while background scanning and verification of `.sen` files is active in that directory.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [x] **Batch Converter Decryption Logic**: Fix the `decrypt` mode. Currently, it decrypts the entire file, but it should only extract and decrypt the most recent history entry.
