//! File collection module for finding Rust source files.

use std::path::PathBuf;

/// Check if a directory should be skipped based on target directory rules.
/// Returns true if the directory should be skipped.
pub(crate) fn should_skip_target_dir(path: &std::path::Path) -> bool {
    // Rule 2: If directory name is "target" and has no .rs files, skip it
    if path.file_name().and_then(|n| n.to_str()) == Some("target") {
        // Check if there are any .rs files directly in this target directory
        if let Ok(entries) = std::fs::read_dir(path) {
            let has_rust_files = entries.filter_map(Result::ok).any(|e| {
                let path = e.path();
                path.is_file() && path.extension().is_some_and(|ext| ext == "rs")
            });

            if !has_rust_files {
                return true;
            }
        }
    }

    // Rule 3: If parent directory has Cargo.lock, and this is a target directory, skip it
    if path.file_name().and_then(|n| n.to_str()) == Some("target")
        && let Some(parent) = path.parent()
    {
        let cargo_lock = parent.join("Cargo.lock");
        if cargo_lock.exists() {
            return true;
        }
    }

    false
}

/// Collect Rust files in a directory recursively using the ignore crate.
/// This respects .gitignore files and applies custom target directory filtering.
pub fn collect_rust_files_recursive(
    dir: &PathBuf,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    // Use ignore::WalkBuilder which respects .gitignore, .ignore files, etc.
    let walker = ignore::WalkBuilder::new(dir)
        .standard_filters(true)  // Enable standard ignore filters (.gitignore, etc.)
        .filter_entry(|entry| {
            let path = entry.path();

            // Apply custom target directory filtering
            if path.is_dir() && should_skip_target_dir(path) {
                return false;
            }

            true
        })
        .build();

    for result in walker {
        let entry = result?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}
