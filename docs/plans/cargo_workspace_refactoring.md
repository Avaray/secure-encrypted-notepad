# Cargo Workspace Refactoring Plan: Secure Encrypted Notepad (SEN)

## Overview
This document outlines a step-by-step refactoring plan to split the current monolithic repository into a Cargo Workspace. The primary goal is to isolate the core business logic (cryptography, state management, file parsing) from the Desktop UI layer (eframe/egui). This separation will allow future platforms (like Android) to use `sen-core` natively without dragging in desktop-specific dependencies.

## Target Architecture
- **Root Workspace**: Manages shared dependencies and build profiles.
- **`crates/sen-core`**: A UI-agnostic library containing cryptography, settings data models, and logic.
- **`crates/sen-desktop`**: The existing eframe/egui GUI, depending on `sen-core`.

---

## Execution Steps

Please execute the following steps in order. Stop and verify that the project compiles after completing Phase 2 and Phase 3.

### Phase 1: Workspace Skeleton Initialization
1. Create a `crates/` directory in the project root.
2. Inside `crates/`, create two subdirectories: `sen-core` and `sen-desktop`.
3. Move `src`, `build.rs`, and the original `Cargo.toml` into `crates/sen-desktop/`. Make sure to rename `crates/sen-desktop/Cargo.toml`'s package name to `sen-desktop` if necessary (keeping binary name as `sen`).
4. Move `assets/` and `locales/` into `crates/sen-desktop/` (since these are currently tied to the UI).
5. Create a new `Cargo.toml` at the root of the repository with a `[workspace]` definition containing `members = ["crates/sen-core", "crates/sen-desktop"]`.
6. Initialize `crates/sen-core/` with a basic `src/lib.rs` and its own `Cargo.toml`.

### Phase 2: Dependency Splitting & Workspace Configuration
1. Move the `[profile.*]` sections from the desktop `Cargo.toml` up to the root workspace `Cargo.toml`.
2. Extract common dependencies into `[workspace.dependencies]` in the root `Cargo.toml` to unify versions (e.g., `serde`, `orion`, `aes-gcm`, etc.).
3. Configure `crates/sen-core/Cargo.toml` to include core logic dependencies:
   - Cryptography (`orion`, `aes-gcm`, `sha2`, `hkdf`)
   - Utils (`rand`, `zeroize`, `hex`, `base64`)
   - Serialization (`serde`, `serde_json`, `toml`)
4. Configure `crates/sen-desktop/Cargo.toml` to include UI and system dependencies:
   - UI (`eframe`, `egui`, `egui_extras`, `resvg`, `font-kit`)
   - Platform (`rfd`, `dirs`, `windows-sys`, `winreg`, `notify`)
   - Local `path` dependency on `sen-core`.

### Phase 3: Extracting Modules into `sen-core`
Move the following files from `crates/sen-desktop/src/` to `crates/sen-core/src/`:
1. `crypto.rs`
2. `config_crypto.rs`
3. `settings.rs` (Data structures and persistence logic; remove egui types if any)
4. `history.rs` (Backup history data structures)
5. `theme.rs` (Theme configuration models. *Note: Replace `egui::Color32` with a generic RGBA structure like `[u8; 4]` to decouple from egui. The desktop crate can have helpers to convert `[u8; 4]` back to `Color32`.*)

### Phase 4: API Refactoring & Visibility Fixes
1. In `sen-core/src/lib.rs`, declare all moved modules as `pub` (e.g., `pub mod crypto;`).
2. Update struct, enum, and function definitions in `sen-core` to be `pub` so the desktop crate can access them.
3. Update imports in `crates/sen-desktop/src/*` to pull from `sen_core::` (e.g., `use sen_core::crypto;`).
4. Fix any compilation errors caused by the decoupling of UI objects from core structs (e.g., color conversions).

### Phase 5: Verification & Cleanup
1. Ensure the desktop application successfully builds with `cargo build`.
2. Ensure standard functionality works:
   - Create, encrypt, and decrypt `.sen` files.
   - Run unit tests in both crates (`cargo test --workspace`).
3. Document the new boundaries in the root `README.md`.

---

## Important Rules for the LLM during Execution
- **Incremental Steps**: Do not try to move thousands of lines of code in a single command. Move one module, fix its visibility, fix the imports on the desktop side, and type-check (`cargo check`).
- **No UI in Core**: `sen-core` must absolutely NOT depend on `eframe`, `egui`, or any file-dialog GUI crates. If a core function needs to return a locale string, return string keys and let the desktop crate resolve the translation.
- **Maintain Current Behavior**: Refactor purely for architecture; do not add new features or modify the existing encryption formats during this extraction process.
