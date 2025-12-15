// Missing crate-level documentation - should trigger D104

#[cfg(any())]
pub mod calculator;
#[cfg(any())]
pub mod utils;

/// Add two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
