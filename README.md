# PEP 257 Rust Docstring Checker

A Rust tool that uses tree-sitter to parse Rust files and check that
documentation comments (docstrings) follow Python's
[PEP 257 conventions](https://peps.python.org/pep-0257/) as much as possible
within the context of Rust code.

## Features

- Uses tree-sitter for accurate AST-based parsing
- Checks documentation against adapted PEP 257 rules
- Supports multiple comment styles: `///`, `/** */`, and `#[doc = "..."]`
- Multiple output formats (text and JSON)
- Checks functions, structs, enums, traits, impl blocks, modules, and constants

## Installation

```bash
cargo install pep257
```

Or install from the repository:

```bash
cargo install --git https://github.com/jayvdb/pep257-rs
```

Or build from source:

```bash
git clone https://github.com/jayvdb/pep257-rs
cd pep257-rs
cargo build --release
```

## Quick Start

```bash
# Check current directory
pep257 check

# Check a specific file
pep257 check src/main.rs

# Check with warnings
pep257 check --warnings

# JSON output
pep257 check --format json
```

For detailed usage and all available options, see [HELP.md](HELP.md).

For a complete list of supported options, run:

```bash
pep257 --help
```

## Example Output

```text
src/main.rs:1:1 error [D403]: First word of the first line should be properly capitalized
src/main.rs:1:1 error [D400]: First line should end with a period
src/main.rs:15:1 error [D100]: Missing docstring in public function
```

## Documentation

- [HELP.md](HELP.md) - Complete command-line usage
- [CHECKS.md](CHECKS.md) - Detailed documentation of all PEP 257 rules, with examples

## CI Integration

```bash
pep257 check src/
```

The tool exits with a non-zero status if violations are found (use
`--no-fail` to override).

## Contributing

Contributions are welcome! Please ensure:

- Tests pass: `cargo test`
- Clippy passes: `cargo clippy`
- Code is formatted and linted: `tools/tidy.sh`

## License

MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [tree-sitter](https://tree-sitter.github.io/) and [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
- Inspired by Python's [pydocstyle](https://github.com/PyCQA/pydocstyle) tool
- Based on [PEP 257](https://peps.python.org/pep-0257/) conventions
