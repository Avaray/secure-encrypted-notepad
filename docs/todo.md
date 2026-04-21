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
- [ ] **Action Bar Icon Ordering**: Reorder the first group of icons in the action bar to: New File, Save, Save As, Open, Open Directory, and Close.
- [ ] **Default Theme Refinement**: Improve and refine the color palettes for the default Light and Dark themes to enhance contrast and visual appeal.
- [ ] **Theme System Refactoring**: Expand the collection of built-in themes and refactor the theme engine to load definitions from a dedicated directory within the project. The default Light and Dark themes should also be migrated to this directory as standalone files while maintaining automatic system theme detection.
- [ ] **Scrollbar Customization (Themes)**: Enhance the theme system to allow customization of scrollbar colors for all states, including default (idle) and active (grabbed/held), as currently only the hover state is configurable.

---

## 🐛 Bug Fixes (To Fix)

- [x] **Enforce .sen Extension on Save**: When a user opens a regular text file (non-SEN) and attempts to save it, the application currently defaults to the original filename and extension. This leads to the file being encrypted while retaining its old extension (e.g. `.txt`). Upon reopening, the application treats it as plain text but displays encrypted content. The save dialog should default to the `.sen` extension for these cases.
- [ ] **Line Height Crash**: Fix application crash occurring when the user rapidly changes the "line height" value in settings.

