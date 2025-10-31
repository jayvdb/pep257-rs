/// Analyzer module for Rust documentation.
mod analyzer;
/// Parser module for extracting docstrings.
mod parser;
/// PEP 257 checker implementation.
mod pep257;

use analyzer::RustDocAnalyzer;
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use clap_verbosity_flag::Verbosity;
use pep257::Severity;
use std::path::PathBuf;
use std::process;

/// Command-line interface configuration.
#[derive(ClapParser, Debug)]
#[command(name = "pep257")]
#[command(about = "A tool to check Rust docstrings against PEP 257 conventions")]
#[command(version)]
struct Cli {
    #[command(flatten)]
    verbose: Verbosity,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Input file to check
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Show warnings in addition to errors
    #[arg(short, long)]
    warnings: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,

    /// Exit with code 0 even if violations are found
    #[arg(long)]
    no_fail: bool,

    /// Generate markdown help
    #[cfg(feature = "clap-markdown")]
    #[arg(long, hide = true)]
    markdown_help: bool,
}

/// Available subcommands for the CLI.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Check a single file
    Check {
        /// File to check
        file: PathBuf,
    },
    /// Check all Rust files in a directory
    CheckDir {
        /// Directory to check
        dir: PathBuf,
        /// Check files recursively
        #[arg(short, long)]
        recursive: bool,
    },
}

/// Output format options.
#[derive(Clone, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

/// Entry point for the application.
fn main() {
    let cli = Cli::parse();

    #[cfg(feature = "clap-markdown")]
    if cli.markdown_help {
        clap_markdown::print_help_markdown::<Cli>();
        process::exit(0);
    }

    // Initialize the logger based on verbosity level
    env_logger::Builder::new()
        .filter_level(cli.verbose.into())
        .init();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

/// Run the main logic of the application.
fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let mut analyzer = RustDocAnalyzer::new()?;
    let mut total_violations = 0;

    match &cli.command {
        Some(Commands::Check { file }) => {
            total_violations += check_file(&mut analyzer, file, &cli)?;
        }
        Some(Commands::CheckDir { dir, recursive }) => {
            total_violations += check_directory(&mut analyzer, dir, *recursive, &cli)?;
        }
        None => {
            if let Some(ref file) = cli.file {
                total_violations += check_file(&mut analyzer, file, &cli)?;
            } else {
                eprintln!("No file or command specified. Use --help for usage information.");
                process::exit(1);
            }
        }
    }

    if total_violations > 0 && !cli.no_fail {
        process::exit(1);
    }

    Ok(())
}

/// Check a single file for violations.
fn check_file(
    analyzer: &mut RustDocAnalyzer,
    file: &PathBuf,
    cli: &Cli,
) -> Result<usize, Box<dyn std::error::Error>> {
    let violations = analyzer.analyze_file(file)?;

    let filtered_violations: Vec<_> = violations
        .into_iter()
        .filter(|v| cli.warnings || matches!(v.severity, Severity::Error))
        .collect();

    match cli.format {
        OutputFormat::Text => {
            for violation in &filtered_violations {
                println!("{}:{}", file.display(), violation);
            }
        }
        OutputFormat::Json => {
            let json_output = serde_json::json!({
                "file": file.display().to_string(),
                "violations": filtered_violations.iter().map(|v| {
                    serde_json::json!({
                        "rule": v.rule,
                        "message": v.message,
                        "line": v.line,
                        "column": v.column,
                        "severity": match v.severity {
                            Severity::Error => "error",
                            Severity::Warning => "warning",
                        }
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
    }

    Ok(filtered_violations.len())
}

/// Check all files in a directory.
fn check_directory(
    analyzer: &mut RustDocAnalyzer,
    dir: &PathBuf,
    recursive: bool,
    cli: &Cli,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut total_violations = 0;

    let entries = if recursive {
        collect_rust_files_recursive(dir)?
    } else {
        collect_rust_files(dir)?
    };

    for file in entries {
        total_violations += check_file(analyzer, &file, cli)?;
    }

    Ok(total_violations)
}

/// Collect all Rust files in a directory without recursion.
fn collect_rust_files(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }

    Ok(files)
}

/// Collect Rust files in a directory recursively.
fn collect_rust_files_recursive(dir: &PathBuf) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    /// Visit a directory and collect Rust files recursively.
    fn visit_dir(
        dir: &PathBuf,
        files: &mut Vec<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                visit_dir(&path, files)?;
            } else if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
        Ok(())
    }

    visit_dir(dir, &mut files)?;
    Ok(files)
}
