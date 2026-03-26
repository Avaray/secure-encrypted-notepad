# SEN - Project TODO List

This document tracks planned features and reported bugs for the **SEN (Secure Encrypted Notepad)** project.

---

## 🚀 Features & Improvements (To Implement)

*Items in this section represent new functionality or enhancements planned for future releases.*

- [ ] **Icon Refresh**: Replace current temporary icons with higher-quality, professionally designed ones.
- [ ] **Logo & App Icon**: Create a new professional logo for SEN and replace the current application icon.
- [ ] **UI Icon Sets**: Add the ability to change icon sets from a dropdown in settings. This will allow users to choose between different visual styles for the application icons (folders, keys, files, etc.), requiring architectural changes to support dynamic icon loading.
- [ ] **Stealth Mode**: Option to save files without the `.sen` extension and without any identifying headers (pure binary noise) to make it impossible to identify the application associated with the file. By default, `.sen` files would load automatically in the file tree, while extensionless "stealth" files would require a background verification process (trial decryption) to be identified and marked.
- [ ] **Documentation Deep Audit**: Perform a comprehensive documentation audit using a high-level thinking model.
- [ ] **Font Autodetection List Polish**: Refine the preferred font lists (UI and Editor) for smart detection, as the current lists are early versions that need better selection.
- [ ] **Cargo Workspace Refactoring**: Consider refactoring the project into a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) to better support future platforms like Android, allowing shared logic between desktop and mobile versions.
- [ ] **History Revert Button**: Add a "Revert changes" button in the history panel. If the user marks entries for deletion (single or all) but hasn't saved the file yet, this button should appear and allow reverting history to its last saved state.
- [ ] **Translation Polish**: Review and refine translations throughout the application to ensure consistency and correctness across all supported languages.
- [ ] **Refactor `center_row` Helper**: Improve or replace the `crate::app_helpers::center_row` helper to ensure consistent vertical centering.
    - It should act like Tailwind's `items-center`, ensuring all elements in the row are perfectly aligned vertically and stay in one line.
    - It must be robust enough for universal use: Settings options, panel headings (with close buttons), and other UI elements.
    - **Current Status**: Previous attempts (nested `horizontal` + `with_layout`, `allocate_ui_with_layout`, using `Button` as `Label`) have not provided a perfect visual fix. Mathematical centering often results in baseline misalignment due to `egui`'s widget padding and `interact_size`.
    - **Open Problem**: Achieving "perfect to the eye" vertical alignment for mixed labels and inputs without breaking parent container layouts (e.g., preventing headers from expanding to fill the application window).
    - Goal: A reliable, reusable solution for vertically centered horizontal layouts throughout the application.
- [ ] **Theme Editor Color Audit & Smarter Defaults**: Reduce the manual effort required to create themes by implementing intelligent color defaults.
    - Many UI elements (e.g., separators) should automatically derive their default colors from primary sources like the application's text foreground color.
    - Perform a thorough audit of the existing color set to identify opportunities for simplification and derivation.
    - Goal: A more user-friendly theme creation process that doesn't sacrifice depth of customization.
- [x] **File Tree Max Width**: Limit the maximum width of the file tree panel (e.g., to 90% of the screen) to prevent it from covering the entire editor.

---

## 🐛 Bug Fixes (To Fix)

- [ ] **Theme Editor UI Polish (Copy/Paste & Color Pickers)**: Ensure ideal alignment of grid elements in the theme editor.
    - Fix sizes of copy/paste buttons (should be framed, squared, matching `interact_size.y` height).
    - Replace the native `ui.color_edit_button_srgb` with custom buttons (e.g., `ui.add(egui::Button::image(...).fill(color))`). This allows forcing a square `min_size` matching the input height, eliminating the native selector's rigid dimensions.
    - Use `egui::show_tooltip_for_rect` or `egui::Popup` with `egui::color_picker::color_picker_color32` to display the color adjustment interface only after clicking the custom button.
- [ ] **Settings Slider Crash**: Investigate and fix application crashes occurring when rapidly dragging value-based sliders (e.g., Font Size, Line Height, Transparency) in the settings and theme panels.
- [ ] **Menu Bar Margin Fix (Vertical Position)**: Fix inconsistent horizontal margins when the menu bar is positioned vertically (left/right side).
    - When on the left, the left margin is larger than the right one; when on the right, the right margin is larger than the left one.
    - Identify the source of this asymmetry and ensure uniform horizontal margins.
    - Additionally, reduce top and bottom margins in the vertical position to ensure equal spacing on all sides and improved aesthetics.
    - **Note**: Ensure that the existing scrollability logic for menu bar icons (which kicks in when icon sizes/count exceed available space) remains functional and unaffected by margin changes.
- [ ] **Scrollbar Flickering Bug**: Fix the infinite flickering/flashing effect when the mouse hovers near the vertical line that splits panels.
    - This happens when a scrollbar appears in the same location as the splitter, causing the layout to enter an unstable state where it cannot decide if the scrollbar should be visible/grown or not.
    - Goal: Ensure stable layout even when scrollbars and splitters overlap or interact.
- [x] **Search Panel Sync (Undo/Redo)**: If Replace/Replace All is used and then undone (CTRL + Z), the search panel doesn't refresh and findings are not highlighted.
    - Goal: Ensure search results are always synchronized with the current editor content, especially after undo/redo operations.
