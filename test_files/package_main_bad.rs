// Missing crate-level documentation - should trigger D104

use std::env;

/// Main entry point.
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Arguments: {:?}", args);
}
