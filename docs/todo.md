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
- [ ] **About Panel Sponsor Buttons Layout**: Improve the layout of sponsor buttons in the "About" panel. Ensure that when the buttons wrap to a new line, they remain horizontally centered.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Line Height Crash**: Fix application crashes occurring when users rapidly change the 'line height' value in settings.
- [x] **Theme Editor Row Gaps**: Audit vertical spacing between elements in the theme editor. Ensure consistent gaps/margins between all rows (e.g., between "Icon default tint" and "hover tint", and between "line numbers" and "cursor color").
