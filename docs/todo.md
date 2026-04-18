# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Debug Panel Log Filter Layout**: Improve the layout of log type toggles in the debug panel. Currently, text wraps and expands vertically, causing layout instability.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **Action-Bar Icon Refresh**: Replace current temporary icons with custom-made icons.
- [ ] **Auto-Updater**: Implement an update check system (leveraging GitHub Releases) that is disabled by default. Consider supporting full automatic updates with binary replacement to streamline the update process for users.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Enforce .sen Extension on Save**: When a user opens a regular text file (non-SEN) and attempts to save it, the application currently defaults to the original filename and extension. This leads to the file being encrypted while retaining its old extension (e.g. `.txt`). Upon reopening, the application treats it as plain text but displays encrypted content. The save dialog should default to the `.sen` extension for these cases.
- [ ] **Atomic Safe-Save Protection Against Corruption**: Ensure file saving is resilient to interrupted writes (e.g., system crash, BSOD, forced app termination). Prevent any scenario where a partially/corruptly written file replaces the original and becomes impossible to open correctly.
