//! Core library for Secure Encrypted Notepad (SEN).
//!
//! This crate contains all platform-agnostic business logic:
//! - File encryption/decryption (XChaCha20-Poly1305 via `orion`)
//! - Config encryption (AES-256-GCM with machine-bound key wrapping)
//! - Settings data model and persistence
//! - Version history embedded in encrypted files
//! - Theme data model (UI-agnostic `[u8; 4]` colors)

/// Custom debug logging macro that only prints to console in debug builds.
/// This prevents sensitive data leakage in release versions.
#[allow(unused_macros)]
macro_rules! sen_debug {
    ($($arg:tt)*) => {
        {
            #[cfg(debug_assertions)]
            eprintln!("[SEN-CORE] {}", format!($($arg)*));

            #[cfg(not(debug_assertions))]
            if false {
                let _ = format_args!($($arg)*);
            }
        }
    };
}

// pub(crate) use sen_debug;

pub mod config_crypto;
pub mod crypto;
pub mod history;
pub mod settings;
pub mod theme;
pub mod fs;
pub mod models;

#[cfg(feature = "egui")]
pub mod theme_egui;
