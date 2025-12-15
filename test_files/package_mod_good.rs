//! Utility functions module.
//!
//! Contains helper functions used throughout the crate.

pub mod helpers;
pub mod validators;

/// Helper function.
pub fn format_message(msg: &str) -> String {
    format!("[INFO] {}", msg)
}
