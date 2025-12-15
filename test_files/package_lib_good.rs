//! A mathematics library.
//!
//! Provides various calculation and utility functions.

#[cfg(any())]
pub mod calculator;
#[cfg(any())]
pub mod utils;

/// Add two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
