/// calculate the sum of two numbers
fn add_bad_capitalization(a: i32, b: i32) -> i32 {
    a + b
}

/// Calculate the sum of two numbers
fn add_missing_period(a: i32, b: i32) -> i32 {
    a + b
}

/// this function calculates the difference
fn subtract_bad_style(a: i32, b: i32) -> i32 {
    a - b
}

fn multiply_missing_docs(a: i32, b: i32) -> i32 {
    a * b
}

/// Calculate division
/// This function divides two numbers
fn divide_missing_blank_line(a: i32, b: i32) -> f64 {
    a as f64 / b as f64
}

/// gets the absolute value of a number
fn abs_non_imperative(x: i32) -> i32 {
    if x < 0 { -x } else { x }
}

/// Calculate the power: pow(base, exponent)
fn power_has_signature(base: i32, exponent: u32) -> i32 {
    base.pow(exponent)
}

struct UndocumentedStruct {
    value: i32,
}

/// represents a basic counter
enum UndocumentedEnum {
    Zero,
    One,
    Many(i32),
}