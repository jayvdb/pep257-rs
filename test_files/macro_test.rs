/// Log an error message to stderr with ERROR prefix.
///
/// This macro provides a convenient way to log error messages
/// with consistent formatting.
///
/// # Examples
///
/// ```
/// log_error!("Something went wrong: {}", error);
/// ```
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("ERROR: {}", format_args!($($arg)*));
    };
}

/// Create a new vector with the given elements.
#[macro_export]
macro_rules! vec_of {
    ($($x:expr),*) => {
        vec![$($x),*]
    };
}

// Missing docstring - should trigger R103
#[macro_export]
macro_rules! undocumented_macro {
    () => {
        println!("Hello");
    };
}

/// Assert that a condition is true with a custom message.
///
/// Similar to `assert!` but with more detailed error reporting.
macro_rules! assert_with_msg {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            panic!("Assertion failed: {}", $msg);
        }
    };
}
