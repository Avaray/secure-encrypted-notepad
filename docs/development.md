# SEN - Development Guide

This document provides instructions for developers who want to build, run, or contribute to the **SEN (Secure Encrypted Notepad)** project.

## 🛠 Prerequisites

To develop SEN, you need the following installed on your system:

1.  **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs/).
    - Recommended version: Stable (Rust 1.75+).
2.  **Git**: To clone the repository.
3.  **Platform Dependencies**:
    - **Windows**: No extra dependencies (uses MSVC or GNU toolchain).
    - **Linux**: Requires development headers for `libsecret` and GUI libraries.
        - *Ubuntu/Debian:* `sudo apt install libsecret-1-dev libssl-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev`
    - **macOS**: No extra dependencies (uses internal Keychain and AppKit).

## 🚀 Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/Avaray/sen.git
cd sen
```

### 2. Run the Application (Debug Mode)
This uses the `dev` profile with incremental compilation and light optimizations for a balanced experience.
```bash
cargo run
```

### 3. Build for Production
There are two ways to build a release binary:

- **Standard Release**: Fast build with Thin LTO and optimizations.
  ```bash
  cargo build --release
  ```
- **Final Release (Optimized & Small)**: Slow build with full LTO and size optimizations (used for public releases).
  ```bash
  cargo build --profile release-final
  ```

## 🔧 Development Workflow

- **Fast Check**: Always run this before a full build to catch syntax and type errors.
  ```bash
  cargo check
  ```
- **Testing**: Run unit tests (especially for theme resolution and crypto logic).
  ```bash
  cargo test
  ```
- **Linting**: Keep the code clean and follow Rust best practices.
  ```bash
  cargo clippy
  ```
- **Formatting**: Ensure consistent code style.
  ```bash
  cargo fmt
  ```

## 🌍 Localization (i18n)

SEN uses `rust-i18n` for multi-language support. 
- Translation files are located in `locales/*.yml`.
- To add a new language:
    1. Create `locales/XX.yml`.
    2. Add the SVG flag icon in `assets/flags/` (Emojione style recommended).
    3. Register the new flag in `src/icons.rs` (`Icons` struct and `load()` method).
    4. Update `src/settings.rs` (`default_language` detection).
    5. Update `src/ui_panels.rs` (Language Selector UI).

## 📁 Project Structure

- `src/`: All Rust source code.
    - `main.rs`: Application entry point.
    - `app.rs`: Main application logic, event loop, and state management.
    - `app_actions.rs`: Implementation of high-level actions (Open, Save, Export, etc.).
    - `app_helpers.rs`: Logging, background task management, and UI utilities.
    - `app_state.rs`: Enums and shared state structures.
    - `crypto.rs`: Core XChaCha20-Poly1305 encryption logic for `.sen` files.
    - `config_crypto.rs`: AES-256 encryption for protecting sensitive paths in `settings.toml`.
    - `settings.rs`: Persistence and management of user preferences.
    - `history.rs`: Management of document snapshots and metadata.
    - `theme.rs`: Custom theme engine (TOML) and color resolution logic.
    - `icons.rs`: SVG rendering and icon management.
    - `fonts.rs`: Dynamic font discovery and system font handling.
    - `single_instance.rs`: Windows-specific single instance enforcement via named pipes.
    - `ui_*.rs`: Modular GUI components:
        - `ui_toolbar.rs`: Main action bar.
        - `ui_editor.rs`: The main text editor and line numbers.
        - `ui_panels.rs`: Settings, History, Debug, and Theme Editor panels.
        - `ui_search.rs`: Search and replace interface.
        - `ui_batch.rs`: Batch encryption/decryption converter.
        - `ui_dialogs.rs`: Modal confirmation and utility dialogs.
- `assets/`: UI icons and branding resources.
- `docs/`: Technical documentation and design notes.

## ⚙️ Configuration Location

SEN stores its settings and encryption keys in standard OS directories:

- **Windows**: `%APPDATA%\sen`
- **Linux**: `~/.config/sen`
- **macOS**: `~/Library/Application Support/sen`

> [!IMPORTANT]
> The `.keyfile_key` file in these directories is essential for decrypting paths in your `settings.toml`. Do not delete it if you have a global keyfile configured!
