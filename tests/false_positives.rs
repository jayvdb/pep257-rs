//! Integration tests guarding against known docstring false positives.
//!
//! These exercise the public [`RustDocAnalyzer`] API end-to-end against real
//! files on disk, which is necessary for the `mod foo;` resolution that follows
//! a declaration to its backing file.
#![cfg(test)]

use std::{fs, path::Path};

use pep257::analyzer::RustDocAnalyzer;
use tempfile::TempDir;

/// Analyze a single source file and return the rule codes that fired.
fn rules_for(path: &Path) -> Vec<String> {
    let mut analyzer = RustDocAnalyzer::new().unwrap();
    analyzer.analyze_file(path).unwrap().into_iter().map(|v| v.rule).collect()
}

/// `mod foo;` is documented by its backing `foo.rs` `//!`, so no D100.
#[test]
fn mod_declaration_credited_to_backing_file_docs() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("lib.rs"), "//! Crate.\n\npub mod agent;\n").unwrap();
    fs::write(src.join("agent.rs"), "//! An agent.\n\n/// A thing.\npub struct A;\n").unwrap();

    assert!(
        !rules_for(&src.join("lib.rs")).contains(&"D100".to_string()),
        "documented module declaration must not raise D100"
    );
}

/// A `foo/mod.rs`-style backing file is also resolved and credited.
#[test]
fn mod_declaration_credited_to_mod_rs_docs() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("lib.rs"), "//! Crate.\n\npub mod agent;\n").unwrap();
    fs::create_dir(src.join("agent")).unwrap();
    fs::write(src.join("agent").join("mod.rs"), "//! An agent.\n\npub struct A;\n").unwrap();

    assert!(
        !rules_for(&src.join("lib.rs")).contains(&"D100".to_string()),
        "documented agent/mod.rs must credit the declaration"
    );
}

/// `mod foo;` whose backing file lacks `//!` docs still reports missing (D100).
#[test]
fn mod_declaration_reports_when_backing_file_undocumented() {
    let dir = TempDir::new().unwrap();
    let src = dir.path().join("src");
    fs::create_dir(&src).unwrap();
    fs::write(src.join("lib.rs"), "//! Crate.\n\npub mod plain;\n").unwrap();
    fs::write(src.join("plain.rs"), "pub struct A;\n").unwrap();

    assert!(
        rules_for(&src.join("lib.rs")).contains(&"D100".to_string()),
        "undocumented backing module must still raise D100"
    );
}

/// An inline `mod foo { ... }` without docs still reports missing (D100).
#[test]
fn inline_module_still_requires_docs() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("inline.rs");
    fs::write(&file, "//! Crate.\n\npub mod inner {\n    pub struct A;\n}\n").unwrap();

    assert!(
        rules_for(&file).contains(&"D100".to_string()),
        "undocumented inline module must still raise D100"
    );
}

/// D401 (imperative mood) applies to functions, not type definitions.
///
/// Type docs are noun phrases ("An error from…"), which is PEP-257-correct and
/// idiomatic Rust; only the function summary should be flagged.
#[test]
fn d401_scoped_to_functions_not_types() {
    let dir = TempDir::new().unwrap();
    let file = dir.path().join("d401.rs");
    fs::write(
        &file,
        "//! Demo.\n\n\
         /// An error from the storage layer.\n\
         pub struct StorageError;\n\n\
         /// Returns the value.\n\
         pub fn get_value() -> i32 {\n    0\n}\n",
    )
    .unwrap();

    let rules = rules_for(&file);
    // The function's non-imperative summary fires D401...
    assert!(rules.contains(&"D401".to_string()), "function summary should fire D401");
    // ...but only once: the struct's noun-phrase summary must not.
    assert_eq!(
        rules.iter().filter(|r| *r == "D401").count(),
        1,
        "only the function, not the struct, should raise D401"
    );
}
