/// A specialized Result type for this crate's operations.
///
/// This type alias simplifies error handling by providing a consistent
/// Result type throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// A vector of strings.
pub type StringVec = Vec<String>;

// Missing docstring - should trigger R101
pub type UndocumentedType = i32;

/// Generic callback function type.
///
/// Used for event handlers and callbacks throughout the system.
pub type Callback<T> = Box<dyn Fn(T) -> ()>;

/// Configuration map type.
pub type ConfigMap = std::collections::HashMap<String, String>;
