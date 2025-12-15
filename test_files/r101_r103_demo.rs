//! Demonstration of R101 (type alias) and R103 (macro) rules.

/// A specialized Result type for this module.
pub type Result<T> = std::result::Result<T, Error>;

// Missing docstring - should trigger R101
pub type BadType = String;

/// Log an error message with formatting.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("ERROR: {}", format_args!($($arg)*));
    };
}

// Missing docstring - should trigger R103
#[macro_export]
macro_rules! bad_macro {
    () => {};
}

/// A simple error type for this module.
pub struct Error {
    message: String,
}
