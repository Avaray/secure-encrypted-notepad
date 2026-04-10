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
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Vertical Alignment Refinement**: Fix UI layouts for better vertical centering of heterogeneous single-line elements. Comboboxes still do not respect proper vertical alignment.
