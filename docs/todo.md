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

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Enforce .sen Extension on Save**: When a user opens a regular text file (non-SEN) and attempts to save it, the application currently defaults to the original filename and extension. This leads to the file being encrypted while retaining its old extension (e.g. `.txt`). Upon reopening, the application treats it as plain text but displays encrypted content. The save dialog should default to the `.sen` extension for these cases.
