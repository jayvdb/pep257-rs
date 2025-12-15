use std::path::Path;

use log::info;

use crate::{
    parser::{ParseError, RustParser},
    pep257::{Pep257Checker, Violation},
};

/// Main analyzer that combines parsing and checking.
pub(crate) struct RustDocAnalyzer {
    parser: RustParser,
}

/// Implementation of analyzer methods.
impl RustDocAnalyzer {
    /// Create a new analyzer instance.
    pub(crate) fn new() -> Result<Self, ParseError> {
        Ok(Self { parser: RustParser::new()? })
    }

    /// Analyze a Rust file and return all PEP 257 violations.
    pub(crate) fn analyze_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Vec<Violation>, ParseError> {
        info!("Processing file: {}", path.as_ref().display());
        let docstrings = self.parser.parse_file(&path)?;
        let mut violations = Vec::new();

        for docstring in docstrings {
            violations.extend(Pep257Checker::check_docstring(&docstring));
        }

        Ok(violations)
    }

    /// Analyze Rust source code and return all PEP 257 violations.
    #[allow(dead_code)]
    pub(crate) fn analyze_source(&mut self, source: &str) -> Result<Vec<Violation>, ParseError> {
        let docstrings = self.parser.parse_source(source)?;
        let mut violations = Vec::new();

        for docstring in docstrings {
            violations.extend(Pep257Checker::check_docstring(&docstring));
        }

        Ok(violations)
    }
}

/// Unit tests for the analyzer.
#[cfg(test)]
mod tests {
    use super::*;

    /// Test analyzer with properly formatted code.
    #[test]
    fn test_analyze_good_code() {
        let mut analyzer = RustDocAnalyzer::new().unwrap();
        let source = r"
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Represents a point in 2D space.
struct Point {
    x: f64,
    y: f64,
}
";

        let violations = analyzer.analyze_source(source).unwrap();
        assert!(violations.is_empty());
    }

    /// Test analyzer with poorly formatted code.
    #[test]
    fn test_analyze_bad_code() {
        let mut analyzer = RustDocAnalyzer::new().unwrap();
        let source = r"
/// calculate the sum of two numbers
fn add(a: i32, b: i32) -> i32 {
    a + b
}

        pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
";

        let violations = analyzer.analyze_source(source).unwrap();
        assert!(!violations.is_empty());

        // Should have violations for:
        // 1. Missing period in docstring
        // 2. Not properly capitalized
        // 3. Missing docstring for subtract function
        assert!(violations.iter().any(|v| v.rule == "D400")); // Missing period
        assert!(violations.iter().any(|v| v.rule == "D403")); // Not capitalized
        assert!(violations.iter().any(|v| v.rule == "D103")); // Missing function docstring
    }
}
