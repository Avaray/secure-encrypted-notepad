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
4.  **Android Development** (Optional):
    - **Android Studio**: Jellyfish or newer recommended.
    - **Android SDK & NDK**: Installed via Android Studio (NDK version 26.x recommended).
    - **cargo-ndk**: Install via `cargo install cargo-ndk`.
    - **Rust Targets**: Add Android targets for your emulator/device:
        ```bash
        rustup target add aarch64-linux-android x86_64-linux-android
        ```
5.  **Bun**: Required for running utility scripts. Install via [bun.sh](https://bun.sh/).

## 🚀 Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/Avaray/secure-encrypted-notepad.git
cd secure-encrypted-notepad
```

### 2. Run the Desktop Application (Debug Mode)
This uses the `dev` profile with incremental compilation for a better experience.
```bash
cargo run
```

### 3. Build for Production
There are two ways to build a release binary:

- **Standard Release**: Balanced build with Thin LTO.
  ```bash
  cargo build --release
  ```
- **Final Release (Optimized)**: Full LTO and size optimizations (used for public releases).
  ```bash
  cargo build --profile release-final
  ```

### 4. Build and Run on Android

The Android version is a wrapper around the core Rust library using `GameActivity`.

1.  **Open Android Studio** and points to the `crates/sen-android/android` directory.
2.  **Wait for Gradle Sync** to finish.
3.  **Build/Run**: Click **Run** (▶) or run from CLI:
    ```bash
    cd crates/sen-android/android
    ./gradlew assembleDebug
    ```

> [!TIP]
> The Android project is configured to automatically run `cargo-ndk` during the `preBuild` phase. If you encounter errors, ensure `cargo-ndk` is in your PATH and your `ANDROID_NDK_HOME` environment variable is set to your NDK location.

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

## 🌍 Localization

SEN uses a custom **zero-latency translation parser** for multi-language support (extracted to the `sen-translations` workspace crate for near-instant compilation).
- Translation files are located in `crates/sen-translations/locales/*.yml`.
- To add a new language:
    1. Create `crates/sen-translations/locales/XX.yml`.
    2. Register the file in `crates/sen-translations/src/lib.rs` by adding a `load_lang!("xx", "../locales/xx.yml");` entry.
    3. Add the SVG flag icon in `crates/sen-desktop/assets/flags/` (Emojione style recommended).
    4. Register the new flag in `crates/sen-desktop/src/icons.rs` (`Icons` struct and `load()` method).
    5. Update `crates/sen-core/src/settings.rs` (`default_language` detection).
    6. Update `crates/sen-desktop/src/ui_panels.rs` (Language Selector UI).

## 🎨 Theme System

SEN features a highly customizable theme engine that allows users to create their own visual styles using [TOML](https://toml.io/en/) files.

### 1. Theme Locations
- **Default Themes**: Embedded `dark.toml` and `light.toml` are located in `crates/sen-core/themes/`.
- **Custom Themes**: Loaded from the user's configuration directory:
    - Windows: `%APPDATA%\sen\themes\*.toml`
    - Linux: `~/.config/sen/themes/*.toml`
    - macOS: `~/Library/Application Support/sen/themes/*.toml`

### 2. Theme Format
Themes are defined as TOML files. Colors are specified as arrays of either `[R, G, B]` or `[R, G, B, A]` where each value is 0-255. If alpha is omitted, it defaults to 255.

Example theme structure:
```toml
name = "My Custom Theme"
color_scheme = "Dark" # or "Light"

[colors]
background = [18, 18, 18, 255]
foreground = [255, 255, 255, 255]
accent = [100, 150, 255, 255]
# ... and many more granular color options
```

### 3. Implementation Details
The theme engine is split into two layers:
- **Core (`sen-core`)**: UI-agnostic data models and TOML serialization logic in `theme.rs`.
- **Egui Integration (`sen-core`)**: Conversion from RGBA to `egui::Color32` and application of visuals to the UI context in `theme_egui.rs`.

## 📜 Utility Scripts

The project includes several helper scripts in the `/scripts` directory to automate common development tasks. These scripts are written in TypeScript and run using [Bun](https://bun.com).

### 1. Locales Synchronization

The `scripts/locales-sync.ts` script ensures that all translation files stay in sync with English (the source language) using the [Gemini AI API](https://aistudio.google.com/api-keys). It handles automatic translation of missing keys, forced synchronization of specific keys, and cleanup of unused translations.

For detailed documentation, requirements, and comprehensive usage examples, please refer to the **[scripts/README.md](../scripts/README.md)**.


## 📁 Project Structure

SEN is organized as a Cargo Workspace to support code sharing across multiple platforms (e.g., Desktop, Android).

- `crates/sen-core/`: Core headless library containing the engine logic.
    - `crypto.rs`: XChaCha20-Poly1305 encryption logic for `.sen` files.
    - `config_crypto.rs`: AES-256 encryption for protecting sensitive paths in `config.toml`.
    - `settings.rs`: Persistence and management of user preferences (shared across platforms).
    - `theme.rs`: UI-agnostic theme data models and TOML serialization.
    - `theme_egui.rs`: Integration layer between core themes and the `egui` framework.
    - `history.rs`: Management of document snapshots and metadata.
- `crates/sen-translations/`: Custom internationalization engine and localization resources.
    - `locales/`: YAML translation files for all supported languages.
- `crates/sen-desktop/`: The main GUI application (Windows, Linux, macOS).
    - `src/main.rs`: Application entry point.
    - `src/app.rs`: Main application logic, event loop, and state management.
    - `src/app_actions.rs`: Implementation of high-level actions (Open, Save, Export, etc.).
    - `src/app_helpers.rs`: Logging, background task management, and UI utilities.
    - `src/fonts.rs`: Dynamic font discovery and system font handling.
    - `src/single_instance.rs`: Windows-specific single instance enforcement.
    - `src/ui_*.rs`: Modular GUI components (`ui_editor`, `ui_panels`, `ui_toolbar`, `ui_batch`, etc.).
    - `assets/`: UI icons and branding resources.
- `crates/sen-android/`: The Android port of SEN.
    - `src/lib.rs`: Entry point for the Android library (`android_main`).
    - `android/`: The native Android Studio / Gradle project.
    - `android/app/src/main/java/com/sen/android/MainActivity.kt`: The Kotlin wrapper.
- `docs/`: Technical documentation and design notes.
- `scripts/`: Bun utility scripts for translation synchronization and automated maintenance.
- `.github/`: CI/CD automation workflows for testing and automated releases.


## ⚙️ Configuration Location

SEN stores its settings and encryption keys in standard OS directories:

- **Windows**: `%APPDATA%\sen`
- **Linux**: `~/.config/sen`
- **macOS**: `~/Library/Application Support/sen`

> [!IMPORTANT]
> The `.keyfile_key` file in these directories is essential for decrypting paths in your `config.toml`. Do not delete it if you have a global keyfile configured!
