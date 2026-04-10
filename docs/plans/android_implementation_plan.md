# Android Implementation Proposal (Google Play)

## Ovreview

Following the successful migration to a Cargo workspace, the `secure-encrypted-notepad` (SEN) project is now architecturally ready to support multi-platform targets. This document outlines the implementation plan for creating an Android version of SEN targeted for the Google Play Store.

## 1. Project Structure & Cargo Workspace

Thanks to the workspace refactoring, we can create a dedicated crate for the Android target that depends on the shared core/UI logic, keeping the desktop and mobile entry points completely separate.

*   **New Crate**: Create a new crate named `sen-android` within the workspace (`crates/sen-android`).
*   **Shared Logic**: Ensure `sen-core` (or the unified logic crate) is heavily utilized by `sen-android`. Egui UI definitions and state management should remain platform-agnostic as much as possible.
*   **Storage Abstraction Layer**: To keep our core clean and prevent JNI/native logic from bleeding into it, we will define a `FileSystem` trait in `sen-core`:
    ```rust
    sen-core
      └── trait FileSystem { open, save, ... }
            ├── DesktopFs  (std::fs)
            └── AndroidFs  (JNI/SAF)
    ```
    This way, `sen-android` and `sen-desktop` simply provide their platform-specific implementation.

## 2. Tooling & Prerequisites

To compile Rust code for Android, we will need to set up the appropriate cross-compilation toolchains and utilize tools built for Rust-to-Android integration.

*   **Rust Targets**: Add Android targets via rustup:
    *   `aarch64-linux-android` (Standard 64-bit ARM)
    *   `armv7-linux-androideabi` (Older 32-bit ARM, if supported)
    *   `x86_64-linux-android` (For emulators)
*   **NDK and SDK**: Install the Android SDK and NDK via Android Studio. Set the `ANDROID_SDK_ROOT` and `ANDROID_NDK_ROOT` environment variables.
*   **Build Tools**: Use `cargo-ndk` to generate the APK and Android App Bundle (.aab), as it is newer and better supported than `cargo-apk`. Google Play requires the `.aab` format for new submissions.
*   **Windowing/Integration**: Use `android-activity` backend (now standard in modern `winit`/`eframe`) to handle Android application lifecycles natively in Rust.

## 3. Platform-Specific Adjustments

### 3.1. File System & Storage (Scoped Storage)
Android 11+ enforces Scoped Storage. Direct filesystem access using standard `std::fs` to arbitrary paths is not allowed.
*   **Solution**: We must use Android's Storage Access Framework (SAF). Writing a custom JNI bridge to SAF from scratch is tedious and error-prone. Instead, we should explore using:
    *   **`android-document-picker`** or an abstraction at the `winit`/`eframe` level.
    *   `android-activity` 0.6+, which might support `ACTION_OPEN_DOCUMENT` and handle intents without manual JNI.
    *   Alternatively, the `jni` crate combined with ready-made wrappers from the `ndk` crate instead of starting from zero.
*   **File Handling**: The returned URIs will need to be resolved to file streams or temporarily copied into the app's cache directory to be manipulated by Rust.

### 3.2. Soft Keyboard Handling
Text inputs in Egui need to accurately trigger the Android virtual keyboard.
*   **Solution**: Ensure `eframe` is correctly configured to show/hide the software keyboard when `egui::TextEdit` gains/loses focus. We may need platform-specific tweaks if the viewport gets obscured by the keyboard.

### 3.3. UI Touch Optimization
While Egui is usable on mobile, it originates on desktop.
*   **Solution**: Provide a mobile-specific configuration scaling up `egui::Context::set_pixels_per_point`. Emphasize larger touch targets for buttons and adjust scrolling sensitivity. Ensure the "Hover" states are minimized as they don't apply well to touchscreens.

## 4. Feature Scope & UI Streamlining

The Android version will intentionally feature a streamlined and more constrained user interface compared to the desktop client. To maximize usability and clarity on smaller mobile screens, complex customization options will be omitted, focusing entirely on the most essential core functionalities.

**Included Core Features (Action Menu Equivalents):**
*   **Open File**: Native Android file picker integration to load encrypted `.sen` files or plaintext documents.
*   **Save / Save As**: Saving documents securely back to the device storage.
*   **Password & Keyfile Management**: Interfaces to input encryption passwords or select keyfiles.
*   **Core Text Editor**: The primary workspace for reading and editing text.
*   **Basic Settings**: Minimal settings required for app functionality, such as language/localization toggles.

**Excluded Features:**
*   **Custom Theme Editor**: The complex color picking and theme generation tools will be removed. The app will strictly use a polished, pre-defined light/dark scheme.
*   **Advanced Font Customization**: Options to load external fonts or manually adjust all typography sizes will be omitted. Desktop-centric font rendering tools will be replaced with standard system typography.
*   **Complex Debug/Developer Panels**: The desktop debug/log UI will be hidden or removed for the end-user release.

## 5. Permissions & AndroidManifest.xml

We need a carefully crafted `AndroidManifest.xml` within the `sen-android` crate.
*   We generally won't need generic `READ_EXTERNAL_STORAGE` or `WRITE_EXTERNAL_STORAGE` if we strictly use the SAF (File Picker intents), which is an advantage for our privacy-focused notepad.
*   No networking permissions are necessary, emphasizing the offline-first security of the app.

## 6. Google Play Store Requirements

For a successful release over Google Play, the following steps are mandatory:
*   **App Bundle (.aab)**: The build process must spit out an `.aab` file instead of directly distributing `.apk`s.
*   **Key Signing**: Generate a Keystore to sign either the release build or allow Google Play App Signing to manage the key.
*   **Assets**: We need to compile adaptive icons (`mipmap`) and standard splash screens natively in the Android resources directory.
*   **Privacy Policy**: As an encryption app, a clear privacy policy page must be created and linked to the Google Play Store declaring how data is handled (entirely locally).

## 7. Implementation Phases

1.  **Phase 1: Proof of Concept**: Initialize `sen-android`, configure `cargo-ndk`, and get a basic Egui window rendering on an Android emulator.
2.  **Phase 2: Storage Abstraction & Integration**: Implement the `FileSystem` trait for desktop and Android, leveraging `android-activity`/SAF intents for the latter.
3.  **Phase 3: Feature Streamlining**: Strip out the desktop-specific settings and enforce the simplified Android-centric user interface. Doing this first reduces the surface area that needs styling making the subsequent step simpler.
4.  **Phase 4: Mobile UI Polish**: Implement the mobile layout adjustments, dynamic font scaling, and soft keyboard integration.
5.  **Phase 5: Build & Release Pipeline**: Update GitHub Actions to automatically compile `.aab` files using the Android NDK and sign them for release. Properly configure NDK toolchain caching (e.g., via `actions/cache`) to eliminate minutes of setup overhead on every CI build.

This plan sets the foundation for standardizing SEN across desktop and Android without compromising its foundational security and performance elements.
