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
- [ ] **Documentation Deep Audit**: Perform a comprehensive documentation audit using a high-level thinking model.

## 🐛 Bug Fixes (To Fix)

**none**

### 🔄 Batch Converter Improvements
- [ ] **Layout Refinement**: Change the batch converter layout, specifically the top info section and close button. Move the main action button to the bottom of the left column.
- [ ] **Toolbar Button Integration**: Move the batch converter toolbar button and ensure it highlights when active (like other panels).
- [ ] **Quick Key Import**: Add a button to import the currently loaded application key into the batch converter's key fields. Only show this when a key is loaded in the main editor.
- [ ] **Extension Persistence**: Save and restore the last used file extension in the application settings.

---

*Last Updated: 2026-03-18*
