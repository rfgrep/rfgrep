# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [0.5.0] - 2026-02-14


### Added
- Compatibility with latest Rust and dependency ecosystem (2026).
- Improved test data generation utilities for more robust testing.

### Changed
- Updated `libc` dependency to resolve version conflict with `nix`.
- Updated man page version and release date.

### Fixed
- Fixed test failures due to missing `rand::RngExt` import.
- All Clippy and test warnings addressed for release.

## [0.4.0] - 2025-10-15

### Added

- **Unix Pipeline Integration**:
    - Automatic pipe detection using `is_terminal::is_terminal()`
    - `--quiet` / `-q` flag for explicit output suppression
    - Smart message suppression when output is piped
    - Seamless integration with grep, awk, sed, xargs, cut, sort, and other Unix tools
- **New Output Formats**:
    - CSV output format (`--output-format csv`) with proper character escaping
    - TSV output format (`--output-format tsv`) with tab/newline escaping
- **New Search Modes**:
    - Count-only mode (`--count` / `-c`) - like `grep -c`
    - Files-with-matches mode (`--files-with-matches` / `-l`) - like `grep -l`
- **Performance Profiling Infrastructure**:
    - `scripts/quick_profile.sh` - Fast performance testing with hyperfine
    - `scripts/profile_benchmarks.sh` - Comprehensive profiling with flamegraph support
- **Comprehensive Documentation**:
    - `docs/SHELL_PIPELINE_EXAMPLES.md` - 50+ pipeline integration examples (600+ lines)
    - `docs/OPTIMIZATION_SUGGESTIONS.md` - Performance optimization roadmap (450+ lines)
    - `docs/SESSION_SUMMARY.md` - Complete development session record (800+ lines)

### Changed

- Output automatically suppressed when piped to other commands
- Verbose messages respect quiet flag and pipe detection
- Benchmark test data reduced for stability (100→20 small, 20→5 medium, 5→2 large files)
- Benchmark file sizes reduced (100KB→10KB medium, 1MB→100KB large)

### Fixed

- **NDJSON Output Bug**: `--ndjson` flag now correctly outputs newline-delimited JSON instead of
  text
- **List Extension Filter Bug**: `rfgrep list --extensions rs` now correctly filters to only .rs
  files using case-insensitive comparison
- **Benchmark Memory Issues**: Fixed SIGKILL errors by creating single Tokio runtime per benchmark
  function instead of per iteration

### Performance

- Established performance baseline: 5.5ms for small repos, 41.6ms for 1000 files
- Identified optimization opportunities for 40-50% improvement potential
- CSV/TSV output adds minimal overhead (<0.2ms)
- Count mode optimized for fast counting without full result formatting

### Documentation

- Added 50+ Unix pipeline integration examples
- Created performance optimization roadmap with 8 detailed recommendations
- Documented profiling infrastructure and benchmarking strategies
- Complete session summary with all changes and metrics

## [0.3.1] - 2025-09-23

### Added
- Performance module upgrades:
  - Atomic metrics (`AtomicPerformanceMetrics`) with thread-safe counters
  - Timing helpers in `PerformanceMonitor` and `MemoryTracker` stats API
- Parallel processing:
  - Adaptive chunking based on CPU cores and memory pressure
  - Processing stats with memory pressure levels
- I/O optimizations:
  - Optimized memory-mapped I/O handler with fallback strategies
  - Memory pool for `memmap2::Mmap` with eviction and pressure-aware cleanup
- Zero-copy processing utilities for string/line handling
- Enhanced benchmarks (Criterion): broader scenarios and stability fixes
- TUI input modes (Normal/Search/Command) with live editing

### Changed
- `RfgrepApp` and `TuiApp` initialization made async-friendly; uses existing Tokio runtime when present
- Updated `ratatui` to 0.29; added `atty` terminal checks
- Regex cache switched to `Mutex<HashMap<..>>` for simplicity and compatibility

### Fixed
- Resolved build issues related to `dashmap` dependency removal
- Addressed clippy warnings across performance and optimized I/O modules
- Bench harness no longer relies on gated test utilities; portable data generation used
- Removed snap build from CI workflows to simplify release process

### Performance
- Faster parallel file processing under load due to adaptive chunking
- Lower peak memory via pooled mmaps and streaming fallback under pressure
- Fewer allocations with zero-copy slices along hot paths

### Security
- Documented allowance for `paste` advisory transitively via `ratatui` (RUSTSEC-2024-0436)

### Docs
- Expanded crate and module docs (`lib.rs`), new blog posts and performance docs

## [0.3.0] - 2025-09-15

### Added
- **Comprehensive File Type Classification System**: Support for 153 file formats across 4 categories
  - Always Search (74 formats): Plain text, source code, configuration files
  - Conditional Search (41 formats): Office documents, archives, media files
  - Skip by Default (27 formats): Executables and system files
  - Never Search (11 formats): Dangerous/irrelevant files
- **Smart Search Modes**: 
  - FullText: Complete file content search
  - Metadata: File headers and properties search
  - Filename: Archive contents by filename
  - Structured: JSON/XML/YAML parsing
- **CLI Enhancements**:
  - `--file-types <strategy>`: Control file type handling (default/comprehensive/conservative/performance)
  - `--include-extensions <extensions>`: Override to include specific file types
  - `--exclude-extensions <extensions>`: Override to exclude specific file types
  - `--search-all-files`: Search all file types (comprehensive mode)
  - `--text-only`: Only search text files (conservative mode)
  - `--safety-policy <policy>`: Safety policies (default/conservative/performance)
  - `--threads <count>`: Number of parallel processing threads
- **New Commands**:
  - `rfgrep simulate`: Performance testing and benchmarking
- **Enhanced Binary Detection**:
  - UTF-16 BOM detection (LE/BE)
  - UTF-8 BOM support
  - UTF-16 pattern recognition
  - Reduced false positives for text files
- **Size Limits and Safety**:
  - Intelligent size limits based on file type
  - Memory protection with configurable policies
  - Safety-first approach for large files
- **Comprehensive Documentation**:
  - `API_REFERENCE.md`: Complete API documentation
  - `LIBRARY_DOCUMENTATION.md`: Library usage guide
  - `DESIGN_OPTIMIZATION.md`: Future roadmap and optimization plans
  - `BRANCHING_STRATEGY.md`: Development workflow and branching strategy
  - `SNAP_RELEASE.md`: Snap package release guide

### Changed
- **File Processing**: Enhanced file type detection with MIME type fallback
- **Memory Management**: Better size-based filtering and memory protection
- **Error Handling**: Improved error messages and validation
- **Performance**: Optimized file filtering and search algorithms
- **CLI Interface**: More intuitive and powerful command-line options

### Fixed
- **Binary Detection**: Fixed UTF-16 file misclassification as binary
- **Clippy Warnings**: Resolved needless_borrow warning in file_types.rs
- **Memory Usage**: Controlled memory consumption with size limits
- **File Type Recognition**: Better handling of various file formats

### Performance
- **File Format Support**: +665% increase (20 → 153 formats)
- **Binary Detection Accuracy**: +10% improvement (85% → 95%)
- **Search Modes**: +300% increase (1 → 4 modes)
- **Memory Safety**: Controlled memory usage with intelligent limits

### Documentation
- **README.md**: Updated with new features and examples
- **Man Pages**: Enhanced with new CLI options
- **API Documentation**: Comprehensive library documentation
- **Developer Guides**: Detailed development and contribution guides

## [0.2.1] - 2025-08-24

  ### Added
  - Structured search results: `search_file` now returns `SearchMatch` objects that include the file `path`, `line_number`, `matched_text`, and surrounding context (`context_before` / `context_after`).

  ### Changed
  - Removed `anyhow` across the crate and standardized on `crate::error::Result<T>` and `RfgrepError` as the canonical error type.
  - JSON output now serializes structured `SearchMatch` objects for machine-friendly results.
  - CI workflow (`.github/workflows/ci.yml`) rewritten to run a matrix of tests (Linux/macOS/Windows), enforce formatting and clippy, build platform artifacts, and provide an optional integration harness under Xvfb.

  ### Fixed
  - Clippy and style fixes across the codebase; ran `cargo fmt` and resolved warnings.
  - Updated scripts (e.g. `scripts/run_benchmarks.rs`) to avoid reintroducing `anyhow`.

  ### Docs
  - Tidied man pages (prefer `--path` in `man/rfgrep.1`) and updated release notes.

  ## [0.2.0] - 2025-08-05

  ### Added
  - **Interactive Search Mode**: Real-time search with filtering and navigation
    - Interactive command-line interface with keyboard shortcuts
    - Real-time result filtering and refinement
    - Context viewing and result navigation
    - Search statistics and performance metrics
  - **Advanced Search Algorithms**: Multiple search algorithm support
    - Boyer-Moore algorithm for fast plain text search
    - Regex algorithm for pattern matching
    - Simple linear search as fallback option
    - Unified search algorithm trait for extensibility
  - **Multiple Output Formats**: Support for various output formats
    - Text format with colored highlighting (default)
    - JSON format for programmatic processing
    - XML format for structured data
    - HTML format for web display
    - Markdown format for documentation
  - **Adaptive Memory Management**: Intelligent memory usage optimization
    - Dynamic memory mapping thresholds based on system resources
    - Adaptive chunk sizing for parallel processing
    - Memory usage monitoring and optimization
    - Configurable performance settings
  - **Comprehensive Man Pages**: Professional documentation system
    - Main man page (`rfgrep.1`) with complete overview
    - Command-specific man pages for all subcommands
    - Detailed examples and performance tips
    - Troubleshooting guides and best practices
  - **Shell Completion Support**: Tab completion for all major shells
    - Bash completion with command and option completion
    - Zsh completion with descriptions and fuzzy matching
    - Fish completion with built-in support
    - PowerShell completion for cross-platform support
    - Elvish completion for modern shell experience
  - **Enhanced CLI Interface**: Improved command-line experience
    - Detailed help messages with examples
    - Better error handling and user feedback
    - Progress indicators and status updates
    - Verbose logging and debugging options
  - **Installation and Testing Tools**: Professional deployment system
    - Makefile for easy man page installation
    - Automated testing scripts for completions and man pages
    - Comprehensive installation guide
    - Verification and troubleshooting tools

  ### Changed
  - **Performance Optimizations**: Improved search and processing speed
    - Enhanced memory mapping for large files
    - Optimized parallel processing with adaptive chunking
    - Better binary file detection and skipping
    - Improved regex caching and compilation
    - **Error Handling**: More robust error management
    - Better error messages and user feedback
    - Graceful handling of file system errors
    - Improved logging and debugging capabilities
    - **Documentation**: Enhanced user experience
    - Updated README with comprehensive installation instructions
    - Added troubleshooting guides and performance tips
    - Improved help messages and examples
    - Better cross-references between man pages

  ### Fixed
  - **Compilation Issues**: Resolved dependency and build problems
    - Fixed indicatif dependency version conflicts
    - Resolved serde_json import issues
    - Fixed man page formatting and syntax errors
    - Corrected regex pattern escaping in examples
  - **Runtime Errors**: Improved stability and reliability
    - Fixed index out of bounds in Boyer-Moore algorithm
    - Resolved interactive mode display issues
    - Fixed memory management edge cases
    - Corrected completion script generation

  ## [0.1.0] - 2025-06-23

  ### Added
  - Initial implementation of recursive file search functionality
  - Core features:
    - Recursive directory traversal
    - Regex/text/whole-word search modes
    - File extension filtering
    - Binary file detection
    - Size-based filtering
  - Cross-platform support (Windows, macOS, Linux)
  - GitHub Actions CI/CD pipeline
  - Comprehensive documentation
  - Man pages and shell completions


[Unreleased]: https://github.com/rfgrep/rfgrep/compare/v0.5.0...HEAD

[0.5.0]: https://github.com/rfgrep/rfgrep/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/rfgrep/rfgrep/compare/v0.3.1...v0.4.0
  [0.3.1]: https://github.com/rfgrep/rfgrep/compare/v0.3.0...v0.3.1
  [0.3.0]: https://github.com/rfgrep/rfgrep/compare/v0.2.1...v0.3.0
  [0.2.1]: https://github.com/rfgrep/rfgrep/compare/v0.2.0...v0.2.1
  [0.2.0]: https://github.com/rfgrep/rfgrep/compare/v0.1.0...v0.2.0
  [0.1.0]: https://github.com/rfgrep/rfgrep/releases/tag/v0.1.0
