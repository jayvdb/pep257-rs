use imperative::Mood;
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
        violations.extend(self.check_common_rust_types(docstring));

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
        // Only check functions, and avoid false positives from Markdown links [text](url)
        if docstring.target_type == DocstringTarget::Function {
            // Remove Markdown links to avoid false positives
            let without_md_links = self.remove_markdown_links(first_line);

            // Check if it looks like a function signature (has parentheses with possible parameters)
            // and doesn't just contain parentheses for other reasons
            if without_md_links.contains('(') && without_md_links.contains(')') {
                // Additional heuristic: likely a signature if it has -> or starts with a likely function name pattern
                let looks_like_signature = without_md_links.contains("->")
                    || without_md_links
                        .trim_start()
                        .chars()
                        .next()
                        .map(|c| c.is_lowercase() || c == '_')
                        .unwrap_or(false);

                if looks_like_signature {
                    violations.push(Violation {
                        rule: "D402".to_string(),
                        message: "First line should not be the function's signature".to_string(),
                        line: docstring.line,
                        column: docstring.column,
                        severity: Severity::Error,
                    });
                }
            }
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

        // D405: Markdown links with code references should have backticks inside brackets
        violations.extend(self.check_markdown_link_backticks(docstring));

        violations
    }

    /// Determine if a line is not in imperative mood using the imperative crate.
    fn is_not_imperative(&self, line: &str) -> bool {
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.is_empty() {
            return false;
        }

        let first_word = words[0];

        // Use the imperative crate to check if the first word is imperative
        let mood_checker = Mood::new();
        match mood_checker.is_imperative(first_word) {
            Some(true) => false, // It IS imperative, so NOT non-imperative
            Some(false) => true, // It's NOT imperative
            None => {
                // Fallback for words not recognized by the checker
                // Check for common non-imperative patterns
                let first_word_lower = first_word.to_lowercase();
                let non_imperative_starts = [
                    "this", "the", "a", "an", "returns", "gets", "creates", "makes", "builds",
                ];
                non_imperative_starts.contains(&first_word_lower.as_str())
            }
        }
    }

    /// Remove Markdown links from a string to avoid false positives in checks.
    /// Converts "[text](url)" to "text"
    fn remove_markdown_links(&self, text: &str) -> String {
        let mut result = String::new();
        let mut chars = text.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '[' {
                // Collect text until ]
                let mut link_text = String::new();
                let mut found_bracket = false;

                for ch in chars.by_ref() {
                    if ch == ']' {
                        found_bracket = true;
                        break;
                    }
                    link_text.push(ch);
                }

                // Check if followed by (url)
                if found_bracket && chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    // Skip until ')'
                    for ch in chars.by_ref() {
                        if ch == ')' {
                            break;
                        }
                    }
                    // Add just the link text
                    result.push_str(&link_text);
                } else {
                    // Not a markdown link, keep the bracket
                    result.push('[');
                    result.push_str(&link_text);
                    if found_bracket {
                        result.push(']');
                    }
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    /// Check for markdown links that should have backticks inside square brackets.
    /// For example, [SqlType::Custom] should be [`SqlType::Custom`].
    /// This includes both markdown links [text](url) and standalone references [text].
    fn check_markdown_link_backticks(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let content = &docstring.content;

        // Look for text in square brackets: [text] or [text](url)
        let mut chars = content.chars().enumerate().peekable();
        let mut line_num = docstring.line;
        let mut col_num = docstring.column;
        let mut in_backticks = false;

        while let Some((pos, ch)) = chars.next() {
            if ch == '\n' {
                line_num += 1;
                col_num = docstring.column;
                continue;
            } else {
                col_num += 1;
            }

            // Track when we're inside inline code (backticks)
            if ch == '`' {
                in_backticks = !in_backticks;
                continue;
            }

            // Skip checking brackets inside inline code
            if in_backticks {
                continue;
            }

            if ch == '[' {
                // Collect text until ]
                let mut link_text = String::new();
                let mut found_bracket = false;
                let _link_start_pos = pos;
                let link_start_line = line_num;
                let link_start_col = col_num;

                while let Some((_, ch)) = chars.peek() {
                    if *ch == ']' {
                        found_bracket = true;
                        chars.next(); // consume ']'
                        col_num += 1;
                        break;
                    }
                    if *ch == '\n' {
                        line_num += 1;
                        col_num = docstring.column;
                    } else {
                        col_num += 1;
                    }
                    if let Some((_, c)) = chars.next() {
                        link_text.push(c);
                    }
                }

                // Check if this is a markdown reference (with or without URL)
                if found_bracket {
                    let mut is_reference_label = false;

                    // Peek ahead to see if there's a URL or another bracket (reference-style link)
                    while let Some((_, ch)) = chars.peek() {
                        if *ch == '(' {
                            chars.next(); // consume '('
                            col_num += 1;

                            // Skip until ')'
                            loop {
                                match chars.peek() {
                                    Some((_, ')')) => {
                                        chars.next();
                                        col_num += 1;
                                        break;
                                    }
                                    Some((_, '\n')) => {
                                        chars.next();
                                        line_num += 1;
                                        col_num = docstring.column;
                                    }
                                    Some(_) => {
                                        chars.next();
                                        col_num += 1;
                                    }
                                    None => break,
                                }
                            }
                            break;
                        } else if *ch == '[' {
                            // This is a reference-style link: [text][label]
                            // Skip the entire label part
                            chars.next(); // consume '['
                            col_num += 1;

                            // Skip until ']'
                            loop {
                                match chars.peek() {
                                    Some((_, ']')) => {
                                        chars.next();
                                        col_num += 1;
                                        break;
                                    }
                                    Some((_, '\n')) => {
                                        chars.next();
                                        line_num += 1;
                                        col_num = docstring.column;
                                    }
                                    Some(_) => {
                                        chars.next();
                                        col_num += 1;
                                    }
                                    None => break,
                                }
                            }
                            is_reference_label = true;
                            break;
                        } else if !ch.is_whitespace() {
                            // Not followed by URL or label, but still check standalone [text]
                            break;
                        } else {
                            if *ch == '\n' {
                                line_num += 1;
                                col_num = docstring.column;
                            } else {
                                col_num += 1;
                            }
                            chars.next();
                        }
                    }

                    // Skip checking reference labels in reference-style links [text][label]
                    // Only check the display text, not the label
                    if !is_reference_label
                        && self.looks_like_code(&link_text)
                        && !self.has_backticks(&link_text)
                    {
                        violations.push(Violation {
                            rule: "D405".to_string(),
                            message: format!(
                                "Markdown link text looks like code but lacks backticks: [{}] should be [`{}`]",
                                link_text.trim(), link_text.trim()
                            ),
                            line: link_start_line,
                            column: link_start_col,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        violations
    }

    /// Check if text looks like code (contains :: or PascalCase identifiers).
    fn looks_like_code(&self, text: &str) -> bool {
        let trimmed = text.trim();

        // Check for Rust path separator
        if trimmed.contains("::") {
            return true;
        }

        // Check for PascalCase (starts with uppercase, has lowercase)
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_uppercase() {
                // Check if it has a mix of upper and lowercase (PascalCase pattern)
                let has_lower = trimmed.chars().any(|c| c.is_lowercase());
                let has_upper_after_first = trimmed.chars().skip(1).any(|c| c.is_uppercase());
                if has_lower && has_upper_after_first {
                    return true;
                }
            }
        }

        false
    }

    /// Check if text already has backticks.
    fn has_backticks(&self, text: &str) -> bool {
        text.contains('`')
    }

    /// Check for common Rust types that should use backticks instead of markdown links.
    /// D406: Common types like [Option] and [Result] should be `Option` and `Result`.
    fn check_common_rust_types(&self, docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let content = &docstring.content;

        // List of common Rust types that should use inline code instead of markdown links
        let common_types = [
            "Option", "Result", "Vec", "Box", "Rc", "Arc", "Some", "None", "Ok", "Err",
        ];

        // Look for [Type] or [Type](url) patterns
        let mut chars = content.chars().enumerate().peekable();
        let mut line_num = docstring.line;
        let mut col_num = docstring.column;
        let mut in_backticks = false;

        while let Some((_pos, ch)) = chars.next() {
            if ch == '\n' {
                line_num += 1;
                col_num = docstring.column;
                continue;
            } else {
                col_num += 1;
            }

            // Track when we're inside inline code (backticks)
            if ch == '`' {
                in_backticks = !in_backticks;
                continue;
            }

            // Skip checking brackets inside inline code
            if in_backticks {
                continue;
            }

            if ch == '[' {
                let link_start_line = line_num;
                let link_start_col = col_num;
                let mut link_text = String::new();
                let mut found_bracket = false;

                // Collect text until ]
                while let Some((_, ch)) = chars.peek() {
                    if *ch == ']' {
                        found_bracket = true;
                        chars.next(); // consume ']'
                        col_num += 1;
                        break;
                    }
                    if *ch == '\n' {
                        line_num += 1;
                        col_num = docstring.column;
                    } else {
                        col_num += 1;
                    }
                    if let Some((_, c)) = chars.next() {
                        link_text.push(c);
                    }
                }

                if found_bracket {
                    let trimmed_text = link_text.trim();

                    // Skip if already has backticks
                    if self.has_backticks(trimmed_text) {
                        continue;
                    }

                    // Check if it's a common Rust type (exact match)
                    if common_types.contains(&trimmed_text) {
                        // Peek ahead to see if followed by ( or [, but warn either way
                        let mut has_url_or_ref = false;
                        while let Some((_, ch)) = chars.peek() {
                            if *ch == '(' {
                                // [Type](url) format - consume it
                                chars.next(); // consume '('
                                col_num += 1;
                                loop {
                                    match chars.peek() {
                                        Some((_, ')')) => {
                                            chars.next();
                                            col_num += 1;
                                            break;
                                        }
                                        Some((_, '\n')) => {
                                            chars.next();
                                            line_num += 1;
                                            col_num = docstring.column;
                                        }
                                        Some(_) => {
                                            chars.next();
                                            col_num += 1;
                                        }
                                        None => break,
                                    }
                                }
                                has_url_or_ref = true;
                                break;
                            } else if *ch == '[' {
                                // [Type][ref] format - consume the reference
                                chars.next(); // consume '['
                                col_num += 1;
                                loop {
                                    match chars.peek() {
                                        Some((_, ']')) => {
                                            chars.next();
                                            col_num += 1;
                                            break;
                                        }
                                        Some((_, '\n')) => {
                                            chars.next();
                                            line_num += 1;
                                            col_num = docstring.column;
                                        }
                                        Some(_) => {
                                            chars.next();
                                            col_num += 1;
                                        }
                                        None => break,
                                    }
                                }
                                has_url_or_ref = true;
                                break;
                            } else if !ch.is_whitespace() {
                                break;
                            } else {
                                if *ch == '\n' {
                                    line_num += 1;
                                    col_num = docstring.column;
                                } else {
                                    col_num += 1;
                                }
                                chars.next();
                            }
                        }

                        violations.push(Violation {
                            rule: "D406".to_string(),
                            message: format!(
                                "Use inline code for common Rust type: [{}]{} should be `{}`",
                                trimmed_text,
                                if has_url_or_ref { "(...)" } else { "" },
                                trimmed_text
                            ),
                            line: link_start_line,
                            column: link_start_col,
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        violations
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

    /// D401: "Create" should be considered imperative mood
    #[test]
    fn test_d401_create_is_imperative() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Create a migration.".to_string(),
            raw_content: "/// Create a migration.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };

        let violations = checker.check_docstring(&docstring);
        // Should NOT trigger D401 because "Create" is imperative
        assert!(!violations.iter().any(|v| v.rule == "D401"));
    }

    /// D401: "Creates" should be non-imperative
    #[test]
    fn test_d401_creates_is_not_imperative() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Creates a migration.".to_string(),
            raw_content: "/// Creates a migration.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };

        let violations = checker.check_docstring(&docstring);
        // Should trigger D401 because "Creates" is third person, not imperative
        assert!(violations.iter().any(|v| v.rule == "D401"));
    }

    /// D401: Common imperative verbs should pass
    #[test]
    fn test_d401_common_imperatives() {
        let checker = Pep257Checker::new();
        let imperatives = vec![
            "Return the value.",
            "Calculate the sum.",
            "Get the result.",
            "Set the value.",
            "Add two numbers.",
            "Remove the item.",
        ];

        for content in imperatives {
            let docstring = Docstring {
                content: content.to_string(),
                raw_content: format!("/// {}", content),
                line: 1,
                column: 1,
                is_multiline: false,
                target_type: DocstringTarget::Function,
            };
            let violations = checker.check_docstring(&docstring);
            assert!(
                !violations.iter().any(|v| v.rule == "D401"),
                "Failed for: {}",
                content
            );
        }
    }

    /// Test remove_markdown_links helper
    #[test]
    fn test_remove_markdown_links() {
        let checker = Pep257Checker::new();
        let input = "For use with [SqlType::Custom](crate::SqlType).";
        let expected = "For use with SqlType::Custom.";
        let output = checker.remove_markdown_links(input);
        assert_eq!(output, expected);

        let input2 = "No links here.";
        assert_eq!(checker.remove_markdown_links(input2), input2);

        let input3 = "Multiple [A](x) and [B](y) links.";
        let expected3 = "Multiple A and B links.";
        assert_eq!(checker.remove_markdown_links(input3), expected3);
    }

    /// D402: Should NOT trigger on markdown link docstring
    #[test]
    fn test_d402_no_false_positive_markdown_link() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "For use with [SqlType::Custom](crate::SqlType).".to_string(),
            raw_content: "/// For use with [SqlType::Custom](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D402"));
    }

    /// D402: Should trigger on actual function signature
    #[test]
    fn test_d402_true_positive_signature() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "my_func(x: i32, y: i32) -> i32".to_string(),
            raw_content: "/// my_func(x: i32, y: i32) -> i32".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D402"));
    }

    /// D405: Markdown link with code reference should have backticks
    #[test]
    fn test_d405_markdown_link_without_backticks() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "For use with [SqlType::Custom](crate::SqlType).".to_string(),
            raw_content: "/// For use with [SqlType::Custom](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D405"));
        let d405_violation = violations.iter().find(|v| v.rule == "D405").unwrap();
        assert!(d405_violation.message.contains("SqlType::Custom"));
    }

    /// D405: Markdown link with backticks should not trigger
    #[test]
    fn test_d405_markdown_link_with_backticks() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "For use with [`SqlType::Custom`](crate::SqlType).".to_string(),
            raw_content: "/// For use with [`SqlType::Custom`](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D405"));
    }

    /// D405: Markdown link with plain text should not trigger
    #[test]
    fn test_d405_markdown_link_plain_text() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "See the [documentation](https://example.com) for details.".to_string(),
            raw_content: "/// See the [documentation](https://example.com) for details."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D405"));
    }

    /// D405: Markdown link with PascalCase should trigger
    #[test]
    fn test_d405_markdown_link_pascalcase() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns a [MyType](crate::MyType) instance.".to_string(),
            raw_content: "/// Returns a [MyType](crate::MyType) instance.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D405"));
    }

    /// D405: Standalone bracket reference without URL should trigger
    #[test]
    fn test_d405_standalone_bracket_reference() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Wrapper around a [PrimaryKeyType] to indicate the primary key.".to_string(),
            raw_content: "/// Wrapper around a [PrimaryKeyType] to indicate the primary key."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D405"));
        let d405_violation = violations.iter().find(|v| v.rule == "D405").unwrap();
        assert!(d405_violation.message.contains("PrimaryKeyType"));
    }

    /// D405: Standalone backticked link should NOT trigger
    #[test]
    fn test_d405_standalone_backticked_link() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Where [`Self`] is a [`Migrations`](crate::migrations::Migrations)."
                .to_string(),
            raw_content: "/// Where [`Self`] is a [`Migrations`](crate::migrations::Migrations)."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D405"));
    }

    /// D405: Reference-style link label should NOT trigger
    #[test]
    fn test_d405_reference_style_link_label() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "[`Migrations`][crate::migrations::Migrations].".to_string(),
            raw_content: "/// [`Migrations`][crate::migrations::Migrations].".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        // Should not trigger on the label part [crate::migrations::Migrations]
        assert!(!violations.iter().any(|v| v.rule == "D405"));
    }

    /// D405: Brackets inside inline code should NOT trigger
    #[test]
    fn test_d405_inside_backticks() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Test with attribute macro `#[butane::model]`.".to_string(),
            raw_content: "/// Test with attribute macro `#[butane::model]`.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D405"));
    }

    /// D406: Standalone [Option] should trigger
    #[test]
    fn test_d406_option_standalone() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns an [Option] containing the result.".to_string(),
            raw_content: "/// Returns an [Option] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D406"));
        let d406_violation = violations.iter().find(|v| v.rule == "D406").unwrap();
        assert!(d406_violation.message.contains("Option"));
    }

    /// D406: [Result] with URL should trigger
    #[test]
    fn test_d406_result_with_url() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns a [Result](std::result::Result) value.".to_string(),
            raw_content: "/// Returns a [Result](std::result::Result) value.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D406"));
    }

    /// D406: Backticked [`Option`] should NOT trigger
    #[test]
    fn test_d406_option_with_backticks() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns an [`Option`] containing the result.".to_string(),
            raw_content: "/// Returns an [`Option`] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D406"));
    }

    /// D406: Inline code `Option` should NOT trigger
    #[test]
    fn test_d406_inline_code() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns an `Option` containing the result.".to_string(),
            raw_content: "/// Returns an `Option` containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D406"));
    }

    /// D406: Multiple common types should trigger for each
    #[test]
    fn test_d406_multiple_types() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns [Option] or [Result] or [Vec].".to_string(),
            raw_content: "/// Returns [Option] or [Result] or [Vec].".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        let d406_violations: Vec<_> = violations.iter().filter(|v| v.rule == "D406").collect();
        assert_eq!(d406_violations.len(), 3);
    }

    /// D406: Custom type [MyOption] should NOT trigger
    #[test]
    fn test_d406_custom_type() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Returns a [MyOption] containing the result.".to_string(),
            raw_content: "/// Returns a [MyOption] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D406"));
    }

    /// D406: Brackets inside inline code should NOT trigger
    #[test]
    fn test_d406_inside_backticks() {
        let checker = Pep257Checker::new();
        let docstring = Docstring {
            content: "Use `[Option]` or `[Result]` in inline code.".to_string(),
            raw_content: "/// Use `[Option]` or `[Result]` in inline code.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            target_type: DocstringTarget::Function,
        };
        let violations = checker.check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D406"));
    }
}
