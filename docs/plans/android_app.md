# SEN Android - Mobile Port Design Plan

This document outlines the architecture, UI design, and feature set for the Android version of **Secure Encrypted Notepad (SEN)**.

## 📱 Vision
The Android port aims to be a fully functional companion to the desktop app, allowing users to read and edit their `.sen` files securely on the go. While some power-user features (like the complex batch converter or advanced file tree) may be simplified, the core security and editing experience must remain identical.

## 🛠 Core Architecture

### 1. Shared Logic Layer
- **Encryption**: Uses `sen-core` (XChaCha20-Poly1305) via the shared Rust library.
- **I18n**: Uses `sen-translations` with shared YAML locale files.
- **Themes**: Uses the shared TOML theme engine to ensure visual consistency.

### 2. Android-Specific Implementation
- **Launcher**: `GameActivity` (C++ / Rust bridge) to host the Egui event loop.
- **Storage**: Integration with **Storage Access Framework (SAF)** via JNI (Content URIs instead of direct paths).
- **IME (Keyboard)**: JNI calls to trigger and manage the Android soft keyboard.
- **Lifecycle**: Handling `onPause`/`onResume` to ensure auto-save and sensitive data clearing from memory.

---

## 🎨 User Interface (UI) Design

### 1. Main Editor Screen
- **Top Toolbar**:
    - `[Hamburger Menu]` (left) - Opens a side drawer or overlay with all app options (New, Open, History, Settings, About, etc.).
    - `[Toggle File Tree]` | `[Save]` | `[Close File]` (right).
- **Central Area**:
    - Full-screen `TextEdit` with large touch padding.
    - Soft scroll support with momentum.
- **Status Bar**: Removed (to maximize screen space). Performance and key status will be visible within the Menu or as subtle toast notifications.

### 2. Settings (Full-screen Overlay)
- **Security**: Toggle "Global Keyfile", change password.
- **Aesthetics**: Theme selector (Horizontal carousel or list), Font size slider.
- **Localization**: Language picker with flag icons.

### 3. Navigation & Gestures
- **Swipe Down**: Refresh / Repaint.
- **Double Tap**: Select word / Trigger keyboard.
- **Back Button**: Prompt to save if modified, then exit.

---

## 📋 Feature Roadmap

### Phase 1: Interaction Polish (In Progress)
- [ ] **Soft Keyboard Integration**: Fix the non-responsive `TextEdit` by bridging JNI focus events to `winit`.
- [ ] **Responsive Scaling**: Implement `pixels_per_point` adjustment to ensure the UI looks premium on high-DPI mobile screens.
- [ ] **Touch Padding**: Enforce touch targets of at least 44x44 points for all buttons.

### Phase 2: File Persistence (SAF)
- [ ] **External Open**: Implement `Intent.ACTION_OPEN_DOCUMENT` to load `.sen` files from Google Drive / Local Storage.
- [ ] **External Save**: Implement `Intent.ACTION_CREATE_DOCUMENT`.
- [ ] **Content URI Bridge**: Map Android `content://` URIs to Rust streams (avoiding direct path access which is restricted on Android 11+).

### Phase 3: Desktop Parity
- [ ] **History Port**: Port the Undo/Redo (Snapshots) UI to a mobile-friendly drawer.
- [ ] **Theme parity**: Ensure the desktop themes (`Dark`, `Light`) are correctly loaded from assets.
- [ ] **Stealth Mode**: Toggle to hide file headers/extensions, matching desktop behavior.

---

## 🔒 Security Considerations (Mobile)
- **Memory Security**: Prompt to clear clipboard on app minimize.
- **Biometric Unlock**: (Future) Optional integration with Android BiometricPrompt to unlock the global keyfile.
- **Screen Protection**: Use `WindowFlag.SECURE` (already planned) to prevent screenshots of sensitive notes.

## 🚀 Deployment
- Build via `./gradlew assembleDebug` (already working).
- Final distribution via Google Play Store (Internal Testing) or direct APK download.
