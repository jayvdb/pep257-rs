# PEP 257 Docstring Checks

This document describes all the checks implemented in this tool for Rust documentation comments, adapted from Python's PEP 257 conventions.

## Summary Table

| Rule | Severity | Description | Applies To |
|------|----------|-------------|------------|
| D100 | Error | Missing docstring in public module | Modules |
| D101 | Error | Missing docstring in public class | Structs, Enums, Traits, Unions |
| D102 | Error | Missing docstring in public method | Methods |
| D103 | Error | Missing docstring in public function | Functions |
| D104 | Error | Missing docstring in public package | Packages (lib.rs, mod.rs, module files) |
| D106 | Error | Missing docstring in public nested class | Nested structs/enums |
| R101 | Error | Missing docstring in public type alias | Type aliases |
| R102 | Error | Missing docstring in public const/static | Constants, Static variables |
| R103 | Error | Missing docstring in public macro | Macros |
| D201 | Error | No blank lines before docstring | All items |
| D202 | Error | No blank lines after docstring | All items |
| D205 | Error | Blank line between summary and description | All items |
| D400 | Error | First line should end with period | All items |
| D402 | Error | First line should not be signature | Functions |
| D403 | Error | First word should be capitalized | All items |
| D301 | Warning | Consider raw strings for backslashes | Multi-line docstrings |
| D401 | Warning | First line should be imperative mood | All items |
| R401 | Warning | Markdown links with code need backticks | All items |
| R402 | Warning | Common types should use inline code | All items |

## Check Categories

Checks are categorized by severity:
- **Errors**: Must be fixed for proper documentation
- **Warnings**: Recommended improvements, but not strictly required

---

## Errors

### D100: Missing Docstring in Public Module

**Severity**: Error

**Message**: `Missing docstring in public module`

**Description**: Public modules should have documentation comments to outline their purpose and contents.

**Example (Bad)**:
```rust
// src/calculator.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
// src/calculator.rs
//! A simple calculator module.
//!
//! Provides basic arithmetic operations.

pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

### D104: Missing Docstring in Public Package

**Severity**: Error

**Message**: `Missing docstring in public package`

**Description**: Public packages (crate roots and module files) should have documentation comments to outline their purpose and contents. In Rust, this applies to `lib.rs`, `main.rs`, `mod.rs`, and module files like `x.rs` that have a corresponding `x/` directory.

**Example (Bad)**:
```rust
// src/lib.rs
pub mod calculator;
pub mod utils;
```

**Example (Good)**:
```rust
// src/lib.rs
//! A mathematics library.
//!
//! Provides various calculation and utility functions.

pub mod calculator;
pub mod utils;
```

**Another Example (Good)**:
```rust
// src/utils.rs (with src/utils/ directory)
//! Utility functions module.
//!
//! Contains helper functions used throughout the crate.

pub mod helpers;
pub mod validators;
```

---

### D101: Missing Docstring in Public Class

**Severity**: Error

**Message**: `Missing docstring in public {item_type}`

**Description**: Public class-like items (structs, enums, traits, and unions) should have documentation comments to outline their purpose and behavior.

**Example (Bad) - Struct**:
```rust
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

**Example (Good) - Struct**:
```rust
/// Represents a point in 2D space.
pub struct Point {
    pub x: f64,
    pub y: f64,
}
```

**Example (Bad) - Enum**:
```rust
pub enum Status {
    Active,
    Inactive,
}
```

**Example (Good) - Enum**:
```rust
/// Represents the status of an entity.
pub enum Status {
    Active,
    Inactive,
}
```

**Example (Bad) - Trait**:
```rust
pub trait Drawable {
    fn draw(&self);
}
```

**Example (Good) - Trait**:
```rust
/// A trait for objects that can be drawn.
pub trait Drawable {
    fn draw(&self);
}
```

**Example (Bad) - Union**:
```rust
pub union Data {
    pub int_value: i32,
    pub float_value: f32,
}
```

**Example (Good) - Union**:
```rust
/// Represents data that can be interpreted as either integer or float.
pub union Data {
    pub int_value: i32,
    pub float_value: f32,
}
```

---

### D102: Missing Docstring in Public Method

**Severity**: Error

**Message**: `Missing docstring in public method`

**Description**: Public methods in impl blocks should have documentation comments to outline their purpose and behavior.

**Example (Bad)**:
```rust
impl Point {
    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

**Example (Good)**:
```rust
impl Point {
    /// Calculate the distance from the origin.
    pub fn distance(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}
```

---

### D103: Missing Docstring in Public Function

**Severity**: Error

**Message**: `Missing docstring in public function`

**Description**: Public functions should have documentation comments to outline their purpose and behavior.

**Example (Bad)**:
```rust
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
pub fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
```

---

### D106: Missing Docstring in Public Nested Class

**Severity**: Error

**Message**: `Missing docstring in public nested {item_type}`

**Description**: Public nested structs and enums should have documentation comments. Nested items do not inherit the docstring of their enclosing item.

**Example (Bad)**:
```rust
pub struct Outer {
    pub struct Inner {
        value: i32,
    }
}
```

**Example (Good)**:
```rust
/// Outer container.
pub struct Outer {
    /// Inner nested structure.
    pub struct Inner {
        value: i32,
    }
}
```

---

### R101: Missing Docstring in Public Type Alias

**Severity**: Error

**Message**: `Missing docstring in public type alias`

**Description**: Public type aliases should have documentation comments to explain their purpose and usage.

**Example (Bad)**:
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

**Example (Good)**:
```rust
/// A specialized Result type for this crate's operations.
pub type Result<T> = std::result::Result<T, Error>;
```

---

### R102: Missing Docstring in Public Const/Static

**Severity**: Error

**Message**: `Missing docstring in public {const|static}`

**Description**: Public constants and static variables should have documentation comments to explain their purpose and value.

**Example (Bad)**:
```rust
pub const MAX_SIZE: usize = 1024;

pub static GLOBAL_CONFIG: Config = Config::default();
```

**Example (Good)**:
```rust
/// Maximum buffer size in bytes.
pub const MAX_SIZE: usize = 1024;

/// Global configuration instance.
pub static GLOBAL_CONFIG: Config = Config::default();
```

---

### R103: Missing Docstring in Public Macro

**Severity**: Error

**Message**: `Missing docstring in public macro`

**Description**: Public macros should have documentation comments to explain their usage and behavior.

**Example (Bad)**:
```rust
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("ERROR: {}", format_args!($($arg)*));
    };
}
```

**Example (Good)**:
```rust
/// Log an error message to stderr with ERROR prefix.
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("ERROR: {}", format_args!($($arg)*));
    };
}
```

---

### D201: No Blank Lines Before Docstring

**Severity**: Error

**Message**: `No blank lines allowed before {item_type} docstring`

**Description**: Docstrings should start immediately without leading blank lines. This applies to all item types: functions, structs, enums, traits, impl blocks, modules, and constants.

**Applies to**: All items

**Example (Bad)**:
```rust
///
/// Calculate the sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

///
/// Represents a point.
struct Point {
    x: f64,
    y: f64,
}
```

**Example (Good)**:
```rust
/// Calculate the sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point.
struct Point {
    x: f64,
    y: f64,
}
```

---

### D202: No Blank Lines After Docstring

**Severity**: Error

**Message**: `No blank lines allowed after {item_type} docstring`

**Description**: Docstrings should not have trailing blank lines. This applies to all item types: functions, structs, enums, traits, impl blocks, modules, and constants.

**Applies to**: All items

**Example (Bad)**:
```rust
/// Calculate the sum.
///
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point.
///
struct Point {
    x: f64,
    y: f64,
}
```

**Example (Good)**:
```rust
/// Calculate the sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point.
struct Point {
    x: f64,
    y: f64,
}
```

---

### D205: Blank Line Required Between Summary and Description

**Severity**: Error

**Message**: `1 blank line required between summary line and description`

**Description**: Multi-line docstrings should have a blank line separating the summary (first line) from the detailed description.

**Example (Bad)**:
```rust
/// Calculate the sum of two numbers.
/// This function takes two integers and returns their sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
///
/// This function takes two integers and returns their sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Detection Heuristic**: If the first line ends with terminal punctuation (`.`, `!`, or `?`) and is followed immediately by another line of text, D205 is triggered.

---

### D400: First Line Should End With a Period

**Severity**: Error

**Message**: `First line should end with a period`

**Description**: The summary line (first line) of a docstring should end with a period to form a complete sentence.

**Example (Bad)**:
```rust
/// Calculate the sum of two numbers
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

### D402: First Line Should Not Be the Function's Signature

**Severity**: Error

**Message**: `First line should not be the function's signature`

**Description**: The docstring should describe what the function does, not repeat its signature.

**Applies to**: Functions only

**Example (Bad)**:
```rust
/// add(a: i32, b: i32) -> i32
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Add(a: i32, b: i32) -> i32.
fn add_better(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Add two integers and return the result.
fn add_explicit(a: i32, b: i32) -> i32 {
    a + b
}
```

**Detection**: Checks for patterns that look like function signatures:
- Contains parentheses `()` with the return type arrow `->`
- OR starts with lowercase/underscore followed by parentheses (common function naming pattern)
- Markdown links like `[Type](url)` are excluded from this check to avoid false positives

---

### D403: First Word Should Be Properly Capitalized

**Severity**: Error

**Message**: `First word of the first line should be properly capitalized`

**Description**: The first word of the summary line should start with a capital letter.

**Example (Bad)**:
```rust
/// calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## Warnings

### D301: Raw String Suggestion for Backslashes

**Severity**: Warning

**Message**: `Consider using raw strings for docstrings with backslashes`

**Description**: Docstrings containing escaped backslashes (`\\`) should consider using raw string literals to improve readability.

**Applies to**: Multi-line docstrings only

**Example (Bad)**:
```rust
/// Match the pattern: \\d+\\s+\\w+
fn parse_pattern(input: &str) -> Result<()> {
    // ...
}
```

**Example (Good)**:
```rust
/// Match the pattern: \d+\s+\w+
///
/// Uses raw string representation for clarity.
fn parse_pattern(input: &str) -> Result<()> {
    // ...
}
```

**Note**: This is a suggestion; in some contexts escaped backslashes may be necessary.

---

### D401: First Line Should Be in Imperative Mood

**Severity**: Warning

**Message**: `First line should be in imperative mood`

**Description**: The summary line should use imperative mood (command form) rather than descriptive form. Start with verbs like "Calculate", "Return", "Create", not "Calculates", "Returns", or "Creates".

**Example (Bad)**:
```rust
/// Calculates the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// This function returns the result.
fn get_result() -> i32 {
    42
}
```

**Example (Good)**:
```rust
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Return the computed result.
fn get_result() -> i32 {
    42
}
```

**Detection**: Uses the `imperative` crate to check if the first word is in imperative mood. Fallback patterns include detecting:
- Third-person verbs ending in 's' ("creates", "returns")
- Non-imperative starting words ("this", "the", "a", "an", "returns", "gets", "creates", "makes", "builds")

---

### R401: Markdown Links With Code Should Have Backticks

**Severity**: Warning

**Message**: `Markdown link text looks like code but lacks backticks: [X] should be [\`X\`]`

**Description**: When using Markdown links or references to code elements (types, functions), the text should be wrapped in backticks for proper rendering.

**Detection**: Identifies text that looks like code (contains `::` or PascalCase patterns) within square brackets.

**Example (Bad)**:
```rust
/// For use with [SqlType::Custom](crate::SqlType).
fn custom_type() { }

/// Wrapper around a [PrimaryKeyType] to indicate the primary key.
struct Wrapper;
```

**Example (Good)**:
```rust
/// For use with [`SqlType::Custom`](crate::SqlType).
fn custom_type() { }

/// Wrapper around a [`PrimaryKeyType`] to indicate the primary key.
struct Wrapper;
```

**Special Cases**:
- Plain text links like `[documentation](https://example.com)` do not trigger this check
- Text already containing backticks is exempt
- Brackets inside inline code blocks `` `[...]` `` are ignored
- Reference-style link labels `[text][label]` only check the display text, not the label

---

### R402: Common Rust Types Should Use Inline Code

**Severity**: Warning

**Message**: `Use inline code for common Rust type: [Type] should be \`Type\``

**Description**: Common Rust standard library types should use inline code (backticks) instead of Markdown links.

**Common Types Checked**:
- `Option`
- `Result`
- `Vec`
- `Box`
- `Rc`
- `Arc`
- `Some`
- `None`
- `Ok`
- `Err`

**Example (Bad)**:
```rust
/// Returns an [Option] containing the result.
fn get_value() -> Option<i32> {
    Some(42)
}

/// Returns a [Result](std::result::Result) value.
fn try_parse(s: &str) -> Result<i32, ParseError> {
    // ...
}
```

**Example (Good)**:
```rust
/// Returns an `Option` containing the result.
fn get_value() -> Option<i32> {
    Some(42)
}

/// Returns a `Result` value.
fn try_parse(s: &str) -> Result<i32, ParseError> {
    // ...
}
```

**Special Cases**:
- Already backticked references like `` [`Option`] `` or `` `Option` `` are exempt
- Custom types with similar names (e.g., `MyOption`) do not trigger this check
- Only exact matches to the common types list are flagged

---

## Summary Table

| Rule | Severity | Description | Applies To |
|------|----------|-------------|------------|
| D100 | Error | Missing docstring in public module | Modules |
| D101 | Error | Missing docstring in public class | Structs, Enums, Traits |
| D102 | Error | Missing docstring in public method | Methods |
| D103 | Error | Missing docstring in public function | Functions |
| D104 | Error | Missing docstring in public package | Packages (lib.rs, mod.rs, module files) |
| D106 | Error | Missing docstring in public nested class | Nested structs/enums |
| R101 | Error | Missing docstring in public type alias | Type aliases |
| R102 | Error | Missing docstring in public const/static | Constants, Static variables |
| R103 | Error | Missing docstring in public macro | Macros |
| D201 | Error | No blank lines before docstring | All items |
| D202 | Error | No blank lines after docstring | All items |
| D205 | Error | Blank line between summary and description | All items |
| D400 | Error | First line should end with period | All items |
| D402 | Error | First line should not be signature | Functions |
| D403 | Error | First word should be capitalized | All items |
| D301 | Warning | Consider raw strings for backslashes | Multi-line docstrings |
| D401 | Warning | First line should be imperative mood | All items |
| R401 | Warning | Markdown links with code need backticks | All items |
| R402 | Warning | Common types should use inline code | All items |

---

## Running With Warnings

By default, only errors are reported. To see warnings as well, use the `--warnings` flag:

```bash
pep257 --file src/main.rs --warnings
```

---

## Comment Styles Supported

All checks work with these Rust documentation comment styles:

### Line Comments (`///`)
```rust
/// Single line summary.
///
/// Multi-line description continues here.
fn example() { }
```

### Block Comments (`/** */`)
```rust
/**
 * Single line summary.
 * 
 * Multi-line description continues here.
 */
fn example() { }
```

### Attribute Style (`#[doc]`)
```rust
#[doc = "Single line summary."]
#[doc = ""]
#[doc = "Multi-line description continues here."]
fn example() { }
```

---

## Notes on Adaptation from PEP 257

This tool adapts Python's PEP 257 conventions to Rust. Some rules have been modified:

- **D100-D107**: Adapted to Rust item types. D101 covers all class-like types (structs, enums, traits, unions). D104 applies to crate roots (`lib.rs`, `main.rs`) and module files (`mod.rs`, `x.rs` with `x/` directory). D105 (magic methods) and D107 (`__init__`) are not applicable to Rust.
- **D300**: Not implemented; Rust doesn't use triple quotes
- **D201/D202**: Adapted for Rust comment syntax and apply to all item types
- **D301**: Adapted to suggest raw strings for `\\` patterns
- **R101**: New rule for type aliases (Rust-specific item type)
- **R102**: New rule for constants and static variables (Rust-specific item types)
- **R103**: New rule for macros (Rust-specific item type)
- **R401/R402**: New rules specific to Rust documentation practices (Markdown links and common types)

The goal is to maintain the spirit of PEP 257 while respecting Rust's documentation conventions and best practices.

---

## Checks Not Yet Implemented

The following checks from Python linters (Ruff, Pylint) are not yet implemented in this tool:

### Character and Unicode Checks

- **Ambiguous Unicode Characters**: Detection of Unicode characters that could be visually confusing (similar to Ruff's RUF001/RUF002)
- **Bidirectional Unicode**: Control characters that can be used to obfuscate code (similar to Pylint's PLE2502)
- **Invalid Control Characters**: Detection of control characters like backspace, NUL, ESC, SUB, zero-width space (similar to Pylint's PLE2510-2515)
- **Tab Characters in Docstrings**: Detection of tabs in docstring content (similar to pydocstyle's D206)

These checks may be added in future versions to improve code safety and readability.
