//! Command-line tool for calculations.
//!
//! This binary provides a CLI interface for various mathematical operations.

use std::env;

/// Main entry point.
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments: {:?}", args);
}
