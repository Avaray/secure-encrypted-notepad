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
- [x] **Scrollbar Padding in Panels**: In all side panels (Settings, History, Debug, etc.), ensure there is a right-side margin/padding to account for the vertical scrollbar. This prevents the scrollbar from overlapping buttons, text, or other UI elements when it appears.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [x] **Batch Keyfile Rotation**: Add a "Rotate Keyfile" mode to the Batch Converter. A mode switch (Encrypt / Decrypt / Rotate) should appear as a new field above the keyfile selection. When "Rotate" is selected, two keyfile pickers should be shown (old keyfile and new keyfile) instead of one.
- [x] **Batch Converter Dynamic Action Button**: Replace the two bottom action buttons (Encrypt All / Decrypt All) with a single dynamic button whose label changes based on the selected mode (e.g. "🔒 Encrypt All", "🔓 Decrypt All", or "🔄 Rotate All").
- [ ] **Batch Converter Window Resizing**: Add the ability to expand the Batch Converter window to fill the entire application window (e.g., via a toggle button next to the close button).
- [x] **Batch Converter List Layout Fix**: Fix the file list layout to prevent long filenames from pushing the "remove" (❌) buttons off-screen. Implement text truncation with ellipsis for filenames.
- [ ] **System File Association**: Add an option in settings to associate `.sen` files with the application. Implementation order: Windows first, followed by Linux, and lastly macOS.
- [ ] **Theme Editor Interaction Polish**: Disable text selection for color labels in the Theme Editor to allow scrolling by clicking and dragging on the labels (consistent with background interaction).
- [ ] **File Tree Real-time Monitoring**: Implement a file system watcher (e.g., using the `notify` crate, which is already in dependencies) for the file tree panel. This should automatically update the list of files when they are added, removed, or renamed in the currently open directory. Note: In "expandable" mode, this requires recursive watching of multiple directories simultaneously.
- [ ] **File Tree Status Indicator Alignment**: Refine the position of the color dot (access indicator) in the file tree (expandable mode). Add a small left margin and reduce the right margin to better align it with the filename. Consider replacing the dot with a high-quality SVG icon in the future.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [ ] **Batch Converter Decryption Logic**: Fix the `decrypt` mode. Currently, it decrypts the entire file, but it should only extract and decrypt the most recent history entry.
