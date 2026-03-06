# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Internationalization (I18n)**: Add support for multiple application languages.
- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Editor Comfort (Phase 2)**: Increase the right margin in the main editor further to ensure text never touches the edge, especially when typing spaces (ensuring space at the end of lines).
- [ ] **Theme Editor Hover Colors**: Add ability to customize the background color of menu buttons when hovered.
- [ ] **Theme System Refactoring**: Refactor the color system to allow customization of more editor components.
- [ ] **History Capacity**: Increase the maximum allowed file history entries from 100 to 1000.
- [ ] **Global History Limit**: Add a global (default) setting to control the maximum number of history entries retained per file.
- [ ] **Line Limit**: Implement a 9999 line limit per file/entry to maintain performance and keep the focus on note-taking rather than large databases.
- [ ] **History View Modes (Simple / Detailed)**: Add a toggle to switch between a Detailed view (current) and a Simple view (clean, one-line-per-entry list with exact date/time).
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **Persistent Panel Widths**: Make the Debug, History, Settings, and Theme Editor panels remember their widths when resized by the user, similar to the File Tree.
- [x] **File Format Migration (SED3 to SEN1)**: Update the file magic header from `SED3` to `SEN1` to reflect the project's rebranding. Backward compatibility for SED3 is not required.
- [x] **Editor Comfort (Phase 1)**: Increase the right margin in the text editor to improve readability and user comfort.

---

## 🐛 Bug Fixes (To Fix)

*Items in this section represent confirmed issues or glitches that need to be addressed.*

- [ ] **Navbar Visuals (Side Position)**: Fix black bars and disable edge-grabbing/resizing when the toolbar is positioned on the left or right.
- [x] **Remove Legacy Icons**: Delete the shield icon in the bottom-right corner (previously related to clipboard security).
- [ ] **Status Bar Keyfile Visibility**: When "Show keyfile path" is disabled in settings, do not show any trace of the keyfile path/indicator in the status bar at all.
- [x] **Editor Panning & Scrollbars**: 
    - Disable middle-mouse panning when the document is empty.
    - Hide scrollbars when content fits within the panel.
- [ ] **Scroll Animation Timing**: Shorten the animation time when scrolling from the end of a long line to the beginning of the next one.
- [x] **Duplicate "Dark" Theme**: Fix the issue where "Dark" theme appears twice in the theme selection list.
- [ ] **Toolbar Icon Overlap**: Prevent icons on the left and right sides of the top menu from overlapping when the window is narrow or there are too many icons.

---

## 🛡️ Security & Privacy (Planned)

- [ ] **Screen Capture Protection**: Add an option to block application screenshots (OS-level protection where possible) with a toggle in settings.

---

> [!NOTE]
> This list is dynamic and will be updated as the project evolves. Specific details for these items will be provided by the project maintainer.
