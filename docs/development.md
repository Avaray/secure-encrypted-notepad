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
This is the fastest way to test your changes. It uses the `dev` profile with incremental compilation.
```bash
cargo run
```

### 3. Build for Production
There are two ways to build a release binary:

- **Standard Release**: Fast build with optimizations.
  ```bash
  cargo build --release
  ```
- **Final Release (Optimized & Small)**: Slow build with LTO (Link-Time Optimization) and size optimizations.
  ```bash
  cargo build --profile release-final
  ```

## 🔧 Development Workflow

- **Fast Check**: Always run this before a full build to catch syntax and type errors.
  ```bash
  cargo check
  ```
- **Linting**: Keep the code clean and follow Rust best practices.
  ```bash
  cargo clippy
  ```
- **Formatting**: Ensure consistent code style.
  ```bash
  cargo fmt
  ```

## 📁 Project Structure

- `src/`: All Rust source code.
    - `crypto.rs`: Core logic for .sen file encryption (XChaCha20).
    - `config_crypto.rs`: Encryption for sensitive paths in settings.
    - `ui_*.rs`: GUI modules built with `egui`.
- `assets/`: Icons and static resources.
- `docs/`: Technical documentation and research notes.

## ⚙️ Configuration Location

SEN stores its configuration (`config.toml`) and keys in the following standard directories:

- **Windows**: `%APPDATA%\sen`
- **Linux**: `~/.config/sen`
- **macOS**: `~/Library/Application Support/sen`

> [!IMPORTANT]
> The `.keyfile_key` file in these directories is essential for decrypting paths in your `config.toml`. Do not delete it if you have a global keyfile configured!
