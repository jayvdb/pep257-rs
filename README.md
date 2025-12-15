# PEP 257 Rust Docstring Checker

A Rust tool that uses tree-sitter to parse Rust files and check that documentation comments (docstrings)
follow Python's [PEP 257 conventions](https://peps.python.org/pep-0257/) as much as possible
within the context of Rust code.

## Features

- **Tree-sitter parsing**: Uses tree-sitter-rust for accurate AST-based
  parsing
- **PEP 257 compliance**: Checks documentation comments against adapted PEP 257
  rules
- **Multiple comment styles**: Supports `///`, `/** */`, and `#[doc = "..."]`
  documentation styles
- **Multiple output formats**: Text and JSON output formats
- **Comprehensive coverage**: Checks functions, structs, enums, traits, impl
  blocks, modules, and constants
- **Command-line interface**: Easy-to-use CLI with various options

## Supported PEP 257 Rules

The tool implements the following PEP 257 rules adapted for Rust:

- **D100**: Missing docstring in public items
- **D201**: No blank lines allowed before function docstring (adapted)
- **D202**: No blank lines allowed after function docstring (adapted)
- **D205**: 1 blank line required between summary line and description
- **D300**: Use consistent comment style (adapted for Rust `///` comments)
- **D301**: Raw string suggestion for backslashes
- **D302**: Unicode content detection
- **D400**: First line should end with a period
- **D401**: First line should be in imperative mood
- **D402**: First line should not be the function's signature
- **D403**: First word should be properly capitalized

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building from source

```bash
git clone <repository-url>
cd pep257
cargo build --release
```

The binary will be available at `target/release/pep257`.

## Usage

### Check a single file

```bash
# Basic usage
pep257 check src/main.rs

# Or using the --file flag
pep257 --file src/main.rs
```

### Check all Rust files in a directory

```bash
# Check current directory
pep257 check-dir .

# Check recursively
pep257 check-dir src --recursive
```

### Show warnings in addition to errors

```bash
pep257 --file src/main.rs --warnings
```

### Output formats

```bash
# Default text format
pep257 --file src/main.rs

# JSON format
pep257 --file src/main.rs --format json
```

### Exit with success even if violations are found

```bash
pep257 --file src/main.rs --no-fail
```

## Example Output

### Text Format

src/main.rs:1:1 error [D403]: First word of the first line should be properly capitalized
src/main.rs:1:1 error [D400]: First line should end with a period
src/main.rs:15:1 error [D100]: Missing docstring in public function

### JSON Format

```json
{
  "file": "src/main.rs",
  "violations": [
    {
      "column": 1,
      "line": 1,
      "message": "First word of the first line should be properly capitalized",
      "rule": "D403",
      "severity": "error"
    },
    {
      "column": 1,
      "line": 1,
      "message": "First line should end with a period",
      "rule": "D400",
      "severity": "error"
    }
  ]
}
```

## Documentation Comment Styles

The tool supports all three Rust documentation comment styles:

### Line comments (`///`)

```rust
/// Calculate the sum of two numbers.
/// 
/// This function takes two integers and returns their sum.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Block comments (`/** */`)

```rust
/**
 * Calculate the sum of two numbers.
 * 
 * This function takes two integers and returns their sum.
 */
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Attribute style (`#[doc]`)

```rust
#[doc = "Calculate the sum of two numbers."]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Multi-line with multiple attributes
#[doc = "Calculate the sum of two numbers."]
#[doc = ""]
#[doc = "This function takes two integers and returns their sum."]
fn add_detailed(a: i32, b: i32) -> i32 {
    a + b
}
```

## Best Practices

Based on PEP 257 adapted for Rust:

1. **Write concise one-line summaries**: The first line should be a brief
   summary ending with a period.

2. **Use imperative mood**: Start with verbs like "Calculate", "Create",
   "Return", not "Calculates" or "This function calculates".

3. **Proper capitalization**: Start the first word with a capital letter.

4. **Separate summary from description**: Use a blank line between the
   summary and detailed description.

5. **Avoid signatures**: Don't repeat the function signature in the docstring.

### Good Examples

```rust
/// Calculate the area of a rectangle.
/// 
/// Takes width and height as parameters and returns the calculated area.
/// Both parameters must be positive numbers.
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

/// Represents a point in 2D space.
struct Point {
    x: f64,
    y: f64,
}
```

### Bad Examples

```rust
/// calculates the area of a rectangle  // Missing period, not capitalized
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

/// This function calculates the area  // Not imperative mood
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

/// calculate_area(width: f64, height: f64) -> f64  // Contains signature
fn calculate_area(width: f64, height: f64) -> f64 {
    width * height
}

fn undocumented_function() {  // Missing docstring
    // This will trigger D100
}
```

## Integration

### CI/CD Integration

You can integrate this tool into your CI/CD pipeline:

```bash
# In your CI script
cargo run --bin pep257 -- check-dir src --recursive
if [ $? -ne 0 ]; then
    echo "Documentation style violations found!"
    exit 1
fi
```

### Pre-commit Hook

Add to your `.git/hooks/pre-commit`:

```bash
#!/bin/sh
cargo run --bin pep257 -- check-dir src --recursive
```

## Limitations

- **Rust-specific adaptation**: Some PEP 257 rules don't directly apply to
  Rust, so they've been adapted
- **Public vs. private**: Currently checks all items, not just public ones
  (this could be enhanced)
- **Module-level docs**: Limited support for module-level documentation
- **Macro documentation**: Does not check documentation for macros

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Built on top of [tree-sitter](https://tree-sitter.github.io/) and
  [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
- Inspired by Python's [pep257](https://pypi.org/project/pep257/) tool
- Follows the [PEP 257](https://www.python.org/dev/peps/pep-0257/) docstring
  conventions where applicable
