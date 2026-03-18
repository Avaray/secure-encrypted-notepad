# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **File Tree Icons Refresh**: Replace current folder and file icons in the file tree panel with high-quality SVG equivalents for better visual consistency.
- [ ] **Multicursor Support**: Implement multicursor functionality in the text editor.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [x] **Batch Converter Window Resizing**: Add the ability to expand the Batch Converter window to fill the entire application window (e.g., via a toggle button next to the close button).
- [ ] **System File Association**: Add an option in settings to associate `.sen` files with the application. Implementation order: Windows first, followed by Linux, and lastly macOS.
- [ ] **File Tree Real-time Monitoring**: Implement a file system watcher (e.g., using the `notify` crate, which is already in dependencies) for the file tree panel. This should automatically update the list of files when they are added, removed, or renamed in the currently open directory. Note: In "expandable" mode, this requires recursive watching of multiple directories simultaneously.
- [ ] **Documentation Audit**: Verify the correctness and completeness of the instructions in `docs/development.md` and `docs/encryption_architecture.md`.
- [x] **Hide Undecryptable Files Toggle**: Add a toggle in the File Tree settings to show only files that can be decrypted with the currently loaded keyfile. Since file access is already checked in the background, this should be straightforward. Ensure the file tree is refreshed automatically whenever the keyfile changes.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [ ] **Batch Converter Decryption Logic**: Fix the `decrypt` mode. Currently, it decrypts the entire file, but it should only extract and decrypt the most recent history entry.
