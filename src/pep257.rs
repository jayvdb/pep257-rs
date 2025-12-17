use std::fmt;

use imperative::Mood;
use regex::Regex;

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
            "{}:{} {} [{}]: {}", // filename will be added by caller
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
pub(crate) struct Docstring {
    pub content: String,
    #[allow(dead_code)]
    pub raw_content: String,
    pub line: usize,
    pub column: usize,
    pub is_multiline: bool,
    pub is_public: bool,
    pub target_type: DocstringTarget,
}

/// Type of construct that has a docstring.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum DocstringTarget {
    Function,
    Struct,
    Enum,
    Module,
    Package,
    Impl,
    Trait,
    Const,
    #[allow(dead_code)]
    Static,
    TypeAlias,
    Macro,
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
            DocstringTarget::Package => "package",
            DocstringTarget::Impl => "impl",
            DocstringTarget::Trait => "trait",
            DocstringTarget::Const => "const",
            DocstringTarget::Static => "static",
            DocstringTarget::TypeAlias => "type alias",
            DocstringTarget::Macro => "macro",
        };
        write!(f, "{name}")
    }
}

/// PEP 257 checker implementation.
pub(crate) struct Pep257Checker {
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
    pub(crate) fn new() -> Self {
        Self {
            whitespace_regex: Regex::new(r"\s+").unwrap(),
            leading_space_regex: Regex::new(r"^\s*").unwrap(),
        }
    }

    /// Check a docstring against PEP 257 rules.
    pub(crate) fn check_docstring(docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();

        // Skip empty docstrings
        if docstring.content.trim().is_empty() && docstring.is_public {
            let (rule_code, item_description) =
                Self::get_missing_docstring_rule(docstring.target_type);
            violations.push(Violation {
                rule: rule_code,
                message: format!("Missing docstring in public {item_description}"),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
            return violations;
        }

        // Check for proper docstring format
        violations.extend(Self::check_d200_series(docstring));
        violations.extend(Self::check_d300_series(docstring));
        violations.extend(Self::check_d400_series(docstring));
        violations.extend(Self::check_common_rust_types(docstring));

        violations
    }

    /// Check D200 series: One-line docstring whitespace issues.
    fn check_d200_series(docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let content = &docstring.content;
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return violations;
        }

        // D201: No blank lines allowed before docstring
        if content.starts_with('\n') {
            violations.push(Violation {
                rule: "D201".to_string(),
                message: format!(
                    "No blank lines allowed before {} docstring",
                    docstring.target_type
                ),
                line: docstring.line,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D202: No blank lines allowed after docstring
        if content.ends_with('\n') {
            violations.push(Violation {
                rule: "D202".to_string(),
                message: format!(
                    "No blank lines allowed after {} docstring",
                    docstring.target_type
                ),
                line: docstring.line + lines.len() - 1,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D205: 1 blank line required between summary paragraph and description
        // Find the end of the summary paragraph (first blank line separates paragraphs)
        let mut summary_end_index = None::<usize>;
        for (line_index, line_contents) in lines.iter().enumerate() {
            if line_contents.trim().is_empty() {
                // summary paragraph ends at the previous non-empty line
                summary_end_index = Some(if line_index == 0 { 0 } else { line_index - 1 });
                break;
            }
        }

        if let Some(summary_end_index) = summary_end_index {
            // There is a blank line; ensure that the line after the summary is blank (it will be)
            if summary_end_index + 1 < lines.len()
                && !lines[summary_end_index + 1].trim().is_empty()
            {
                // No blank line separating summary and description
                violations.push(Violation {
                    rule: "D205".to_string(),
                    message: "1 blank line required between summary line and description"
                        .to_string(),
                    line: docstring.line + summary_end_index + 1,
                    column: docstring.column,
                    severity: Severity::Error,
                });
            }
        } else {
            // No blank line found. If there's more than one non-empty line, we need to decide
            // whether it's a wrapped summary (allowed) or a summary followed immediately by a
            // description (should be flagged). Heuristic: if the FIRST non-empty line ends with
            // terminal punctuation (., !, ?) and there is a subsequent non-empty line, then
            // treat that subsequent line as a description that must be separated by a blank line.
            let non_empty_lines: Vec<&str> =
                lines.iter().filter(|l| !l.trim().is_empty()).copied().collect();
            if non_empty_lines.len() > 1
                && let Some(first) = non_empty_lines.first().map(|l| l.trim())
                && (first.ends_with('.') || first.ends_with('!') || first.ends_with('?'))
            {
                // Missing blank line between summary and description
                violations.push(Violation {
                    rule: "D205".to_string(),
                    message: "1 blank line required between summary line and description"
                        .to_string(),
                    line: docstring.line + 1,
                    column: docstring.column,
                    severity: Severity::Error,
                });
            }
        }

        violations
    }

    /// Check D300 series: Triple double quotes and closing quotes position.
    fn check_d300_series(docstring: &Docstring) -> Vec<Violation> {
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
        }

        violations
    }

    /// Check D400 series: First line should be a summary.
    fn check_d400_series(docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let lines: Vec<&str> = docstring.content.lines().collect();

        if lines.is_empty() {
            return violations;
        }

        // Find the first non-empty line to treat as the start of the summary
        let mut first_non_empty_idx = 0usize;
        for (i, l) in lines.iter().enumerate() {
            if !l.trim().is_empty() {
                first_non_empty_idx = i;
                break;
            }
        }

        let first_line = lines[first_non_empty_idx].trim();

        // D400: Check that the first non-empty line (the summary) ends with a period.
        if !first_line.is_empty() && !first_line.ends_with('.') {
            violations.push(Violation {
                rule: "D400".to_string(),
                message: "First line should end with a period".to_string(),
                line: docstring.line + first_non_empty_idx,
                column: docstring.column,
                severity: Severity::Error,
            });
        }

        // D401: First line should be in imperative mood
        if !first_line.is_empty() && Self::is_not_imperative(first_line) {
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
            let without_md_links = Self::remove_markdown_links(first_line);

            // Check if it looks like a function signature (has parentheses with
            // possible parameters) and doesn't just contain parentheses for
            // other reasons
            if without_md_links.contains('(') && without_md_links.contains(')') {
                // Additional heuristic: likely a signature if it has -> or
                // starts with a likely function name pattern
                let looks_like_signature = without_md_links.contains("->")
                    || without_md_links
                        .trim_start()
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_lowercase() || c == '_');

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

        // R401: Markdown links with code references should have backticks inside brackets
        violations.extend(Self::check_markdown_link_backticks(docstring));

        violations
    }

    /// Get the appropriate rule code and description for a missing docstring based on target type.
    fn get_missing_docstring_rule(target_type: DocstringTarget) -> (String, &'static str) {
        match target_type {
            DocstringTarget::Module => ("D100".to_string(), "module"),
            DocstringTarget::Package => ("D104".to_string(), "package"),
            DocstringTarget::Struct => ("D101".to_string(), "struct"),
            DocstringTarget::Enum => ("D101".to_string(), "enum"),
            DocstringTarget::Trait => ("D101".to_string(), "trait"),
            DocstringTarget::Function => ("D103".to_string(), "function"),
            DocstringTarget::Impl => ("D102".to_string(), "method"),
            DocstringTarget::Const => ("R102".to_string(), "const"),
            DocstringTarget::Static => ("R102".to_string(), "static"),
            DocstringTarget::TypeAlias => ("R101".to_string(), "type alias"),
            DocstringTarget::Macro => ("R103".to_string(), "macro"),
        }
    }

    /// Determine if a line is not in imperative mood using the imperative crate.
    fn is_not_imperative(line: &str) -> bool {
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
                let non_imperative_starts =
                    ["this", "the", "a", "an", "returns", "gets", "creates", "makes", "builds"];
                non_imperative_starts.contains(&first_word_lower.as_str())
            }
        }
    }

    /// Remove Markdown links from a string to avoid false positives in checks.
    ///
    /// Converts `[text](url)` to "text".
    fn remove_markdown_links(text: &str) -> String {
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
    ///
    /// This includes both markdown links `[text](url)` and standalone references `[text]`.
    fn check_markdown_link_backticks(docstring: &Docstring) -> Vec<Violation> {
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
            }
            col_num += 1;

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
                let _ = pos;
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
                        }
                        if *ch == '\n' {
                            line_num += 1;
                            col_num = docstring.column;
                        } else {
                            col_num += 1;
                        }
                        chars.next();
                    }

                    // Skip checking reference labels in reference-style links [text][label]
                    // Only check the display text, not the label
                    if !is_reference_label
                        && Self::looks_like_code(&link_text)
                        && !Self::has_backticks(&link_text)
                    {
                        violations.push(Violation {
                            rule: "R401".to_string(),
                            message: format!(
                                concat!(
                                    "Markdown link text looks like code but lacks ",
                                    "backticks: [{}] should be [`{}`]"
                                ),
                                link_text.trim(),
                                link_text.trim()
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
    fn looks_like_code(text: &str) -> bool {
        let trimmed = text.trim();

        // Check for Rust path separator
        if trimmed.contains("::") {
            return true;
        }

        // Check for PascalCase (starts with uppercase, has lowercase)
        if let Some(first_char) = trimmed.chars().next()
            && first_char.is_uppercase()
        {
            // Check if it has a mix of upper and lowercase (PascalCase pattern)
            let has_lower = trimmed.chars().any(char::is_lowercase);
            let has_upper_after_first = trimmed.chars().skip(1).any(char::is_uppercase);
            if has_lower && has_upper_after_first {
                return true;
            }
        }

        false
    }

    /// Check if text already has backticks.
    fn has_backticks(text: &str) -> bool {
        text.contains('`')
    }

    /// Check for common Rust types that should use backticks instead of markdown links.
    ///
    /// R402: Common types like [Option] and [Result] should be `Option` and `Result`.
    fn check_common_rust_types(docstring: &Docstring) -> Vec<Violation> {
        let mut violations = Vec::new();
        let content = &docstring.content;

        // List of common Rust types that should use inline code instead of markdown links
        let common_types =
            ["Option", "Result", "Vec", "Box", "Rc", "Arc", "Some", "None", "Ok", "Err"];

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
            }
            col_num += 1;

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
                    if Self::has_backticks(trimmed_text) {
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
                            }
                            if *ch == '\n' {
                                line_num += 1;
                                col_num = docstring.column;
                            } else {
                                col_num += 1;
                            }
                            chars.next();
                        }

                        violations.push(Violation {
                            rule: "R402".to_string(),
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
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            // This test verifies that D103 is reported for public functions
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D103");
    }

    /// Test that empty docstring for a private function does NOT trigger D103
    #[test]
    fn test_empty_docstring_private_no_d103() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        // Private functions should not trigger D103 for missing docstrings
        assert!(!violations.iter().any(|v| v.rule == "D103"));
    }

    /// Test empty docstring detection for module (D100)
    #[test]
    fn test_empty_docstring_module() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Module,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D100");
        assert!(violations[0].message.contains("module"));
    }

    /// Test empty docstring detection for struct (D101)
    #[test]
    fn test_empty_docstring_struct() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Struct,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D101");
        assert!(violations[0].message.contains("struct"));
    }

    /// Test empty docstring detection for enum (D101)
    #[test]
    fn test_empty_docstring_enum() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Enum,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D101");
        assert!(violations[0].message.contains("enum"));
    }

    /// Test empty docstring detection for trait (D101)
    #[test]
    fn test_empty_docstring_trait() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Trait,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D101");
        assert!(violations[0].message.contains("trait"));
    }

    /// Test empty docstring detection for method (D102)
    #[test]
    fn test_empty_docstring_method() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Impl,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D102");
        assert!(violations[0].message.contains("method"));
    }

    /// Test empty docstring detection for const (R102)
    #[test]
    fn test_empty_docstring_const() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Const,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "R102");
        assert!(violations[0].message.contains("const"));
    }

    /// Test empty docstring detection for static (R102)
    #[test]
    fn test_empty_docstring_static() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Static,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "R102");
        assert!(violations[0].message.contains("static"));
    }

    /// Test empty docstring detection for type alias (R101)
    #[test]
    fn test_empty_docstring_type_alias() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::TypeAlias,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "R101");
        assert!(violations[0].message.contains("type alias"));
    }

    /// Test empty docstring detection for macro (R103)
    #[test]
    fn test_empty_docstring_macro() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Macro,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "R103");
        assert!(violations[0].message.contains("macro"));
    }

    /// Test empty docstring detection for package (D104)
    #[test]
    fn test_empty_docstring_package() {
        let docstring = Docstring {
            content: String::new(),
            raw_content: String::new(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Package,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].rule, "D104");
        assert!(violations[0].message.contains("package"));
    }

    /// Test a properly formatted docstring.
    #[test]
    fn test_good_docstring() {
        let docstring = Docstring {
            content: "Calculate the sum of two numbers.".to_string(),
            raw_content: "/// Calculate the sum of two numbers.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.is_empty());
    }

    /// Test missing period detection.
    #[test]
    fn test_missing_period() {
        let docstring = Docstring {
            content: "Calculate the sum of two numbers".to_string(),
            raw_content: "/// Calculate the sum of two numbers".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D400"));
    }

    /// D401: "Create" should be considered imperative mood
    #[test]
    fn test_d401_create_is_imperative() {
        let docstring = Docstring {
            content: "Create a migration.".to_string(),
            raw_content: "/// Create a migration.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        // Should NOT trigger D401 because "Create" is imperative
        assert!(!violations.iter().any(|v| v.rule == "D401"));
    }

    /// D401: "Creates" should be non-imperative
    #[test]
    fn test_d401_creates_is_not_imperative() {
        let docstring = Docstring {
            content: "Creates a migration.".to_string(),
            raw_content: "/// Creates a migration.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        // Should trigger D401 because "Creates" is third person, not imperative
        assert!(violations.iter().any(|v| v.rule == "D401"));
    }

    /// D401: Common imperative verbs should pass
    #[test]
    fn test_d401_common_imperatives() {
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
                raw_content: format!("/// {content}"),
                line: 1,
                column: 1,
                is_multiline: false,
                is_public: false,
                target_type: DocstringTarget::Function,
            };
            let violations = Pep257Checker::check_docstring(&docstring);
            assert!(!violations.iter().any(|v| v.rule == "D401"), "Failed for: {content}");
        }
    }

    /// Test remove_markdown_links helper
    #[test]
    fn test_remove_markdown_links() {
        let input = "For use with [SqlType::Custom](crate::SqlType).";
        let expected = "For use with SqlType::Custom.";
        let output = Pep257Checker::remove_markdown_links(input);
        assert_eq!(output, expected);

        let input2 = "No links here.";
        assert_eq!(Pep257Checker::remove_markdown_links(input2), input2);

        let input3 = "Multiple [A](x) and [B](y) links.";
        let expected3 = "Multiple A and B links.";
        assert_eq!(Pep257Checker::remove_markdown_links(input3), expected3);
    }

    /// D402: Should NOT trigger on markdown link docstring
    #[test]
    fn test_d402_no_false_positive_markdown_link() {
        let docstring = Docstring {
            content: "For use with [SqlType::Custom](crate::SqlType).".to_string(),
            raw_content: "/// For use with [SqlType::Custom](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D402"));
    }

    /// D402: Should trigger on actual function signature
    #[test]
    fn test_d402_true_positive_signature() {
        let docstring = Docstring {
            content: "my_func(x: i32, y: i32) -> i32".to_string(),
            raw_content: "/// my_func(x: i32, y: i32) -> i32".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D402"));
    }

    /// D402: Capitalized signature should still trigger D402
    #[test]
    fn test_d402_capitalized_signature() {
        let docstring = Docstring {
            content: "Add(a: i32, b: i32) -> i32.".to_string(),
            raw_content: "/// Add(a: i32, b: i32) -> i32.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        // Should trigger D402 because it's a signature pattern with ->
        assert!(violations.iter().any(|v| v.rule == "D402"));
    }

    /// R401: Markdown link with code reference should have backticks
    #[test]
    fn test_r401_markdown_link_without_backticks() {
        let docstring = Docstring {
            content: "For use with [SqlType::Custom](crate::SqlType).".to_string(),
            raw_content: "/// For use with [SqlType::Custom](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "R401"));
        let r401_violation = violations.iter().find(|v| v.rule == "R401").unwrap();
        assert!(r401_violation.message.contains("SqlType::Custom"));
    }

    /// R401: Markdown link with backticks should not trigger
    #[test]
    fn test_r401_markdown_link_with_backticks() {
        let docstring = Docstring {
            content: "For use with [`SqlType::Custom`](crate::SqlType).".to_string(),
            raw_content: "/// For use with [`SqlType::Custom`](crate::SqlType).".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R401"));
    }

    /// R401: Markdown link with plain text should not trigger
    #[test]
    fn test_r401_markdown_link_plain_text() {
        let docstring = Docstring {
            content: "See the [documentation](https://example.com) for details.".to_string(),
            raw_content: "/// See the [documentation](https://example.com) for details."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R401"));
    }

    /// R401: Markdown link with PascalCase should trigger
    #[test]
    fn test_r401_markdown_link_pascalcase() {
        let docstring = Docstring {
            content: "Returns a [MyType](crate::MyType) instance.".to_string(),
            raw_content: "/// Returns a [MyType](crate::MyType) instance.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "R401"));
    }

    /// R401: Standalone bracket reference without URL should trigger
    #[test]
    fn test_r401_standalone_bracket_reference() {
        let docstring = Docstring {
            content: "Wrapper around a [PrimaryKeyType] to indicate the primary key.".to_string(),
            raw_content: "/// Wrapper around a [PrimaryKeyType] to indicate the primary key."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "R401"));
        let r401_violation = violations.iter().find(|v| v.rule == "R401").unwrap();
        assert!(r401_violation.message.contains("PrimaryKeyType"));
    }

    /// R401: Standalone backticked link should NOT trigger
    #[test]
    fn test_r401_standalone_backticked_link() {
        let docstring = Docstring {
            content: "Where [`Self`] is a [`Migrations`](crate::migrations::Migrations)."
                .to_string(),
            raw_content: "/// Where [`Self`] is a [`Migrations`](crate::migrations::Migrations)."
                .to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R401"));
    }

    /// R401: Reference-style link label should NOT trigger
    #[test]
    fn test_r401_reference_style_link_label() {
        let docstring = Docstring {
            content: "[`Migrations`][crate::migrations::Migrations].".to_string(),
            raw_content: "/// [`Migrations`][crate::migrations::Migrations].".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        // Should not trigger on the label part [crate::migrations::Migrations]
        assert!(!violations.iter().any(|v| v.rule == "R401"));
    }

    /// R401: Brackets inside inline code should NOT trigger
    #[test]
    fn test_r401_inside_backticks() {
        let docstring = Docstring {
            content: "Test with attribute macro `#[butane::model]`.".to_string(),
            raw_content: "/// Test with attribute macro `#[butane::model]`.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R401"));
    }

    /// R402: Standalone [Option] should trigger
    #[test]
    fn test_r402_option_standalone() {
        let docstring = Docstring {
            content: "Returns an [Option] containing the result.".to_string(),
            raw_content: "/// Returns an [Option] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "R402"));
        let r402_violation = violations.iter().find(|v| v.rule == "R402").unwrap();
        assert!(r402_violation.message.contains("Option"));
    }

    /// R402: [Result] with URL should trigger
    #[test]
    fn test_r402_result_with_url() {
        let docstring = Docstring {
            content: "Returns a [Result](std::result::Result) value.".to_string(),
            raw_content: "/// Returns a [Result](std::result::Result) value.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "R402"));
    }

    /// R402: Backticked [`Option`] should NOT trigger
    #[test]
    fn test_r402_option_with_backticks() {
        let docstring = Docstring {
            content: "Returns an [`Option`] containing the result.".to_string(),
            raw_content: "/// Returns an [`Option`] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R402"));
    }

    /// R402: Inline code `Option` should NOT trigger
    #[test]
    fn test_r402_inline_code() {
        let docstring = Docstring {
            content: "Returns an `Option` containing the result.".to_string(),
            raw_content: "/// Returns an `Option` containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R402"));
    }

    /// R402: Multiple common types should trigger for each
    #[test]
    fn test_r402_multiple_types() {
        let docstring = Docstring {
            content: "Returns [Option] or [Result] or [Vec].".to_string(),
            raw_content: "/// Returns [Option] or [Result] or [Vec].".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        let r402_violations: Vec<_> = violations.iter().filter(|v| v.rule == "R402").collect();
        assert_eq!(r402_violations.len(), 3);
    }

    /// R402: Custom type [MyOption] should NOT trigger
    #[test]
    fn test_r402_custom_type() {
        let docstring = Docstring {
            content: "Returns a [MyOption] containing the result.".to_string(),
            raw_content: "/// Returns a [MyOption] containing the result.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R402"));
    }

    /// R402: Brackets inside inline code should NOT trigger
    #[test]
    fn test_r402_inside_backticks() {
        let docstring = Docstring {
            content: "Use `[Option]` or `[Result]` in inline code.".to_string(),
            raw_content: "/// Use `[Option]` or `[Result]` in inline code.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: false,
            target_type: DocstringTarget::Function,
        };
        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "R402"));
    }

    /// Test Display implementation for Violation with Error severity
    /// Test Display implementation for Violation with Error severity
    #[test]
    fn test_violation_display_error() {
        let violation = Violation {
            rule: "D400".to_string(),
            message: "First line should end with a period".to_string(),
            line: 42,
            column: 5,
            severity: Severity::Error,
        };

        let formatted = format!("{violation}");
        assert_eq!(formatted, "42:5 error [D400]: First line should end with a period");
    }

    /// Test Display implementation for Violation with Warning severity
    #[test]
    fn test_violation_display_warning() {
        let violation = Violation {
            rule: "D401".to_string(),
            message: "First line should be in imperative mood".to_string(),
            line: 10,
            column: 1,
            severity: Severity::Warning,
        };

        let formatted = format!("{violation}");
        assert_eq!(formatted, "10:1 warning [D401]: First line should be in imperative mood");
    }

    /// Test Display implementation with multi-digit line and column numbers
    #[test]
    fn test_violation_display_large_numbers() {
        let violation = Violation {
            rule: "D205".to_string(),
            message: "1 blank line required between summary line and description".to_string(),
            line: 1234,
            column: 567,
            severity: Severity::Error,
        };

        let formatted = format!("{violation}");
        assert_eq!(
            formatted,
            "1234:567 error [D205]: 1 blank line required between summary line and description"
        );
    }

    /// Test Display implementation with special characters in message
    #[test]
    fn test_violation_display_special_chars() {
        let violation = Violation {
            rule: "R401".to_string(),
            message: "Markdown link text looks like code but lacks backticks: ".to_owned()
                + "[SqlType::Custom] should be [`SqlType::Custom`]",
            line: 5,
            column: 20,
            severity: Severity::Warning,
        };

        let formatted = format!("{violation}");
        assert!(formatted.starts_with("5:20 warning [R401]:"));
        assert!(formatted.contains("[SqlType::Custom]"));
        assert!(formatted.contains("[`SqlType::Custom`]"));
    }

    /// Test Display implementation preserves exact message content
    #[test]
    fn test_violation_display_message_preservation() {
        let message = "Use inline code for common Rust type: [Option](...) should be `Option`";
        let violation = Violation {
            rule: "R402".to_string(),
            message: message.to_string(),
            line: 99,
            column: 8,
            severity: Severity::Warning,
        };

        let formatted = format!("{violation}");
        assert_eq!(formatted, format!("99:8 warning [R402]: {message}"));
    }

    /// Test Display implementation with line 1, column 1
    #[test]
    fn test_violation_display_start_position() {
        let violation = Violation {
            rule: "D103".to_string(),
            message: "Missing docstring in public function".to_string(),
            line: 1,
            column: 1,
            severity: Severity::Error,
        };

        let formatted = format!("{violation}");
        assert_eq!(formatted, "1:1 error [D103]: Missing docstring in public function");
    }

    /// Test that to_string() works correctly (uses Display)
    #[test]
    fn test_violation_to_string() {
        let violation = Violation {
            rule: "D402".to_string(),
            message: "First line should not be the function's signature".to_string(),
            line: 7,
            column: 4,
            severity: Severity::Error,
        };

        let as_string = violation.to_string();
        assert_eq!(
            as_string,
            "7:4 error [D402]: First line should not be the function's signature"
        );
    }

    /// Test Display formatting consistency across multiple violations
    #[test]
    fn test_violation_display_consistency() {
        let violations = [
            Violation {
                rule: "D201".to_string(),
                message: "No blank lines allowed before function docstring".to_string(),
                line: 15,
                column: 1,
                severity: Severity::Error,
            },
            Violation {
                rule: "D301".to_string(),
                message: "Consider using raw strings for docstrings with backslashes".to_string(),
                line: 20,
                column: 1,
                severity: Severity::Warning,
            },
            Violation {
                rule: "D403".to_string(),
                message: "First word of the first line should be properly capitalized".to_string(),
                line: 25,
                column: 1,
                severity: Severity::Error,
            },
        ];

        // Verify each violation formats correctly and consistently
        let formatted: Vec<String> = violations.iter().map(|v| format!("{v}")).collect();

        assert_eq!(
            formatted[0],
            "15:1 error [D201]: No blank lines allowed before function docstring"
        );
        assert_eq!(
            formatted[1],
            "20:1 warning [D301]: Consider using raw strings for docstrings with backslashes"
        );
        assert_eq!(
            formatted[2],
            "25:1 error [D403]: First word of the first line should be properly capitalized"
        );

        // Verify the format pattern is consistent
        for display_str in formatted {
            let parts: Vec<&str> = display_str.split(':').collect();
            assert!(parts.len() >= 3, "Should have line:column:rest format");
            assert!(
                display_str.contains("error") || display_str.contains("warning"),
                "Should contain severity"
            );
            assert!(
                display_str.contains('[') && display_str.contains(']'),
                "Should contain rule in brackets"
            );
        }
    }

    /// D201: Test blank line before function docstring
    #[test]
    fn test_d201_function_with_leading_blank() {
        let docstring = Docstring {
            content: "\nCalculate the sum.".to_string(),
            raw_content: "///\n/// Calculate the sum.".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D201"));
        let d201 = violations.iter().find(|v| v.rule == "D201").unwrap();
        assert!(d201.message.contains("function"));
    }

    /// D201: Test blank line before struct docstring
    #[test]
    fn test_d201_struct_with_leading_blank() {
        let docstring = Docstring {
            content: "\nRepresents a point in 2D space.".to_string(),
            raw_content: "///\n/// Represents a point in 2D space.".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Struct,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D201"));
        let d201 = violations.iter().find(|v| v.rule == "D201").unwrap();
        assert!(d201.message.contains("struct"));
    }

    /// D201: Test blank line before enum docstring
    #[test]
    fn test_d201_enum_with_leading_blank() {
        let docstring = Docstring {
            content: "\nRepresents different states.".to_string(),
            raw_content: "///\n/// Represents different states.".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Enum,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D201"));
        let d201 = violations.iter().find(|v| v.rule == "D201").unwrap();
        assert!(d201.message.contains("enum"));
    }

    /// D201: Test blank line before trait docstring
    #[test]
    fn test_d201_trait_with_leading_blank() {
        let docstring = Docstring {
            content: "\nDefines behavior for serialization.".to_string(),
            raw_content: "///\n/// Defines behavior for serialization.".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Trait,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D201"));
        let d201 = violations.iter().find(|v| v.rule == "D201").unwrap();
        assert!(d201.message.contains("trait"));
    }

    /// D201: Test no false positive when docstring starts properly
    #[test]
    fn test_d201_no_false_positive() {
        let docstring = Docstring {
            content: "Calculate the sum.".to_string(),
            raw_content: "/// Calculate the sum.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D201"));
    }

    /// D202: Test blank line after function docstring
    #[test]
    fn test_d202_function_with_trailing_blank() {
        let docstring = Docstring {
            content: "Calculate the sum.\n".to_string(),
            raw_content: "/// Calculate the sum.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D202"));
        let d202 = violations.iter().find(|v| v.rule == "D202").unwrap();
        assert!(d202.message.contains("function"));
    }

    /// D202: Test blank line after struct docstring
    #[test]
    fn test_d202_struct_with_trailing_blank() {
        let docstring = Docstring {
            content: "Represents a point in 2D space.\n".to_string(),
            raw_content: "/// Represents a point in 2D space.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Struct,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D202"));
        let d202 = violations.iter().find(|v| v.rule == "D202").unwrap();
        assert!(d202.message.contains("struct"));
    }

    /// D202: Test blank line after enum docstring
    #[test]
    fn test_d202_enum_with_trailing_blank() {
        let docstring = Docstring {
            content: "Represents different states.\n".to_string(),
            raw_content: "/// Represents different states.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Enum,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D202"));
        let d202 = violations.iter().find(|v| v.rule == "D202").unwrap();
        assert!(d202.message.contains("enum"));
    }

    /// D202: Test blank line after trait docstring
    #[test]
    fn test_d202_trait_with_trailing_blank() {
        let docstring = Docstring {
            content: "Defines behavior for serialization.\n".to_string(),
            raw_content: "/// Defines behavior for serialization.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Trait,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D202"));
        let d202 = violations.iter().find(|v| v.rule == "D202").unwrap();
        assert!(d202.message.contains("trait"));
    }

    /// D202: Test blank line after const docstring
    #[test]
    fn test_d202_const_with_trailing_blank() {
        let docstring = Docstring {
            content: "Maximum buffer size.\n".to_string(),
            raw_content: "/// Maximum buffer size.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Const,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D202"));
        let d202 = violations.iter().find(|v| v.rule == "D202").unwrap();
        assert!(d202.message.contains("const"));
    }

    /// D202: Test no false positive when docstring ends properly
    #[test]
    fn test_d202_no_false_positive() {
        let docstring = Docstring {
            content: "Calculate the sum.".to_string(),
            raw_content: "/// Calculate the sum.".to_string(),
            line: 1,
            column: 1,
            is_multiline: false,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(!violations.iter().any(|v| v.rule == "D202"));
    }

    /// D201 and D202: Test both blank lines before and after
    #[test]
    fn test_d201_and_d202_both_violations() {
        let docstring = Docstring {
            content: "\nCalculate the sum.\n".to_string(),
            raw_content: "///\n/// Calculate the sum.\n///".to_string(),
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(violations.iter().any(|v| v.rule == "D201"));
        assert!(violations.iter().any(|v| v.rule == "D202"));
    }

    /// Summary paragraph wraps across lines — should trigger D400 but not D205
    #[test]
    fn test_wrapped_summary_no_false_positives() {
        let docstring = Docstring {
            content:
                "Summary line that continues on to the next line incorrectly\ndue to wrapping."
                    .to_string(),
            raw_content: "/// Summary line that continues on to the next line ".to_owned()
                + "incorrectly\n/// due to wrapping.",
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        // Summary must be single-line, so wrapped summaries should trigger D400
        // But it should NOT trigger D205 since there's no description following
        assert!(violations.iter().any(|v| v.rule == "D400"));
        assert!(!violations.iter().any(|v| v.rule == "D205"));
    }

    /// Missing blank line between summary paragraph and description should trigger D205
    #[test]
    fn test_missing_blank_line_triggers_d205() {
        let docstring = Docstring {
            content: "Summary line that ends properly.\nThis is a description ".to_owned()
                + "line immediately following the summary without a blank line.",
            raw_content: "/// Summary line that ends properly.\n/// This is a ".to_owned()
                + "description line immediately following the summary without a "
                + "blank line.",
            line: 1,
            column: 1,
            is_multiline: true,
            is_public: true,
            target_type: DocstringTarget::Function,
        };

        let violations = Pep257Checker::check_docstring(&docstring);
        assert!(
            violations.iter().any(|v| v.rule == "D205"),
            "Expected D205 when description immediately follows summary"
        );
    }
}
