//! PEP 257 docstring style checker for Rust code.

/// Analyzer module for Rust documentation.
pub mod analyzer;
/// File collection module for finding Rust source files.
pub mod file_collector;
/// Parser module for extracting docstrings.
pub mod parser;
/// PEP 257 checker implementation.
pub mod pep257;

// Re-export commonly used public functions
pub use file_collector::collect_rust_files_recursive;
