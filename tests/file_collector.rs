//! Integration tests for file collection functionality.

use std::fs;

use pep257::file_collector::collect_rust_files_recursive;
use tempfile::TempDir;

#[test]
fn test_collect_basic_rust_files() {
    let test_dir = TempDir::new().unwrap();

    // Create some .rs files
    fs::write(test_dir.path().join("file1.rs"), "// test file 1").unwrap();
    fs::write(test_dir.path().join("file2.rs"), "// test file 2").unwrap();
    fs::write(test_dir.path().join("other.txt"), "// not rust").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    assert_eq!(files.len(), 2);
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file1.rs"));
    assert!(files.iter().any(|f| f.file_name().unwrap() == "file2.rs"));
}

#[test]
fn test_recursive_collection() {
    let test_dir = TempDir::new().unwrap();

    // Create nested structure
    fs::create_dir_all(test_dir.path().join("src")).unwrap();
    fs::create_dir_all(test_dir.path().join("src/module")).unwrap();

    fs::write(test_dir.path().join("main.rs"), "// main").unwrap();
    fs::write(test_dir.path().join("src/lib.rs"), "// lib").unwrap();
    fs::write(test_dir.path().join("src/module/mod.rs"), "// mod").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    assert_eq!(files.len(), 3);
}

#[test]
fn test_skip_target_without_rust_files() {
    let test_dir = TempDir::new().unwrap();

    // Create target directory without .rs files
    fs::create_dir_all(test_dir.path().join("target/debug")).unwrap();
    fs::write(test_dir.path().join("target/debug/binary"), "binary data").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.path().join("src")).unwrap();
    fs::write(test_dir.path().join("src/lib.rs"), "// lib").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    // Should only find the src file, not traverse into target
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/lib.rs"));
}

#[test]
fn test_target_with_cargo_lock() {
    let test_dir = TempDir::new().unwrap();

    // Create Cargo.lock at project root
    fs::write(test_dir.path().join("Cargo.lock"), "# Cargo.lock").unwrap();

    // Create target directory (should be skipped)
    fs::create_dir_all(test_dir.path().join("target/debug")).unwrap();
    fs::write(test_dir.path().join("target/debug/build.rs"), "// build script").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.path().join("src")).unwrap();
    fs::write(test_dir.path().join("src/main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    // Should skip target directory because of Cargo.lock
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/main.rs"));
}

#[test]
fn test_target_with_rust_files_no_cargo_lock() {
    let test_dir = TempDir::new().unwrap();

    // No Cargo.lock

    // Create target directory with .rs file directly in target
    fs::create_dir_all(test_dir.path().join("target")).unwrap();
    fs::write(test_dir.path().join("target/test.rs"), "// test in target").unwrap();

    // Create src with rust files
    fs::create_dir_all(test_dir.path().join("src")).unwrap();
    fs::write(test_dir.path().join("src/main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    // Should find both files since target has .rs file and no Cargo.lock
    assert_eq!(files.len(), 2);
}

#[test]
fn test_gitignore_respected() {
    let test_dir = TempDir::new().unwrap();

    // Initialize a git repository (required for ignore crate to respect .gitignore)
    std::process::Command::new("git").args(["init"]).current_dir(test_dir.path()).output().ok();

    // Create .gitignore
    fs::write(test_dir.path().join(".gitignore"), "ignored/\n").unwrap();

    // Create ignored directory
    fs::create_dir_all(test_dir.path().join("ignored")).unwrap();
    fs::write(test_dir.path().join("ignored/file.rs"), "// ignored").unwrap();

    // Create non-ignored file
    fs::write(test_dir.path().join("main.rs"), "// main").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    // Should only find main.rs, not the ignored file
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("main.rs"));
}

#[test]
fn test_nested_target_directories() {
    let test_dir = TempDir::new().unwrap();

    // Create Cargo.lock at root
    fs::write(test_dir.path().join("Cargo.lock"), "# Cargo.lock").unwrap();

    // Create nested target directories
    fs::create_dir_all(test_dir.path().join("target/debug/build/something/out")).unwrap();
    fs::write(test_dir.path().join("target/debug/build.rs"), "// build").unwrap();

    // Create workspace member without Cargo.lock
    fs::create_dir_all(test_dir.path().join("member/src")).unwrap();
    fs::write(test_dir.path().join("member/src/lib.rs"), "// member lib").unwrap();

    // Create member target (should not be skipped as no Cargo.lock beside it)
    fs::create_dir_all(test_dir.path().join("member/target")).unwrap();
    fs::write(test_dir.path().join("member/target/test.rs"), "// member target").unwrap();

    let files = collect_rust_files_recursive(&test_dir.path().to_path_buf()).unwrap();

    // Should skip root target but not member/target (no Cargo.lock beside it)
    assert!(files.iter().any(|f| f.ends_with("member/src/lib.rs")));
    assert!(files.iter().any(|f| f.ends_with("member/target/test.rs")));
    assert!(!files.iter().any(|f| f.to_str().unwrap().contains("target/debug")));
}
