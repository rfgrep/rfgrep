//! # rfgrep - High-Performance File Search Library
//!
//! A comprehensive Rust library for fast, memory-efficient file searching with advanced
//! filtering capabilities, parallel processing, and multiple search algorithms.
//!
//! ## Features
//!
//! - **Multiple Search Algorithms**: SIMD-optimized, Boyer-Moore, regex, and simple string matching
//! - **Parallel Processing**: Multi-threaded file processing with adaptive chunking
//! - **Memory Optimization**: Memory-mapped I/O, zero-copy string processing, and intelligent caching
//! - **Plugin System**: Extensible architecture with dynamic plugin loading
//! - **Interactive TUI**: Real-time search interface with live pattern editing
//! - **Streaming Search**: Handles files larger than available memory
//! - **File Type Classification**: Smart handling of 153+ file formats
//! - **Performance Monitoring**: Built-in metrics and benchmarking tools
//!
//! ## Quick Start
//!
//! ```rust
//! use rfgrep::{app_simple::RfgrepApp, Cli, Commands, SearchMode};
//! use clap::Parser;
//!
//! #[tokio::main]
//! async fn main() -> rfgrep::Result<()> {
//!     // In a real application, you would parse from command line
//!     let cli = Cli::try_parse_from(&["rfgrep", ".", "search", "pattern"]).unwrap();
//!     let app = RfgrepApp::new_async().await?;
//!     app.run(cli).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Search Algorithms
//!
//! ```rust
//! use rfgrep::search_algorithms::{SearchAlgorithm, SearchAlgorithmFactory};
//!
//! // Create a search algorithm
//! let algorithm = SearchAlgorithmFactory::create(SearchAlgorithm::BoyerMoore, "pattern");
//!
//! // Search in text
//! let matches = algorithm.search("Hello, world!", "world");
//! ```
//!
//! ## Performance Features
//!
//! - **SIMD Acceleration**: Hardware-optimized string matching
//! - **Memory Pooling**: Reuses expensive resources (mmap, compiled regex)
//! - **Adaptive Strategies**: Chooses optimal algorithm based on context
//! - **Zero-Copy Operations**: Minimizes memory allocations
//! - **Intelligent Caching**: LRU with TTL and invalidation
//!
//! ## Examples
//!
//! See the `examples/` directory for comprehensive usage examples:
//! - `real_world_demo.rs` - Real-world performance demonstration
//! - `performance_benchmark_demo.rs` - Benchmarking suite
//!
//! ## Thread Safety
//!
//! All public APIs are designed to be thread-safe and can be used in concurrent
//! environments. The library uses `Arc` and atomic operations for shared state.
//!
//! ## Performance Characteristics
//!
//! | Algorithm | Best Case | Average Case | Worst Case | Memory |
//! |-----------|-----------|--------------|------------|--------|
//! | SIMD Search | O(n) | O(n) | O(n) | O(1) |
//! | Boyer-Moore | O(n/m) | O(n/m) | O(n) | O(m) |
//! | Regex | O(n) | O(n) | O(n²) | O(m) |
//! | Zero-Copy | O(n) | O(n) | O(n) | O(1) |
//!
//! Where: n = text length, m = pattern length

#![allow(clippy::uninlined_format_args)]
#![allow(dead_code)]
#![allow(clippy::op_ref)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::borrowed_box)]
#![allow(clippy::unnecessary_map_or)]
#![allow(clippy::new_without_default)]
#![allow(unused_assignments)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::needless_borrows_for_generic_args)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::print_literal)]
// Core application modules
/// Simplified application architecture with async runtime support
pub mod app_simple;

/// Application submodules (stdin, filters, handlers, etc.)
pub mod app;

/// Command-line interface definitions and argument parsing
pub mod cli;

/// Internal configuration management
mod config;

/// Error types and result handling
pub mod error;

/// File type classification system supporting 153+ formats
pub mod file_types;

/// Test utilities and benchmarking tools (enabled for test/bench/examples features)
#[cfg(any(test, feature = "bench", feature = "examples"))]
pub mod test_utils;

/// Interactive search mode implementation
mod interactive;

/// File listing and information utilities
pub mod list;

/// Memory management and optimization
mod memory;

/// Performance metrics and monitoring
pub mod metrics;

/// Output format implementations (JSON, XML, HTML, Markdown)
mod output_formats;

/// High-performance optimization modules
///
/// Includes:
/// - Memory pooling and caching
/// - Parallel processing with adaptive chunking
/// - Zero-copy string processing
/// - Optimized memory-mapped I/O
pub mod performance;

/// Plugin command-line interface
pub mod plugin_cli;

/// Extensible plugin system with dynamic loading
pub mod plugin_system;

/// Core file processing and search logic
pub mod processor;

/// Progress tracking and reporting
mod progress;

/// Search algorithm implementations
mod search;

/// Multiple search algorithms (SIMD, Boyer-Moore, Regex, Simple)
pub mod search_algorithms;

/// Streaming search pipeline for large files
pub mod streaming_search;

/// Interactive Terminal User Interface
pub mod tui;

/// Directory walking and file discovery
pub mod walker;
use crate::config::Config;

// Re-export commonly used types for convenience
/// Result type alias for rfgrep operations
pub use crate::error::Result;

/// Command-line argument parser from clap
pub use clap::Parser;

/// CLI definitions and search modes
pub use cli::{Cli, Commands, SearchMode};

/// File information structure for listing operations
pub use list::FileInfo;

/// Core file processing functions
pub use processor::{is_binary, search_file};

/// Search algorithm implementations and utilities
pub use search_algorithms::{
    BoyerMoore, RegexSearch, SearchAlgorithm, SearchAlgorithmFactory, SearchMatch, SimdSearch,
    SimpleSearch,
};

use std::path::Path;

/// Path buffer type for file operations
pub use std::path::PathBuf;

/// Directory walking functionality
pub use walker::walk_dir;
/// Application configuration for rfgrep operations
///
/// Contains runtime configuration including chunk sizes, executable paths,
/// and output directories for benchmarking and testing operations.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Chunk size for parallel processing (in number of files)
    pub chunk_size: Option<u32>,

    /// Path to the rfgrep executable
    pub rfgrep_exe: PathBuf,

    /// Directory for storing benchmark results and test outputs
    pub results_dir: PathBuf,
}

impl AppConfig {
    /// Create application configuration from CLI arguments
    ///
    /// # Arguments
    /// * `cli` - Parsed command-line interface arguments
    ///
    /// # Returns
    /// * `AppConfig` - Configured application settings
    ///
    /// # Panics
    /// Panics if the results directory cannot be created
    pub fn from_cli(cli: &Cli) -> Self {
        let rfgrep_exe = cli.path.join("rfgrep");
        let results_dir = cli.path.join("results");
        std::fs::create_dir_all(&results_dir).expect("Failed to create results directory");

        AppConfig {
            chunk_size: Some(100),
            rfgrep_exe,
            results_dir,
        }
    }
}

/// Load application configuration from file or use defaults
///
/// Attempts to load configuration from the default location, falling back
/// to default values if no configuration file is found.
///
/// # Returns
/// * `AppConfig` - Loaded or default configuration
pub fn load_config() -> AppConfig {
    let mut cfg = Config::default();
    if let Ok(config) = Config::load() {
        cfg = config;
    }
    AppConfig {
        chunk_size: Some(cfg.search.chunk_size as u32),
        rfgrep_exe: std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("rfgrep")),
        results_dir: std::path::PathBuf::from("results"),
    }
}

/// Execute an external command with optional environment variables
///
/// # Arguments
/// * `command` - Command to execute
/// * `args` - Command arguments
/// * `env` - Optional environment variable value
///
/// # Returns
/// * `std::io::Result<()>` - Success or I/O error
///
/// # Example
/// ```no_run
/// use rfgrep::run_external_command;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Run a simple command
///     run_external_command("echo", &["hello"], None)?;
///
///     // Run with environment variable
///     run_external_command("env", &[], Some("test_value"))?;
///     Ok(())
/// }
/// ```
pub fn run_external_command(
    command: &str,
    args: &[&str],
    env: Option<&str>,
) -> std::io::Result<()> {
    let mut cmd = std::process::Command::new(command);
    cmd.args(args);
    if let Some(env_var) = env {
        cmd.env("RFGREP_TEST_ENV", env_var);
    }
    cmd.status()?;
    Ok(())
}

/// Run comprehensive performance benchmarks using hyperfine
///
/// Executes a series of performance tests including warmup and detailed
/// benchmarking with JSON and Markdown output formats.
///
/// # Arguments
/// * `config` - Application configuration with paths and settings
/// * `test_dir` - Directory containing test files for benchmarking
///
/// # Returns
/// * `Result<()>` - Success or error during benchmark execution
///
/// # Example
/// ```no_run
/// use rfgrep::{AppConfig, run_benchmarks};
/// use std::path::Path;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create config manually for doctest
///     let config = AppConfig {
///         chunk_size: Some(100),
///         rfgrep_exe: std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("rfgrep")),
///         results_dir: std::path::PathBuf::from("results"),
///     };
///     let test_dir = Path::new("/path/to/test/files");
///     run_benchmarks(&config, test_dir)?;
///     Ok(())
/// }
/// ```
pub fn run_benchmarks(config: &AppConfig, test_dir: &Path) -> Result<()> {
    println!("Warming up rfgrep...");
    run_external_command(
        config.rfgrep_exe.to_str().unwrap(),
        &["search", "xyz123", test_dir.to_str().unwrap()],
        None,
    )?;

    println!("Running search performance benchmarks...");
    run_external_command(
        "hyperfine",
        &[
            "--warmup",
            "3",
            "--export-json",
            config.results_dir.join("search.json").to_str().unwrap(),
            "--export-markdown",
            config.results_dir.join("search.md").to_str().unwrap(),
            config.rfgrep_exe.to_str().unwrap(),
            "search",
            "pattern1",
            test_dir.to_str().unwrap(),
        ],
        None,
    )?;

    Ok(())
}

/// Run benchmarks from CLI arguments with automatic test data setup
///
/// Convenience function that creates test data directory if needed and
/// runs comprehensive benchmarks using the provided CLI configuration.
///
/// # Arguments
/// * `cli` - Parsed command-line interface arguments
///
/// # Returns
/// * `Result<()>` - Success or error during benchmark execution
///
/// # Example
/// ```no_run
/// use rfgrep::{Cli, run_benchmarks_cli};
/// use clap::Parser;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // In a real application, you would parse from command line
///     let cli = Cli::try_parse_from(&["rfgrep", ".", "search", "pattern"]).unwrap();
///     run_benchmarks_cli(&cli)?;
///     Ok(())
/// }
/// ```
pub fn run_benchmarks_cli(cli: &Cli) -> Result<()> {
    let config = AppConfig::from_cli(cli);
    let test_dir = cli.path.join("test_data");

    if !test_dir.exists() {
        std::fs::create_dir_all(&test_dir).map_err(crate::error::RfgrepError::Io)?;
    }

    run_benchmarks(&config, &test_dir)
}
