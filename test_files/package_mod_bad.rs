// Missing module-level documentation - should trigger D104

#[cfg(any())]
pub mod helpers;
#[cfg(any())]
pub mod validators;

/// Helper function.
pub fn format_message(msg: &str) -> String {
    format!("[INFO] {}", msg)
}
