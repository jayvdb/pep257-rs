use crate::pep257::{Docstring, DocstringTarget};
use std::fs;
use std::path::Path;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

/// Errors that can occur during parsing
#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse file: tree-sitter error")]
    TreeSitter,
    #[error("Query error: {0}")]
    Query(String),
}

/// Rust parser using tree-sitter
pub struct RustParser {
    parser: Parser,
    language: Language,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new() -> Result<Self, ParseError> {
        let language = tree_sitter_rust::language();
        let mut parser = Parser::new();

        parser
            .set_language(language)
            .map_err(|_| ParseError::TreeSitter)?;

        Ok(Self { parser, language })
    }

    /// Parse a Rust file and extract all docstrings
    pub fn parse_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Vec<Docstring>, ParseError> {
        let source_code = fs::read_to_string(path)?;
        self.parse_source(&source_code)
    }

    /// Parse Rust source code and extract all docstrings
    pub fn parse_source(&mut self, source_code: &str) -> Result<Vec<Docstring>, ParseError> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or(ParseError::TreeSitter)?;

        let mut docstrings = Vec::new();

        // Extract docstrings from various Rust constructs
        docstrings.extend(self.extract_function_docs(&tree, source_code)?);
        docstrings.extend(self.extract_struct_docs(&tree, source_code)?);
        docstrings.extend(self.extract_enum_docs(&tree, source_code)?);
        docstrings.extend(self.extract_trait_docs(&tree, source_code)?);
        docstrings.extend(self.extract_impl_docs(&tree, source_code)?);
        docstrings.extend(self.extract_mod_docs(&tree, source_code)?);
        docstrings.extend(self.extract_const_docs(&tree, source_code)?);

        Ok(docstrings)
    }

    /// Extract documentation from function declarations
    fn extract_function_docs(
        &self,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (function_item
              name: (identifier) @name
            ) @function
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the function node (not the name node)
            let function_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @function capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(function_node, source, &DocstringTarget::Function)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from struct declarations
    fn extract_struct_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (struct_item
              name: (type_identifier) @name
            ) @struct
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the struct node (not the name node)
            let struct_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @struct capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(struct_node, source, &DocstringTarget::Struct)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from enum declarations
    fn extract_enum_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (enum_item
              name: (type_identifier) @name
            ) @enum
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the enum node (not the name node)
            let enum_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @enum capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(enum_node, source, &DocstringTarget::Enum)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from trait declarations
    fn extract_trait_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (trait_item
              name: (type_identifier) @name
            ) @trait
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the trait node (not the name node)
            let trait_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @trait capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(trait_node, source, &DocstringTarget::Trait)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from impl blocks
    fn extract_impl_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (impl_item) @impl
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            let impl_node = query_match.captures[0].node;

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(impl_node, source, &DocstringTarget::Impl)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from module declarations
    fn extract_mod_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (mod_item
              name: (identifier) @name
            ) @module
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the module node (not the name node)
            let mod_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @module capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(mod_node, source, &DocstringTarget::Module)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from const declarations
    fn extract_const_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            self.language,
            r#"
            (const_item
              name: (identifier) @name
            ) @const
            "#,
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Find the const node (not the name node)
            let const_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1) // The @const capture
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                self.extract_preceding_docs(const_node, source, &DocstringTarget::Const)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Generic function to extract documentation using a tree-sitter query
    #[allow(dead_code)]
    fn extract_docs_with_query(
        &self,
        tree: &Tree,
        source: &str,
        query: &Query,
        target_type: DocstringTarget,
    ) -> Result<Vec<Docstring>, ParseError> {
        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        for query_match in matches {
            // Get the main node (the function/struct/etc itself, not just the name)
            let main_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 0) // The first capture is typically the main node
                .map(|capture| capture.node)
                .unwrap_or_else(|| query_match.captures[0].node);

            // Look for documentation comments before this node
            if let Some(docstring) = self.extract_preceding_docs(main_node, source, &target_type)? {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation comments preceding a given node
    fn extract_preceding_docs(
        &self,
        node: tree_sitter::Node,
        source: &str,
        target_type: &DocstringTarget,
    ) -> Result<Option<Docstring>, ParseError> {
        let mut doc_comments = Vec::new();
        let mut doc_attributes = Vec::new();
        let mut current_node = node;
        let mut first_doc_node = None;

        // Walk backwards to find preceding comments and attributes
        while let Some(prev_sibling) = current_node.prev_sibling() {
            if prev_sibling.kind() == "line_comment" {
                let comment_text = prev_sibling
                    .utf8_text(source.as_bytes())
                    .map_err(|_| ParseError::TreeSitter)?;

                // Check if it's a doc comment (starts with ///)
                if comment_text.trim_start().starts_with("///") {
                    doc_comments.insert(0, comment_text);
                    if first_doc_node.is_none() {
                        first_doc_node = Some(prev_sibling);
                    }
                } else {
                    break; // Stop if we hit a non-doc comment
                }
            } else if prev_sibling.kind() == "block_comment" {
                let comment_text = prev_sibling
                    .utf8_text(source.as_bytes())
                    .map_err(|_| ParseError::TreeSitter)?;

                // Check if it's a doc comment (starts with /**)
                if comment_text.trim_start().starts_with("/**") {
                    doc_comments.insert(0, comment_text);
                    if first_doc_node.is_none() {
                        first_doc_node = Some(prev_sibling);
                    }
                    break; // Block comments usually stand alone
                } else {
                    break;
                }
            } else if prev_sibling.kind() == "attribute_item"
                || prev_sibling.kind() == "outer_attribute_item"
            {
                // Check for #[doc = "..."] attributes
                if let Some(doc_content) = self.extract_doc_attribute(&prev_sibling, source)? {
                    doc_attributes.insert(0, doc_content);
                    if first_doc_node.is_none() {
                        first_doc_node = Some(prev_sibling);
                    }
                }
            } else if prev_sibling.kind() == "whitespace"
                || prev_sibling
                    .utf8_text(source.as_bytes())
                    .unwrap_or("")
                    .trim()
                    .is_empty()
            {
                // Skip whitespace and continue looking
                current_node = prev_sibling;
                continue;
            } else {
                break; // Stop if we hit something else
            }
            current_node = prev_sibling;
        }

        // Combine doc attributes and comments
        let has_documentation = !doc_comments.is_empty() || !doc_attributes.is_empty();

        // If no documentation was found, create an empty docstring to report missing docs
        if !has_documentation {
            let start_point = node.start_position();
            return Ok(Some(Docstring {
                content: String::new(), // Empty content indicates missing docstring
                raw_content: String::new(),
                line: start_point.row + 1,
                column: start_point.column + 1,
                is_multiline: false,
                target_type: target_type.clone(),
            }));
        }

        // Process the documentation (attributes take precedence, then comments)
        let raw_content = if !doc_attributes.is_empty() {
            doc_attributes.join("\n")
        } else {
            doc_comments.join("\n")
        };

        let processed_content = if !doc_attributes.is_empty() {
            doc_attributes.join("\n")
        } else {
            self.process_doc_comments(&doc_comments)
        };

        let is_multiline = processed_content.lines().count() > 1;

        // Get position of the first documentation element
        let start_point = first_doc_node.unwrap_or(node).start_position();

        Ok(Some(Docstring {
            content: processed_content,
            raw_content,
            line: start_point.row + 1, // Convert to 1-based indexing
            column: start_point.column + 1,
            is_multiline,
            target_type: target_type.clone(),
        }))
    }

    /// Extract documentation from a #[doc = "..."] attribute
    fn extract_doc_attribute(
        &self,
        attr_node: &tree_sitter::Node,
        source: &str,
    ) -> Result<Option<String>, ParseError> {
        let attr_text = attr_node
            .utf8_text(source.as_bytes())
            .map_err(|_| ParseError::TreeSitter)?;

        // Check if it's a doc attribute
        if attr_text.contains("doc") {
            // Parse #[doc = "content"] or #[doc(hidden)] etc.
            if let Some(start) = attr_text.find("doc") {
                let remaining = &attr_text[start..];

                // Look for doc = "..." pattern
                if let Some(eq_pos) = remaining.find('=') {
                    let after_eq = &remaining[eq_pos + 1..].trim_start();

                    // Extract string content between quotes
                    if let Some(stripped) = after_eq.strip_prefix('"') {
                        if let Some(end_quote) = stripped.find('"') {
                            let content = &stripped[..end_quote];
                            return Ok(Some(content.to_string()));
                        }
                    } else if let Some(stripped) = after_eq.strip_prefix("r#\"") {
                        // Handle raw strings r#"..."#
                        if let Some(end_pos) = stripped.find("\"#") {
                            let content = &stripped[..end_pos];
                            return Ok(Some(content.to_string()));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Process documentation comments to extract clean content
    fn process_doc_comments(&self, comments: &[&str]) -> String {
        let mut processed_lines = Vec::new();

        for comment in comments {
            let trimmed = comment.trim();

            if let Some(content) = trimmed.strip_prefix("///") {
                // Handle /// style comments
                let clean_content = content.trim_start();
                processed_lines.push(clean_content);
            } else if let Some(content) = trimmed.strip_prefix("/**") {
                // Handle /** */ style comments
                let content = content.strip_suffix("*/").unwrap_or(content);
                let lines: Vec<&str> = content.lines().collect();

                for line in lines {
                    let clean_line = line.trim_start_matches('*').trim_start();
                    processed_lines.push(clean_line);
                }
            }
        }

        // Remove empty lines at the beginning and end
        while processed_lines
            .first()
            .is_some_and(|line| line.trim().is_empty())
        {
            processed_lines.remove(0);
        }
        while processed_lines
            .last()
            .is_some_and(|line| line.trim().is_empty())
        {
            processed_lines.pop();
        }

        processed_lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, "Calculate the sum of two numbers.");
        assert!(!docstrings[0].is_multiline);
    }

    #[test]
    fn test_parse_multiline_function() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"
/// Calculate the sum of two numbers.
/// 
/// This function takes two integers and returns their sum.
/// It's a simple arithmetic operation.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert!(docstrings[0].is_multiline);
        assert!(docstrings[0].content.contains("Calculate the sum"));
        assert!(docstrings[0].content.contains("arithmetic operation"));
    }

    #[test]
    fn test_parse_struct() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"
/// Represents a point in 2D space.
struct Point {
    x: f64,
    y: f64,
}
"#;

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, "Represents a point in 2D space.");
        assert!(matches!(docstrings[0].target_type, DocstringTarget::Struct));
    }
}
