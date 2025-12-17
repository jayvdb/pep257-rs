//! Integration tests for file collection functionality.

use std::{fs, path::PathBuf};

use pep257::file_collector::collect_rust_files_recursive;

/// Create a test directory structure and return the path to the temp directory.
fn setup_test_dir(test_name: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir().join(format!("pep257_test_{}", test_name));

    // Clean up if exists
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).ok();
    }

    fs::create_dir_all(&temp_dir).unwrap();
    temp_dir
}

/// Clean up test directory.
fn cleanup_test_dir(dir: &PathBuf) {
    if dir.exists() {
        fs::remove_dir_all(dir).ok();
    }
}

#[test]
fn test_collect_basic_rust_files() {
    let test_dir = setup_test_dir("basic");

    // Create some .rs files
    fs::write(test_dir.join("file1.rs"), "// test file 1").unwrap();
    fs::write(test_dir.join("file2.rs"), "// test file 2").unwrap();
    fs::write(test_dir.join("other.txt"), "// not rust").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.rs"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.rs"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_recursive_collection() {
    let test_dir = setup_test_dir("recursive");

    // Create nested structure
    fs::create_dir_all(test_dir.join("src")).unwrap();
    fs::create_dir_all(test_dir.join("src/module")).unwrap();

    fs::write(test_dir.join("main.rs"), "// main").unwrap();
    fs::write(test_dir.join("src/lib.rs"), "// lib").unwrap();
    fs::write(test_dir.join("src/module/mod.rs"), "// mod").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    assert_eq!(files.len(), 3);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_skip_target_without_rust_files() {
    let test_dir = setup_test_dir("target_skip");

    // Create target directory without .rs files
    fs::create_dir_all(test_dir.join("target/debug")).unwrap();
    fs::write(test_dir.join("target/debug/binary"), "binary data").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.join("src")).unwrap();
    fs::write(test_dir.join("src/lib.rs"), "// lib").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    // Should only find the src file, not traverse into target
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/lib.rs"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_target_with_cargo_lock() {
    let test_dir = setup_test_dir("cargo_lock");

    // Create Cargo.lock at project root
    fs::write(test_dir.join("Cargo.lock"), "# Cargo.lock").unwrap();

    // Create target directory (should be skipped)
    fs::create_dir_all(test_dir.join("target/debug")).unwrap();
    fs::write(test_dir.join("target/debug/build.rs"), "// build script").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.join("src")).unwrap();
    fs::write(test_dir.join("src/main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    // Should skip target directory because of Cargo.lock
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/main.rs"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_target_with_rust_files_no_cargo_lock() {
    let test_dir = setup_test_dir("target_with_rs");

    // No Cargo.lock

    // Create target directory with .rs file directly in target
    fs::create_dir_all(test_dir.join("target")).unwrap();
    fs::write(test_dir.join("target/test.rs"), "// test in target").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.join("src")).unwrap();
    fs::write(test_dir.join("src/main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    // Should find both files since target has .rs file and no Cargo.lock
    assert_eq!(files.len(), 2);

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_gitignore_respected() {
    let test_dir = setup_test_dir("gitignore");

    // Initialize a git repository (required for ignore crate to respect .gitignore)
    std::process::Command::new("git").args(["init"]).current_dir(&test_dir).output().ok();

    // Create .gitignore
    fs::write(test_dir.join(".gitignore"), "ignored/\n").unwrap();

    // Create ignored directory
    fs::create_dir_all(test_dir.join("ignored")).unwrap();
    fs::write(test_dir.join("ignored/file.rs"), "// ignored").unwrap();

    // Create non-ignored file
    fs::write(test_dir.join("main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    // Should only find main.rs, not the ignored file
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("main.rs"));

    cleanup_test_dir(&test_dir);
}

#[test]
fn test_nested_target_directories() {
    let test_dir = setup_test_dir("nested_targets");

    // Create Cargo.lock at root
    fs::write(test_dir.join("Cargo.lock"), "# Cargo.lock").unwrap();

    // Create nested target directories
    fs::create_dir_all(test_dir.join("target/debug/build/something/out")).unwrap();
    fs::write(test_dir.join("target/debug/build.rs"), "// build").unwrap();

    // Create workspace member without Cargo.lock
    fs::create_dir_all(test_dir.join("member/src")).unwrap();
    fs::write(test_dir.join("member/src/lib.rs"), "// member lib").unwrap();

    // Create member target (should not be skipped as no Cargo.lock beside it)
    fs::create_dir_all(test_dir.join("member/target")).unwrap();
    fs::write(test_dir.join("member/target/test.rs"), "// member target").unwrap();

    let files = collect_rust_files_recursive(&test_dir).unwrap();

    // Should skip root target but not member/target (no Cargo.lock beside it)
    assert!(files.iter().any(|f| f.ends_with("member/src/lib.rs")));
    assert!(files.iter().any(|f| f.ends_with("member/target/test.rs")));
    assert!(!files.iter().any(|f| f.to_str().unwrap().contains("target/debug")));

    cleanup_test_dir(&test_dir);
}
