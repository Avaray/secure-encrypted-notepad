//! Theme module — re-exports core theme types and adds egui extensions.
//!
//! All data structures live in `sen_core::theme`.
//! The egui-specific `ThemeColorsExt` and `ThemeExt` traits live in `theme_ext`.

// Re-export everything from sen-core's theme module
pub use sen_core::theme::*;

// Re-export egui extension traits so callers get them automatically
pub use crate::theme_ext::{ThemeColorsExt, ThemeExt};
