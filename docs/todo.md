# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [x] **Auto-Backup on Save**: Add option to automatically copy encrypted .sen files to multiple backup folders on save.
- [ ] **Internal Auto-save Integration**: Change auto-save behavior to store content inside the original `.sen` file instead of a separate `.autosave.sen`. When opening a `.sen` file where the internal auto-save is newer than the latest history entry, prompt the user to restore the auto-saved version.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

*No known bugs at the moment.*

---

## 🛡️ Security & Privacy (Planned)

- [ ] **Screen Capture Protection**: Add an option to block application screenshots (OS-level protection where possible) with a toggle in settings.
    - *Recommendation: Use "Best-Effort" approach. Windows (SetWindowDisplayAffinity) is highly effective. macOS (NSWindowSharingNone) is restricted/alert-heavy on 15+. Linux Wayland is secure by default (isolation); X11 lacks native support.*

---

> [!NOTE]
> This list is dynamic and will be updated as the project evolves. Specific details for these items will be provided by the project maintainer.
