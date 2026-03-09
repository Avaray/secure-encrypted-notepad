# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [x] **Editor Comfort (Phase 2)**: Increase the right margin in the main editor further to ensure text never touches the edge, especially when typing spaces (ensuring space at the end of lines).
- [x] **File Tree Key Icons & Access Status**: Replace document icons with color-coded key icons in the file tree to indicate if the current keyfile can decrypt the .sen files (asynchronous/background check for performance).
- [x] **Theme Editor Hover Colors**: Add ability to customize the background color of menu buttons when hovered.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [x] **Whitespace Symbols Color**: Add ability to customize the color of whitespace symbols (spaces, tabs, returns) within the theme editor.
- [x] **History Capacity**: Increase the maximum allowed file history entries from 100 to 1000.
- [x] **Global History Limit**: Add a global (default) setting to control the maximum number of history entries retained per file.
- [x] **Line Limit**: Implement a 9999 line limit per file/entry to maintain performance and keep the focus on note-taking rather than large databases.
- [ ] **History View Modes (Simple / Detailed)**: Add a toggle to switch between a Detailed view (current) and a Simple view (clean, one-line-per-entry list with exact date/time).
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [x] **Persistent Panel Widths**: Make the Debug, History, Settings, and Theme Editor panels remember their widths when resized by the user, similar to the File Tree.
- [x] **Font Selection Confirmation**: Allow confirming UI and Editor font selection with the Enter key.
- [x] **Quick Font Scaling**: Add support for Ctrl+Scroll and Ctrl++/Ctrl+- to quickly resize editor font.
- [x] **Line Height Adjustment**: Add setting to adjust the vertical spacing (line height) between rows in the text editor.
- [x] **Scroll Animation Timing**: Shorten the animation time when scrolling from the end of a long line to the beginning of the next one.
- [x] **Customizable Text Cursor**: Add option to change the text editor cursor shape (e.g., vertical bar, block, underscore).
- [ ] **Auto-Backup on Save**: Add option to automatically copy encrypted .sen files to multiple backup folders on save.
- [x] **Additional Keyboard Shortcuts**: Implement missing shortcuts, such as for "Open Directory".
- [x] **Smart Directory Opening**: Prevent save prompts when opening a directory in the file tree, as it doesn't close or modify the current file.
- [x] **File Format Migration (SED3 to SEN1)**: Update the file magic header from `SED3` to `SEN1` to reflect the project's rebranding. Backward compatibility for SED3 is not required.
- [x] **Editor Comfort (Phase 1)**: Increase the right margin in the text editor to improve readability and user comfort.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [x] **Navbar Visuals (Side Position)**: Fix black bars and disable edge-grabbing/resizing when the toolbar is positioned on the left or right.
- [x] **Remove Legacy Icons**: Delete the shield icon in the bottom-right corner (previously related to clipboard security).
- [x] **Global Keyfile Path Visibility**: When "Show keyfile paths globally" is disabled in settings, do not show any trace of the keyfile path/filename in the status bar, settings panel, batch converter, or debug logs.
- [x] **Editor Panning & Scrollbars**: 
    - Disable middle-mouse panning when the document is empty.
    - Hide scrollbars when content fits within the panel.
- [x] **Scroll Animation Timing**: Shorten the animation time when scrolling from the end of a long line to the beginning of the next one.
- [x] **Duplicate "Dark" Theme**: Fix the issue where "Dark" theme appears twice in the theme selection list.
- [ ] **Toolbar Icon Overlap**: Prevent icons on the left and right sides of the top menu from overlapping when the window is narrow or there are too many icons.
- [x] **Status Message Glitch**: Remove unsupported character appearing at the end of "Ready with global keyfile loaded" message.
- [x] **Duplicated Release Notes**: Fix issue where GitHub Actions generated multiple "Full Changelog" entries in release descriptions.

---

## 🛡️ Security & Privacy (Planned)

- [ ] **Screen Capture Protection**: Add an option to block application screenshots (OS-level protection where possible) with a toggle in settings.

---

> [!NOTE]
> This list is dynamic and will be updated as the project evolves. Specific details for these items will be provided by the project maintainer.
