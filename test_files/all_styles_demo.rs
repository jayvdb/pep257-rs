/// This module demonstrates all documentation styles.
///
/// It shows how the PEP 257 checker works with different Rust doc comment formats.

/// Calculate the sum using line comments.
///
/// This is the most common style in Rust code.
fn add_line_comment(a: i32, b: i32) -> i32 {
    a + b
}

/**
 * Calculate the difference using block comments.
 * 
 * This style is less common but still valid.
 */
fn subtract_block_comment(a: i32, b: i32) -> i32 {
    a - b
}

#[doc = "Calculate the product using doc attributes."]
#[doc = ""]
#[doc = "This style is useful for procedural macros and code generation."]
fn multiply_doc_attribute(a: i32, b: i32) -> i32 {
    a * b
}

/// Represents a mathematical operation.
///
/// This enum demonstrates documentation on different item types.
enum Operation {
    /// Addition operation
    Add,
    /// Subtraction operation
    Subtract,
    /// Multiplication operation
    Multiply,
}

#[doc = "Represents a calculator."]
#[doc = ""]
#[doc = "This struct uses doc attributes for demonstration."]
struct Calculator {
    /// Current result
    result: i32,
}

/// Implementation of Calculator methods.
impl Calculator {
    /// Create a new calculator.
    fn new() -> Self {
        Self { result: 0 }
    }

    #[doc = "Add a value to the current result."]
    fn add(&mut self, value: i32) {
        self.result += value;
    }

    /**
     * Get the current result.
     * 
     * Returns the accumulated result of all operations.
     */
    fn get_result(&self) -> i32 {
        self.result
    }
}