//! Configuration loading and rule filtering.
//!
//! Config lives in `Cargo.toml`, under either `[workspace.metadata.pep257]` or
//! `[package.metadata.pep257]` — the standard cargo convention for third-party
//! tool configuration. Discovery walks upward from the target path looking for
//! a `Cargo.toml`; `--config <PATH>` overrides discovery entirely and may point
//! to any TOML file (a `Cargo.toml`, or a free-standing file whose root keys
//! are the pep257 config itself).
//!
//! Example `Cargo.toml`:
//!
//! ```toml
//! [workspace.metadata.pep257]
//! select = ["D"]      # all D-rules
//! ignore = ["D401"]   # except D401
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use thiserror::Error;

/// The cargo manifest filename searched during auto-discovery.
pub const CARGO_MANIFEST: &str = "Cargo.toml";

/// Errors raised while loading a config file.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// The explicit `--config` path could not be read.
    #[error("config file not found: {0}")]
    NotFound(PathBuf),
    /// The config file content could not be parsed as TOML.
    #[error("failed to parse config {path}: {source}")]
    Parse {
        /// Path of the offending file.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// An IO error occurred while reading the file.
    #[error("failed to read config {path}: {source}")]
    Io {
        /// Path of the offending file.
        path: PathBuf,
        /// Underlying IO error.
        #[source]
        source: std::io::Error,
    },
}

/// Parsed pep257 config.
#[derive(Debug, Default, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Rule codes (or code prefixes) to include. Empty means "include all".
    #[serde(default)]
    pub select: Vec<String>,
    /// Rule codes (or code prefixes) to exclude. Always wins over `select`.
    #[serde(default)]
    pub ignore: Vec<String>,
}

/// Wrapper for extracting `[workspace.metadata.pep257]` / `[package.metadata.pep257]`
/// out of a `Cargo.toml`.
#[derive(Debug, Default, Deserialize)]
struct CargoManifest {
    #[serde(default)]
    workspace: Option<TableWithMetadata>,
    #[serde(default)]
    package: Option<TableWithMetadata>,
}

#[derive(Debug, Default, Deserialize)]
struct TableWithMetadata {
    #[serde(default)]
    metadata: Option<MetadataTable>,
}

#[derive(Debug, Default, Deserialize)]
struct MetadataTable {
    #[serde(default)]
    pep257: Option<Config>,
}

impl Config {
    /// Load config based on an optional `--config` override and a starting path.
    ///
    /// When `explicit` is `Some`, that path is loaded directly; missing file is an error.
    /// When `explicit` is `None`, walks up from `start` looking for `Cargo.toml`; the
    /// nearest manifest containing a pep257 metadata table wins. If no manifest has one,
    /// returns a default (empty) config.
    pub fn load(explicit: Option<&Path>, start: &Path) -> Result<Self, ConfigError> {
        if let Some(path) = explicit {
            if !path.is_file() {
                return Err(ConfigError::NotFound(path.to_path_buf()));
            }
            return Self::read_any(path);
        }

        let start = if start.is_file() { start.parent().unwrap_or(Path::new(".")) } else { start };
        let mut current = Some(start);
        while let Some(dir) = current {
            let manifest = dir.join(CARGO_MANIFEST);
            if manifest.is_file()
                && let Some(cfg) = Self::from_cargo_manifest(&manifest)?
            {
                return Ok(cfg);
            }
            current = dir.parent();
        }

        Ok(Self::default())
    }

    /// Read a TOML file that may be either a `Cargo.toml` (extract metadata.pep257)
    /// or a free-standing config file (root keys are the config).
    fn read_any(path: &Path) -> Result<Self, ConfigError> {
        if path.file_name().and_then(|n| n.to_str()) == Some(CARGO_MANIFEST) {
            return Self::from_cargo_manifest(path)?
                .ok_or_else(|| ConfigError::NotFound(path.to_path_buf()));
        }

        let text = read_to_string(path)?;
        toml::from_str(&text)
            .map_err(|source| ConfigError::Parse { path: path.to_path_buf(), source })
    }

    /// Parse a `Cargo.toml` and extract the pep257 metadata table, if any.
    ///
    /// `[workspace.metadata.pep257]` takes precedence over `[package.metadata.pep257]`
    /// when both are present in the same manifest.
    fn from_cargo_manifest(path: &Path) -> Result<Option<Self>, ConfigError> {
        let text = read_to_string(path)?;
        let manifest: CargoManifest = toml::from_str(&text)
            .map_err(|source| ConfigError::Parse { path: path.to_path_buf(), source })?;
        Ok(manifest
            .workspace
            .and_then(|w| w.metadata)
            .and_then(|m| m.pep257)
            .or_else(|| manifest.package.and_then(|p| p.metadata).and_then(|m| m.pep257)))
    }

    /// Return `true` when a violation with the given rule code should be reported.
    ///
    /// A rule passes when it matches at least one `select` entry (or `select` is empty,
    /// meaning "everything") AND does not match any `ignore` entry. Matching is by exact
    /// code or by prefix (so `"D"` matches `D100`, `"D2"` matches `D200`/`D205`, etc.).
    #[must_use]
    pub fn allows(&self, rule: &str) -> bool {
        let selected = self.select.is_empty() || self.select.iter().any(|s| matches_code(rule, s));
        let ignored = self.ignore.iter().any(|s| matches_code(rule, s));
        selected && !ignored
    }
}

fn read_to_string(path: &Path) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|source| ConfigError::Io { path: path.to_path_buf(), source })
}

fn matches_code(rule: &str, pattern: &str) -> bool {
    rule == pattern || rule.starts_with(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_allows_everything() {
        let cfg = Config::default();
        assert!(cfg.allows("D100"));
        assert!(cfg.allows("R401"));
    }

    #[test]
    fn ignore_drops_an_exact_code() {
        let cfg = Config { select: vec![], ignore: vec!["D401".into()] };
        assert!(!cfg.allows("D401"));
        assert!(cfg.allows("D400"));
    }

    #[test]
    fn ignore_prefix_drops_a_whole_family() {
        let cfg = Config { select: vec![], ignore: vec!["R".into()] };
        assert!(!cfg.allows("R101"));
        assert!(!cfg.allows("R402"));
        assert!(cfg.allows("D100"));
    }

    #[test]
    fn select_restricts_to_listed_rules() {
        let cfg = Config { select: vec!["D1".into()], ignore: vec![] };
        assert!(cfg.allows("D100"));
        assert!(cfg.allows("D103"));
        assert!(!cfg.allows("D200"));
        assert!(!cfg.allows("R101"));
    }

    #[test]
    fn ignore_wins_over_select() {
        let cfg = Config { select: vec!["D".into()], ignore: vec!["D401".into()] };
        assert!(cfg.allows("D400"));
        assert!(!cfg.allows("D401"));
    }

    #[test]
    fn rejects_unknown_keys_in_pep257_section() {
        let result: Result<Config, _> = toml::from_str("banana = true");
        assert!(result.is_err());
    }

    #[test]
    fn loads_workspace_metadata_from_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"
[workspace]
members = ["."]
[workspace.metadata.pep257]
ignore = ["D401"]
"#,
        )
        .unwrap();
        let cfg = Config::load(None, tmp.path()).unwrap();
        assert_eq!(cfg.ignore, vec!["D401".to_string()]);
    }

    #[test]
    fn loads_package_metadata_when_workspace_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"
[package]
name = "x"
version = "0.1.0"
edition = "2024"
[package.metadata.pep257]
select = ["D2"]
"#,
        )
        .unwrap();
        let cfg = Config::load(None, tmp.path()).unwrap();
        assert_eq!(cfg.select, vec!["D2".to_string()]);
    }

    #[test]
    fn workspace_metadata_wins_over_package_metadata_in_same_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"
[workspace]
members = ["."]
[workspace.metadata.pep257]
select = ["D"]
[package]
name = "x"
version = "0.1.0"
edition = "2024"
[package.metadata.pep257]
select = ["R"]
"#,
        )
        .unwrap();
        let cfg = Config::load(None, tmp.path()).unwrap();
        assert_eq!(cfg.select, vec!["D".to_string()]);
    }

    #[test]
    fn discover_walks_up_to_find_cargo_toml() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("a/b/c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["."]
[workspace.metadata.pep257]
ignore = ["D206"]
"#,
        )
        .unwrap();

        let cfg = Config::load(None, &nested).unwrap();
        assert_eq!(cfg.ignore, vec!["D206".to_string()]);
    }

    #[test]
    fn cargo_toml_without_pep257_metadata_yields_default() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(
            tmp.path().join("Cargo.toml"),
            r#"
[package]
name = "x"
version = "0.1.0"
edition = "2024"
"#,
        )
        .unwrap();
        let cfg = Config::load(None, tmp.path()).unwrap();
        assert_eq!(cfg, Config::default());
    }

    #[test]
    fn explicit_path_to_cargo_toml_works() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest = tmp.path().join("Cargo.toml");
        fs::write(
            &manifest,
            r#"
[workspace.metadata.pep257]
ignore = ["D206"]
"#,
        )
        .unwrap();
        let cfg = Config::load(Some(&manifest), Path::new(".")).unwrap();
        assert_eq!(cfg.ignore, vec!["D206".to_string()]);
    }

    #[test]
    fn explicit_path_to_freestanding_file_works() {
        let tmp = tempfile::tempdir().unwrap();
        let cfg_file = tmp.path().join("custom.toml");
        fs::write(&cfg_file, r#"ignore = ["D206"]"#).unwrap();
        let cfg = Config::load(Some(&cfg_file), Path::new(".")).unwrap();
        assert_eq!(cfg.ignore, vec!["D206".to_string()]);
    }

    #[test]
    fn explicit_path_missing_errors() {
        let result = Config::load(Some(Path::new("/does/not/exist/Cargo.toml")), Path::new("."));
        assert!(matches!(result, Err(ConfigError::NotFound(_))));
    }
}
