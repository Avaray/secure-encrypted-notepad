# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [x] **Default Themes Refinement**: Refine the built-in Light and Dark themes for better aesthetics and readability. The Dark theme should have darker backgrounds (it currently feels too gray), while the Light theme should move away from stark white towards softer, slightly grayish tones.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [ ] **Theme Editor Panel Sizing**: Ensure the Theme Editor panel cannot be resized horizontally beyond its longest content. It must dynamically adapt to changes in UI font size or button padding to maintain perfect constraints.
- [x] **Exit Confirmation Logic**: If the user has unsaved changes but the document is completely empty, skip the "unsaved changes" confirmation dialog when exiting the application.
- [ ] **Windows Start Menu Integration**: Add an "Add to Start Menu" button (Windows-specific) in settings. It would generate the necessary manifest files and icons to allow pinning a large, high-quality tile for SEN to the Start Menu, bypassing the current limitations of portable apps.
- [ ] **Draggable Dialogs**: Make custom dialog windows (like "Reset Settings") draggable by clicking and dragging their title bar area.
- [ ] **Auto-Updater**: Implement an update check system (leveraging GitHub Releases) that is disabled by default. Consider supporting full automatic updates with binary replacement to streamline the update process for users.
- [ ] **Debug Panel Log Filter Layout**: Improve the layout of log type toggles in the debug panel. Currently, text wraps and expands vertically, causing layout instability.
- [ ] **Status Bar Activity Feedback**: Add more status bar messages about user actions (e.g., "File saved", "Search finished", etc.).
- [ ] **Hide Status Bar Option**: Add a setting to completely hide the application's status bar.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Settings Slider Crash**: Investigate and fix application crashes occurring when rapidly dragging value-based sliders (e.g., Font Size, Line Height, Transparency) in the settings and theme panels.
- [ ] **Minimum Panel Widths**: Implement minimum width constraints for all side panels to prevent them from overlapping or becoming completely hidden. The user should always be able to easily find and grab the panel edge handles without confusion.
- [x] **Font Detection Audit**: Investigating why preferred fonts from `PREFERRED_UI_FONTS` are not being correctly auto-detected and applied during first launch or settings reset on Windows, even when those fonts are installed in the system. Current behavior defaults to system fonts instead of the high-priority list.
- [ ] **Color Picker Interaction Bug**: Fix a critical issue in the Theme Editor where the color picker popup's internal sliders and selection fields (hue/shade/saturation) become unresponsive or 'stuck' during use, making it impossible to adjust colors correctly.
- [ ] **Vertical Alignment Refinement**: Fix UI layouts for better vertical centering of heterogeneous single-line elements. Comboboxes still do not respect proper vertical alignment in some contexts.
