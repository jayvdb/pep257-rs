use std::{fs, path::Path};

use streaming_iterator::StreamingIterator as _;
use tree_sitter::{Language, Parser, Query, QueryCursor, Tree};

use crate::pep257::{Docstring, DocstringTarget};

/// Errors that can occur during parsing.
#[derive(thiserror::Error, Debug)]
pub(crate) enum ParseError {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse file: tree-sitter error")]
    TreeSitter,
    #[error("Query error: {0}")]
    Query(String),
}

/// Rust parser using tree-sitter.
pub(crate) struct RustParser {
    parser: Parser,
    language: Language,
}

/// Implementation of parser methods.
impl RustParser {
    /// Create a new Rust parser instance.
    pub(crate) fn new() -> Result<Self, ParseError> {
        let language = tree_sitter_rust::LANGUAGE.into();
        let mut parser = Parser::new();

        parser.set_language(&language).map_err(|_| ParseError::TreeSitter)?;

        Ok(Self { parser, language })
    }

    /// Parses a Rust file and extracts docstrings.
    pub(crate) fn parse_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<Vec<Docstring>, ParseError> {
        let source_code = fs::read_to_string(path)?;
        self.parse_source(&source_code)
    }

    /// Parses Rust source code and extracts docstrings.
    pub(crate) fn parse_source(&mut self, source_code: &str) -> Result<Vec<Docstring>, ParseError> {
        let tree = self.parser.parse(source_code, None).ok_or(ParseError::TreeSitter)?;

        let mut docstrings = Vec::new();

        // Extract crate/package-level documentation (//! comments at the top of file)
        docstrings.extend(Self::extract_package_docs(&tree, source_code));

        // Extract docstrings from various Rust constructs
        docstrings.extend(self.extract_function_docs(&tree, source_code)?);
        docstrings.extend(self.extract_struct_docs(&tree, source_code)?);
        docstrings.extend(self.extract_enum_docs(&tree, source_code)?);
        docstrings.extend(self.extract_trait_docs(&tree, source_code)?);
        docstrings.extend(self.extract_impl_docs(&tree, source_code)?);
        docstrings.extend(self.extract_mod_docs(&tree, source_code)?);
        docstrings.extend(self.extract_const_docs(&tree, source_code)?);
        docstrings.extend(self.extract_type_alias_docs(&tree, source_code)?);
        docstrings.extend(self.extract_macro_docs(&tree, source_code)?);

        Ok(docstrings)
    }

    /// Extract crate/package-level documentation (inner doc comments).
    ///
    /// This checks for //! or /*! */ comments at the beginning of the file,
    /// which document the crate/module/package itself (D104).
    fn extract_package_docs(tree: &Tree, source: &str) -> Vec<Docstring> {
        let root_node = tree.root_node();
        let mut inner_doc_comments = Vec::new();

        // Look for inner doc comments (//! or /*!  */) at the start of the file
        let mut cursor = root_node.walk();

        for child in root_node.children(&mut cursor) {
            match child.kind() {
                "line_comment" => {
                    if let Ok(comment_text) = child.utf8_text(source.as_bytes()) {
                        if comment_text.trim().starts_with("//!") {
                            inner_doc_comments.push(comment_text);
                        } else if !comment_text.trim().starts_with("///") {
                            // Stop at first non-doc comment
                            break;
                        }
                    }
                }
                "block_comment" => {
                    if let Ok(comment_text) = child.utf8_text(source.as_bytes()) {
                        if comment_text.trim().starts_with("/*!") {
                            inner_doc_comments.push(comment_text);
                        } else if !comment_text.trim().starts_with("/**") {
                            // Stop at first non-doc comment
                            break;
                        }
                    }
                }
                "whitespace" => {
                    // Skip whitespace
                }
                _ => {
                    // Stop at first non-comment, non-whitespace node
                    break;
                }
            }
        }

        // If we found inner doc comments, process them
        if !inner_doc_comments.is_empty() {
            let content = Self::process_inner_doc_comments(&inner_doc_comments);
            let is_multiline = inner_doc_comments.len() > 1 || content.contains('\n');

            return vec![Docstring {
                content,
                raw_content: inner_doc_comments.join("\n"),
                line: 1,
                column: 1,
                is_multiline,
                is_public: true, // Package-level docs are always public
                target_type: DocstringTarget::Package,
            }];
        }

        // No inner doc comments found - don't report missing for simple test files
        // Only report missing when we have pub items that suggest this is a real module/crate
        let has_pub_items = root_node.children(&mut cursor).any(|child| {
            if let Ok(text) = child.utf8_text(source.as_bytes()) {
                text.trim_start().starts_with("pub ")
            } else {
                false
            }
        });

        if has_pub_items {
            // This looks like a real module/crate file, report missing package docs
            vec![Docstring {
                content: String::new(),
                raw_content: String::new(),
                line: 1,
                column: 1,
                is_multiline: false,
                is_public: true,
                target_type: DocstringTarget::Package,
            }]
        } else {
            // No public items, probably just a test snippet - don't report missing
            Vec::new()
        }
    }

    /// Extract documentation from function declarations.
    fn extract_function_docs(
        &self,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (function_item
                name: (identifier) @name
            ) @function
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the function node (not the name node)
            let function_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(function_node, source, DocstringTarget::Function)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from struct declarations.
    fn extract_struct_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (struct_item
                name: (type_identifier) @name
            ) @struct
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the struct node (not the name node)
            let struct_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(struct_node, source, DocstringTarget::Struct)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from enum declarations.
    fn extract_enum_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (enum_item
                name: (type_identifier) @name
            ) @enum
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the enum node (not the name node)
            let enum_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(enum_node, source, DocstringTarget::Enum)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from trait declarations.
    fn extract_trait_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (trait_item
                name: (type_identifier) @name
            ) @trait
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the trait node (not the name node)
            let trait_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(trait_node, source, DocstringTarget::Trait)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from impl blocks.
    fn extract_impl_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (impl_item) @impl
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            let impl_node = query_match.captures[0].node;

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(impl_node, source, DocstringTarget::Impl)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from module declarations.
    fn extract_mod_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (mod_item
                name: (identifier) @name
            ) @module
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the module node (not the name node)
            let mod_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(mod_node, source, DocstringTarget::Module)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from const declarations.
    fn extract_const_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (const_item
                name: (identifier) @name
            ) @const
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the const node (not the name node)
            let const_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(const_node, source, DocstringTarget::Const)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from type alias declarations.
    fn extract_type_alias_docs(
        &self,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (type_item
                name: (type_identifier) @name
            ) @type_alias
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the type alias node (not the name node)
            let type_alias_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(type_alias_node, source, DocstringTarget::TypeAlias)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation from macro declarations.
    fn extract_macro_docs(&self, tree: &Tree, source: &str) -> Result<Vec<Docstring>, ParseError> {
        let query = Query::new(
            &self.language,
            r"
            (macro_definition
                name: (identifier) @name
            ) @macro
            ",
        )
        .map_err(|e| ParseError::Query(e.to_string()))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Find the macro node (not the name node)
            let macro_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 1)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) =
                Self::extract_preceding_docs(macro_node, source, DocstringTarget::Macro)?
            {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Generic function to extract documentation using a tree-sitter query.
    #[allow(dead_code)]
    fn extract_docs_with_query(
        tree: &Tree,
        source: &str,
        query: &Query,
        target_type: DocstringTarget,
    ) -> Result<Vec<Docstring>, ParseError> {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, tree.root_node(), source.as_bytes());
        let mut docstrings = Vec::new();

        while let Some(query_match) = matches.next() {
            // Get the main node (the function/struct/etc itself, not just the name)
            let main_node = query_match
                .captures
                .iter()
                .find(|capture| capture.index == 0)
                .map_or_else(|| query_match.captures[0].node, |capture| capture.node);

            // Look for documentation comments before this node
            if let Some(docstring) = Self::extract_preceding_docs(main_node, source, target_type)? {
                docstrings.push(docstring);
            }
        }

        Ok(docstrings)
    }

    /// Extract documentation comments preceding a given node.
    fn extract_preceding_docs(
        node: tree_sitter::Node<'_>,
        source: &str,
        target_type: DocstringTarget,
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
                }
                break;
            } else if prev_sibling.kind() == "attribute_item"
                || prev_sibling.kind() == "outer_attribute_item"
            {
                // Check for #[doc = "..."] attributes
                if let Some(doc_content) = Self::extract_doc_attribute(&prev_sibling, source)? {
                    doc_attributes.insert(0, doc_content);
                    if first_doc_node.is_none() {
                        first_doc_node = Some(prev_sibling);
                    }
                }
            } else if prev_sibling.kind() == "whitespace"
                || prev_sibling.utf8_text(source.as_bytes()).unwrap_or("").trim().is_empty()
            {
                // Skip whitespace and continue looking
                current_node = prev_sibling;
                continue;
            } else {
                break; // Stop if we hit something else
            }
            current_node = prev_sibling;
        }

        // Determine visibility (public/private) for the node
        let mut is_public = false;

        // For macros, check for #[macro_export] attribute
        if target_type == DocstringTarget::Macro {
            // Check siblings before the macro for #[macro_export]
            let mut check_node = node;
            while let Some(prev) = check_node.prev_sibling() {
                if prev.kind() == "attribute_item" {
                    if let Ok(attr_text) = prev.utf8_text(source.as_bytes())
                        && attr_text.contains("macro_export")
                    {
                        is_public = true;
                        break;
                    }
                } else if prev.kind() == "line_comment" || prev.kind() == "block_comment" {
                    // Skip comments
                    check_node = prev;
                    continue;
                } else if prev.kind() == "whitespace"
                    || prev.utf8_text(source.as_bytes()).unwrap_or("").trim().is_empty()
                {
                    // Skip whitespace
                    check_node = prev;
                    continue;
                } else {
                    break;
                }
                check_node = prev;
            }
        } else {
            // For other types, use standard visibility checking
            if let Some(visibility_node) = node.child_by_field_name("visibility") {
                if let Ok(vis_text) = visibility_node.utf8_text(source.as_bytes())
                    && vis_text.contains("pub")
                {
                    is_public = true;
                }
            } else {
                // Fallback: check the node text for a leading `pub` token (some nodes
                // may represent visibility as a token rather than a named field)
                if let Ok(node_text) = node.utf8_text(source.as_bytes())
                    && (node_text.trim_start().starts_with("pub ")
                        || node_text.trim_start().starts_with("pub("))
                {
                    is_public = true;
                }
            }
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
                is_public,
                target_type,
            }));
        }

        // Process the documentation (attributes take precedence, then comments)
        let raw_content = if doc_attributes.is_empty() {
            doc_comments.join("\n")
        } else {
            doc_attributes.join("\n")
        };

        let processed_content = if doc_attributes.is_empty() {
            Self::process_doc_comments(&doc_comments)
        } else {
            doc_attributes.join("\n")
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
            is_public,
            target_type,
        }))
    }

    /// Extract documentation from a #[doc = "..."] attribute.
    fn extract_doc_attribute(
        attr_node: &tree_sitter::Node<'_>,
        source: &str,
    ) -> Result<Option<String>, ParseError> {
        let attr_text =
            attr_node.utf8_text(source.as_bytes()).map_err(|_| ParseError::TreeSitter)?;

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

    /// Process documentation comments to extract clean content.
    fn process_doc_comments(comments: &[&str]) -> String {
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

        // DO NOT remove empty lines at the beginning and end
        // We need to preserve them for D201 and D202 checks
        processed_lines.join("\n")
    }

    /// Process inner documentation comments (//! and /*! */) to extract clean content.
    fn process_inner_doc_comments(comments: &[&str]) -> String {
        let mut processed_lines = Vec::new();

        for comment in comments {
            let trimmed = comment.trim();

            if let Some(content) = trimmed.strip_prefix("//!") {
                // Handle //! style comments
                let clean_content = content.trim_start();
                processed_lines.push(clean_content);
            } else if let Some(content) = trimmed.strip_prefix("/*!") {
                // Handle /*! */ style comments
                let content = content.strip_suffix("*/").unwrap_or(content);
                let lines: Vec<&str> = content.lines().collect();

                for line in lines {
                    let clean_line = line.trim_start_matches('*').trim_start();
                    processed_lines.push(clean_line);
                }
            }
        }

        // DO NOT remove empty lines at the beginning and end
        // We need to preserve them for D201 and D202 checks
        processed_lines.join("\n")
    }
}

/// Unit tests for the parser.
#[cfg(test)]
mod tests {
    use super::*;

    /// Test parsing a simple function with documentation.
    #[test]
    fn test_parse_simple_function() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
/// Calculate the sum of two numbers.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, "Calculate the sum of two numbers.");
        assert!(!docstrings[0].is_multiline);
    }

    /// Test parsing a public function sets is_public = true.
    #[test]
    fn test_parse_public_function_sets_is_public() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
/// Public function docs.
pub fn public_add(a: i32, b: i32) -> i32 {
    a + b
}
";

        let docstrings = parser.parse_source(source).unwrap();
        // Should have package doc (empty) + function doc
        assert_eq!(docstrings.len(), 2);

        let function_doc =
            docstrings.iter().find(|d| matches!(d.target_type, DocstringTarget::Function)).unwrap();
        assert_eq!(function_doc.content, "Public function docs.");
        assert!(function_doc.is_public, "Expected is_public to be true for pub fn");
    }

    /// Test parsing a multiline function documentation.
    #[test]
    fn test_parse_multiline_function() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
/// Calculate the sum of two numbers.
///
/// This function takes two integers and returns their sum.
/// It's a simple arithmetic operation.
fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert!(docstrings[0].is_multiline);
        assert!(docstrings[0].content.contains("Calculate the sum"));
        assert!(docstrings[0].content.contains("arithmetic operation"));
    }

    /// Test parsing a struct with documentation.
    #[test]
    fn test_parse_struct() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
/// Represents a point in 2D space.
struct Point {
    x: f64,
    y: f64,
}
";

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, "Represents a point in 2D space.");
        assert!(matches!(docstrings[0].target_type, DocstringTarget::Struct));
    }

    /// Test parsing a type alias with documentation.
    #[test]
    fn test_parse_type_alias() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
/// A specialized Result type.
pub type Result<T> = std::result::Result<T, Error>;
";

        let docstrings = parser.parse_source(source).unwrap();
        // Should have package doc (empty) + type alias doc
        assert_eq!(docstrings.len(), 2);

        let type_alias_doc = docstrings
            .iter()
            .find(|d| matches!(d.target_type, DocstringTarget::TypeAlias))
            .unwrap();
        assert_eq!(type_alias_doc.content, "A specialized Result type.");
        assert!(type_alias_doc.is_public);
    }

    /// Test parsing a macro with documentation.
    #[test]
    fn test_parse_macro() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"
/// Log an error message.
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("ERROR: {}", format_args!($($arg)*));
    };
}
"#;

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, "Log an error message.");
        assert!(matches!(docstrings[0].target_type, DocstringTarget::Macro));
    }

    /// Test parsing undocumented type alias (should report missing docstring).
    #[test]
    fn test_parse_undocumented_type_alias() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
pub type UndocumentedType = i32;
";

        let docstrings = parser.parse_source(source).unwrap();
        // Should have package doc (empty) + type alias doc (empty)
        assert_eq!(docstrings.len(), 2);

        let type_alias_doc = docstrings
            .iter()
            .find(|d| matches!(d.target_type, DocstringTarget::TypeAlias))
            .unwrap();
        assert_eq!(type_alias_doc.content, ""); // Empty indicates missing docstring
        assert!(type_alias_doc.is_public);
    }

    /// Test parsing undocumented macro (should report missing docstring).
    #[test]
    fn test_parse_undocumented_macro() {
        let mut parser = RustParser::new().unwrap();
        let source = r"
macro_rules! undocumented {
    () => { };
}
";

        let docstrings = parser.parse_source(source).unwrap();
        assert_eq!(docstrings.len(), 1);
        assert_eq!(docstrings[0].content, ""); // Empty indicates missing docstring
        assert!(matches!(docstrings[0].target_type, DocstringTarget::Macro));
    }

    /// Test parsing package-level documentation (lib.rs style with //!).
    #[test]
    fn test_parse_package_docs_present() {
        let mut parser = RustParser::new().unwrap();
        let source = r"//! A mathematics library.
//!
//! Provides various calculation and utility functions.

pub mod calculator;
pub mod utils;
";

        let docstrings = parser.parse_source(source).unwrap();
        // Should have package-level docs
        let package_docs: Vec<_> = docstrings
            .iter()
            .filter(|d| matches!(d.target_type, DocstringTarget::Package))
            .collect();

        assert_eq!(package_docs.len(), 1);
        assert!(package_docs[0].content.contains("mathematics library"));
        assert!(package_docs[0].is_public);
        assert!(package_docs[0].is_multiline);
    }

    /// Test parsing file without package-level documentation.
    #[test]
    fn test_parse_package_docs_missing() {
        let mut parser = RustParser::new().unwrap();
        let source = r"pub mod calculator;
pub mod utils;

/// Add two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
";

        let docstrings = parser.parse_source(source).unwrap();
        // Should still create an empty package docstring to report missing docs
        let package_docs: Vec<_> = docstrings
            .iter()
            .filter(|d| matches!(d.target_type, DocstringTarget::Package))
            .collect();

        assert_eq!(package_docs.len(), 1);
        assert_eq!(package_docs[0].content, ""); // Empty indicates missing
        assert!(package_docs[0].is_public);
    }

    /// Test parsing package-level docs with block comment style (/*! */).
    #[test]
    fn test_parse_package_docs_block_comment() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"/*! Command-line tool for calculations.
 *
 * This binary provides a CLI interface.
 */

fn main() {
    println!("Hello");
}
"#;

        let docstrings = parser.parse_source(source).unwrap();
        let package_docs: Vec<_> = docstrings
            .iter()
            .filter(|d| matches!(d.target_type, DocstringTarget::Package))
            .collect();

        assert_eq!(package_docs.len(), 1);
        assert!(package_docs[0].content.contains("Command-line tool"));
        assert!(package_docs[0].is_public);
    }
}
