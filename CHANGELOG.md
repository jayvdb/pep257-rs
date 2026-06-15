# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- New rule `D206`: docstrings should be indented with spaces, not tab characters (adapted from pydocstyle).
- Project configuration under `[workspace.metadata.pep257]` or `[package.metadata.pep257]` in `Cargo.toml`, with `select` and `ignore` lists that accept exact rule codes or prefixes. Auto-discovery walks up from the target path.
- `--config <PATH>` flag to point at a specific `Cargo.toml` or free-standing TOML config file, bypassing discovery.
- Prebuilt binaries published to GitHub Releases for Linux (gnu/musl, x86_64/aarch64), macOS (x86_64/aarch64), and Windows (x86_64/aarch64).
- Automated publishing to crates.io via trusted publishing.

### Changed

- File collector now honors `.gitignore` and skips `target/` directories when walking a path. ([#9])

## [0.2.0] - 2025-12-15

### Added

- New rules: `R101` (missing docstring on public type aliases), `R102` (missing docstring on public consts/statics), `R103` (missing docstring on public macros).
- CI workflow covering test, tidy, and a `HELP.md` freshness check. ([#6])
- `--verbose`/`--quiet` flags and `--markdown-help` for generating `HELP.md`.

### Changed

- Reorganized lint codes; `R*` codes group Rust-specific rules distinct from PEP 257's `D*` codes. ([#8])
- Simplified the CLI invocation. ([#7])
- `Violation` display format reworked for clearer output. ([#1])

### Fixed

- `D100` no longer fires on private modules. ([#2])
- `D205` and `D400` now use summary paragraph boundaries, eliminating a class of false positives. ([#3])
- Blank-line checks (`D201`/`D202`) handle more edge cases. ([#5])
- Additional false-positive fixes across the rule set.

## [0.1.1] - 2025-10-23

### Added

- Initial public release.
- Core PEP 257-derived checks (`D100`-`D106`, `D201`, `D202`, `D205`, `D400`, `D402`, `D403`, `D301`, `D401`) over `///`, `/** */`, and `#[doc = "..."]` styles.
- Text and JSON output formats.
- Project metadata, MIT license, and dogfooded docstrings on the source itself.

[Unreleased]: https://github.com/jayvdb/pep257-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/jayvdb/pep257-rs/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/jayvdb/pep257-rs/releases/tag/v0.1.1

[#1]: https://github.com/jayvdb/pep257-rs/pull/1
[#2]: https://github.com/jayvdb/pep257-rs/pull/2
[#3]: https://github.com/jayvdb/pep257-rs/pull/3
[#5]: https://github.com/jayvdb/pep257-rs/pull/5
[#6]: https://github.com/jayvdb/pep257-rs/pull/6
[#7]: https://github.com/jayvdb/pep257-rs/pull/7
[#8]: https://github.com/jayvdb/pep257-rs/pull/8
[#9]: https://github.com/jayvdb/pep257-rs/pull/9
