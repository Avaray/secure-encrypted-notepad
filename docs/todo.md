# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **Auto-Backup on Save**: Add option to automatically copy encrypted .sen files to multiple backup folders on save.
- [x] **Reset All Settings**: Implement a reset functionality with a confirmation dialog requiring a slider to be moved to the right and clicking OK.
- [x] **Auto-save on Focus Loss**: Automatically save the current content to a `.autosave.sen` file when the application loses focus.
- [x] **Confirm Keyfile Clearing**: Add a confirmation dialog before clearing the global keyfile in the Settings panel.
- [ ] **Internal Auto-save Integration**: Change auto-save behavior to store content inside the original `.sen` file instead of a separate `.autosave.sen`. When opening a `.sen` file where the internal auto-save is newer than the latest history entry, prompt the user to restore the auto-saved version.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [ ] **Toolbar Icon Overlap**: Prevent icons on the left and right sides of the top menu from overlapping when the window is narrow or there are too many icons.

---

## 🛡️ Security & Privacy (Planned)

- [ ] **Screen Capture Protection**: Add an option to block application screenshots (OS-level protection where possible) with a toggle in settings.

---

> [!NOTE]
> This list is dynamic and will be updated as the project evolves. Specific details for these items will be provided by the project maintainer.
