// Missing module-level documentation - should trigger D104

pub mod helpers;
pub mod validators;

/// Helper function.
pub fn format_message(msg: &str) -> String {
    format!("[INFO] {}", msg)
}
