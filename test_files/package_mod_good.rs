//! Utility functions module.
//!
//! Contains helper functions used throughout the crate.

#[cfg(any())]
pub mod helpers;
#[cfg(any())]
pub mod validators;

/// Helper function.
pub fn format_message(msg: &str) -> String {
    format!("[INFO] {}", msg)
}
