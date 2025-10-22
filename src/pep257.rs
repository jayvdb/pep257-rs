use regex::Regex;
use std::fmt;

/// Represents a PEP 257 violation.
#[derive(Debug, Clone)]
pub struct Violation {
    pub rule: String,
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
}

/// Severity level for violations.
#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
}

/// Format a violation for display.
impl fmt::Display for Violation {
    /// Format the violation as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            ":{}:{} {} [{}]: {}", // filename will be added by caller
            self.line,
            self.column,
            match self.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            self.rule,
            self.message
        )
    }
}

/// Represents a docstring found in the code.
#[derive(Debug, Clone)]
pub struct Docstring {
    pub content: String,
    #[allow(dead_code)]
    pub raw_content: String,
    pub line: usize,
    pub column: usize,
    pub is_multiline: bool,
    pub target_type: DocstringTarget,
}

/// Type of construct that has a docstring.
#[derive(Debug, Clone, PartialEq)]
pub enum DocstringTarget {
    Function,
    Struct,
    Enum,
    Module,
    Impl,
    Trait,
    Const,
    #[allow(dead_code)]
    Static,
}

/// Format a docstring target for display.
impl fmt::Display for DocstringTarget {
    /// Format the docstring target as a string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            DocstringTarget::Function => "function",
            DocstringTarget::Struct => "struct",
            DocstringTarget::Enum => "enum",
            DocstringTarget::Module => "module",
            DocstringTarget::Impl => "impl",
            DocstringTarget::Trait => "trait",
            DocstringTarget::Const => "const",
            DocstringTarget::Static => "static",
        };
        write!(f, "{}", name)
    }
}

/// PEP 257 checker implementation.
pub struct Pep257Checker {
    #[allow(dead_code)]
    whitespace_regex: Regex,
    #[allow(dead_code)]
    leading_space_regex: Regex,
}

/// Provide a default checker instance.
impl Default for Pep257Checker {
    /// Return a new checker with default configuration.
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of checker methods.
impl Pep257Checker {
    /// Create a new checker instance.
    pub fn new() -> Self {
        Self {
            whitespace_regex: Regex::new(r"\s+").unwrap(),
            leading_space_regex: Regex::new(r"^\s*").unwrap(),
        }
    }

    /// Check a docstring against PEP 257 rules.
    pub fn check_docstring(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Skip empty docstrings
        if docstring.content.trim().is_empty() {
            violations.push(Violation {
                rule: "D100".to_string(),
                message: format!("Missing docstring in public {}", docstring.target_type),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
            return violations;
        }

        // Check for proper docstring format
        violations.extend(self.check_d200_series(docstring));
        violations.extend(self.check_d300_series(docstring));
        violations.extend(self.check_d400_series(docstring));

        violations
    }

    /// Check D200 series: One-line docstring whitespace issues.
    fn check_d200_series(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let content = &docstring.content;
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return violations;
        }

        let first_line = lines[0];
        let trimmed_first = first_line.trim();

        // D201: No blank lines allowed before function docstring
        if docstring.target_type == DocstringTarget::Function && content.starts_with('\n') {
            violations.push(Violation {
                rule: "D201".to_string(),
                message: "No blank lines allowed before function docstring".to_string(),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D202: No blank lines allowed after function docstring
        if docstring.target_type == DocstringTarget::Function && content.ends_with('\n') {
            violations.push(Violation {
                rule: "D202".to_string(),
                message: "No blank lines allowed after function docstring".to_string(),
                line: docstring.line + lines.len() - 1,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D205: 1 blank line required between summary line and description
        if lines.len() > 2 && !trimmed_first.is_empty() && !lines[1].trim().is_empty() {
            violations.push(Violation {
                rule: "D205".to_string(),
                message: "1 blank line required between summary line and description".to_string(),
                line: docstring.line + 1,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        violations
    }

    /// Check D300 series: Triple double quotes and closing quotes position.
    fn check_d300_series(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = docstring.content.lines().collect();

        if lines.is_empty() {
            return violations;
        }

        // D300: Use """triple double quotes"""
        // Note: In Rust, we adapt this to check that /// comments are used consistently
        // and follow a similar structure to Python docstrings

        // For multiline docstrings, check closing position
        if docstring.is_multiline && lines.len() > 1 {
            let _last_line = lines[lines.len() - 1];

            // D301: Use r""" if any backslashes in a docstring
            // Adapted for Rust: check for excessive escaping
            if docstring.content.contains("\\\\") {
                violations.push(Violation {
                    rule: "D301".to_string(),
                    message: "Consider using raw strings for docstrings with backslashes"
                        .to_string(),
                    line: docstring.line,
                    column: docstring.column,
                    severity: Severity::Warning,
                });
            }

            // D302: Use u""" for Unicode docstrings
            // Less relevant for Rust, but we can check for Unicode content
            if docstring.content.chars().any(|c| c as u32 > 127) {
                violations.push(Violation {
                    rule: "D302".to_string(),
                    message: "Docstring contains Unicode characters".to_string(),
                    line: docstring.line,
                    column: docstring.column,
                    severity: Severity::Warning,
                });
            }
        }

        violations
    }

    /// Check D400 series: First line should be a summary.
    fn check_d400_series(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = docstring.content.lines().collect();

        if lines.is_empty() {
            return violations;
        }

        let first_line = lines[0].trim();

        // D400: First line should end with a period
        if !first_line.is_empty()
            && !first_line.ends_with('.')
            && !first_line.ends_with('!')
            && !first_line.ends_with('?')
        {
            violations.push(Violation {
                rule: "D400".to_string(),
                message: "First line should end with a period".to_string(),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D401: First line should be in imperative mood
        if !first_line.is_empty() && self.is_not_imperative(first_line) {
            violations.push(Violation {
                rule: "D401".to_string(),
                message: "First line should be in imperative mood".to_string(),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Warning,
            });
        }

        // D402: First line should not be the function's signature
        if first_line.contains('(') && first_line.contains(')') {
            violations.push(Violation {
                rule: "D402".to_string(),
                message: "First line should not be the function's signature".to_string(),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D403: First word of the first line should be properly capitalized
        if let Some(first_word) = first_line.split_whitespace().next()
            && !first_word.chars().next().unwrap_or(' ').is_uppercase()
        {
            violations.push(Violation {
                rule: "D403".to_string(),
                message: "First word of the first line should be properly capitalized".to_string(),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        violations
    }

    /// Determine if a line is not in imperative mood using a simple heuristic.
    fn is_not_imperative(&self, line: &str) -> bool {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }

        let first_word = words[0].to_lowercase();

        // Common non-imperative patterns
        let non_imperative_starts = [
            "this", "the", "a", "an", "returns", "return", "gets", "get", "creates", "create",
            "makes", "make", "builds", "build",
        ];

        non_imperative_starts.contains(&first_word.as_str())
    }
}

/// Unit tests for the PEP 257 checker.
#[cfg(test)]
mod tests {
    use super::*;

    /// Test empty docstring detection.
    #[test]
    fn test_empty_docstring() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "".to_string(),
            raw_content: "".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };

        let violations = checker.check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D100");
    }

    /// Test a properly formatted docstring.
    #[test]
    fn test_good_docstring() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Calculate the sum of two numbers.".to_string(),
            raw_content: "/// Calculate the sum of two numbers.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };

        let violations = checker.check_docstring(&docstring);
        assert!(violations.is_empty());
    }

    /// Test missing period detection.
    #[test]
    fn test_missing_period() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Calculate the sum of two numbers".to_string(),
            raw_content: "/// Calculate the sum of two numbers".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };

        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D400"));
    }
}
