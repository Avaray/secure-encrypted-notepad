extern crate rust_i18n;

// Initialize i18n
i18n!("locales", fallback = "en");

// Re-export rust-i18n macros and functions so consumers just depend on sen_i18n
pub use rust_i18n::*;
