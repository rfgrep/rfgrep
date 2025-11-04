use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(
    name = "rfgrep",
    author = "Khalid Hussein <kh3rld.hussein@gmail.com>",
    version,
    about = "Recursive file grep utility with advanced filtering - search, list, and analyze text files with regex support",
    long_about = r#"
rfgrep - A powerful command-line utility for recursively searching and listing files with advanced filtering capabilities.

FEATURES:
  • Advanced Search: Regex, plain text, and whole-word matching
  • File Listing: Detailed/simple output formats with extension statistics
  • Performance: Parallel processing with memory mapping for large files
  • Filtering: Extension, size, and binary file filtering
  • Utilities: Clipboard copy, dry-run mode, and progress indicators

EXAMPLES:
  # Search for "HashMap" in Rust files
  rfgrep search "HashMap" --extensions rs

  # List all Markdown files under 1MB
  rfgrep list --extensions md --max-size 1

  # Search with regex and copy to clipboard
  rfgrep search "fn\s+\w+\s*\(" regex --copy

  # Recursive search with word boundaries
  rfgrep search "test" word --recursive --extensions rs

PERFORMANCE TIPS:
  • Use --skip-binary to avoid unnecessary file checks
  • Limit scope with --extensions and --max-size
  • Use --dry-run first to preview files
  • Enable --recursive for deep directory traversal
"#,
    after_help = r#"
For more information, visit: https://github.com/kh3rld/rfgrep
"#
)]
pub struct Cli {
    #[clap(default_value = ".")]
    pub path: PathBuf,

    #[clap(long, value_parser, default_value_t = false, global = true)]
    pub verbose: bool,

    /// Suppress all non-essential output (useful for piping)
    #[clap(
        long,
        short = 'q',
        value_parser,
        default_value_t = false,
        global = true
    )]
    pub quiet: bool,

    #[clap(long, value_enum, default_value_t = ColorChoice::Auto, global = true)]
    pub color: ColorChoice,

    #[clap(long, value_parser, global = true)]
    pub log: Option<PathBuf>,

    #[clap(long, value_parser, default_value_t = false, global = true)]
    pub dry_run: bool,

    /// Allow running as root (disabled by default for safety)
    #[clap(long, value_parser, default_value_t = false, global = true)]
    pub allow_root: bool,

    #[clap(long, value_parser, global = true)]
    pub max_size: Option<usize>,

    #[clap(long, value_parser, default_value_t = false, global = true)]
    pub skip_binary: bool,

    /// Safety policy for file processing
    #[clap(long, value_enum, default_value_t = SafetyPolicy::Default, global = true)]
    pub safety_policy: SafetyPolicy,

    /// Number of threads for parallel file processing
    #[clap(long, value_parser, global = true)]
    pub threads: Option<usize>,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum, Debug)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Run simulations and performance benchmarks to evaluate the current implementation
    #[clap(after_help = r#"
SIMULATIONS:
  Run built-in benchmark scenarios over a test corpus to evaluate performance and limitations.

EXAMPLES:
  # Run simulations in the current directory (uses bench_data if present)
  rfgrep simulate

  # Run simulations from a specific path
  rfgrep simulate --path .
"#)]
    Simulate {},
    #[clap(after_help = r#"
SEARCH MODES:
  text    - Plain text search (default)
  word    - Whole word matching with boundaries
  regex   - Regular expression search

EXAMPLES:
  # Basic text search
  rfgrep search "error" --extensions rs

  # Word boundary search
  rfgrep search "test" word --recursive

  # Regex search for function definitions
  rfgrep search "fn\s+\w+\s*\(" regex --extensions rs

  # Search and copy results to clipboard
  rfgrep search "TODO" --copy --extensions rs,md

  # Pipe input from another command
  cat file.log | rfgrep search "error"

  # Chain with other tools
  cat .zsh_history | rfgrep search "git" -c

  # Use with command substitution
  echo "test data" | rfgrep search "test"

PERFORMANCE TIPS:
  • Use --skip-binary for faster processing
  • Limit file size with --max-size
  • Use --dry-run to preview files first
  • Pipe data directly for faster processing
"#)]
    Search {
        pattern: String,

        #[clap(long, value_enum, default_value_t = SearchMode::Text)]
        mode: SearchMode,

        #[clap(long, value_parser, default_value_t = false)]
        copy: bool,

        #[clap(long, value_enum, default_value_t = OutputFormat::Text)]
        output_format: OutputFormat,

        /// Emit newline-delimited JSON (one JSON object per match)
        #[clap(long, value_parser, default_value_t = false)]
        ndjson: bool,

        #[clap(long, value_parser, use_value_delimiter = true)]
        extensions: Option<Vec<String>>,

        /// File type handling strategy
        #[clap(long, value_enum, default_value_t = FileTypeStrategy::Default)]
        file_types: FileTypeStrategy,

        /// Include specific file types (overrides default strategy)
        #[clap(long, value_parser, use_value_delimiter = true)]
        include_extensions: Option<Vec<String>>,

        /// Exclude specific file types (overrides default strategy)
        #[clap(long, value_parser, use_value_delimiter = true)]
        exclude_extensions: Option<Vec<String>>,

        /// Search all file types (comprehensive mode)
        #[clap(long, value_parser, default_value_t = false)]
        search_all_files: bool,

        /// Only search text files (conservative mode)
        #[clap(long, value_parser, default_value_t = false)]
        text_only: bool,

        #[clap(short, long, value_parser, default_value_t = false)]
        recursive: bool,

        #[clap(long, value_parser, default_value_t = 0)]
        context_lines: usize,

        #[clap(long, value_parser, default_value_t = false)]
        case_sensitive: bool,

        #[clap(long, value_parser, default_value_t = false)]
        invert_match: bool,

        /// Per-file timeout in seconds (abort scanning a file after this many seconds)
        #[clap(long, value_parser)]
        timeout_per_file: Option<u64>,

        #[clap(long, value_parser)]
        max_matches: Option<usize>,

        #[clap(long, value_enum, default_value_t = SearchAlgorithm::BoyerMoore)]
        algorithm: SearchAlgorithm,

        /// Only show count of matches, not the matches themselves
        #[clap(long, short = 'c', value_parser, default_value_t = false)]
        count: bool,

        /// Only show filenames with matches, not the matches themselves
        #[clap(long, short = 'l', value_parser, default_value_t = false)]
        files_with_matches: bool,

        #[clap(value_parser, last = true)]
        path: Option<PathBuf>,

        /// Alternative explicit path flag (useful for scripts)
        #[clap(long, value_parser, alias = "path-flag")]
        path_flag: Option<PathBuf>,
    },

    #[clap(after_help = r#"
INTERACTIVE FEATURES:
  • Real-time search with live filtering
  • Keyboard navigation and commands
  • Result highlighting and selection
  • Save results to file
  • Memory-optimized processing

COMMANDS:
  n/new   - Start a new search
  f/filter - Filter current results
  c/clear - Clear all filters
  s/save  - Save results to file
  q/quit  - Exit interactive mode

EXAMPLES:
  # Start interactive search
  rfgrep interactive "error" --extensions rs

  # Interactive search with specific algorithm
  rfgrep interactive "test" --algorithm boyer-moore --recursive
"#)]
    Interactive {
        pattern: String,

        #[clap(long, value_enum, default_value_t = InteractiveAlgorithm::BoyerMoore)]
        algorithm: InteractiveAlgorithm,

        #[clap(long, value_parser, use_value_delimiter = true)]
        extensions: Option<Vec<String>>,

        #[clap(short, long, value_parser, default_value_t = false)]
        recursive: bool,

        #[clap(value_parser, last = true)]
        path: Option<PathBuf>,

        /// Alternative explicit path flag (useful for scripts)
        #[clap(long, value_parser, alias = "path-flag")]
        path_flag: Option<PathBuf>,
    },
    #[clap(after_help = r#"
OUTPUT FORMATS:
  Simple  - Just file paths (default)
  Long    - Detailed table with size, type, and binary info

EXAMPLES:
  # Simple file listing
  rfgrep list --extensions rs

  # Detailed listing with size info
  rfgrep list --long --extensions rs,md

  # Recursive listing with hidden files
  rfgrep list --recursive --show-hidden --extensions rs

  # List files under 1MB
  rfgrep list --max-size 1 --extensions rs

FEATURES:
  • Extension statistics and file counts
  • Binary file detection
  • Size filtering and formatting
  • Hidden file handling
  • Recursive directory traversal
"#)]
    List {
        #[clap(long, value_parser, use_value_delimiter = true)]
        extensions: Option<Vec<String>>,

        #[clap(short, long, value_parser, default_value_t = false)]
        long: bool,

        #[clap(short, long, value_parser, default_value_t = false)]
        recursive: bool,

        #[clap(long, value_parser, default_value_t = false)]
        show_hidden: bool,

        #[clap(long, value_parser)]
        max_size: Option<usize>,

        #[clap(long, value_parser)]
        min_size: Option<usize>,

        #[clap(long, value_parser, default_value_t = false)]
        detailed: bool,

        #[clap(long, value_parser, default_value_t = false)]
        simple: bool,

        #[clap(long, value_parser, default_value_t = false)]
        stats: bool,

        #[clap(long, value_enum, default_value_t = SortCriteria::Name)]
        sort: SortCriteria,

        #[clap(long, value_parser, default_value_t = false)]
        reverse: bool,

        #[clap(long, value_parser)]
        limit: Option<usize>,

        #[clap(long, value_parser, default_value_t = false)]
        copy: bool,

        #[clap(long, value_enum, default_value_t = OutputFormat::Text)]
        output_format: OutputFormat,

        // Optional trailing path allowing `rfgrep list <options> <path>`
        #[clap(value_parser, last = true)]
        path: Option<PathBuf>,

        /// Alternative explicit path flag (useful for scripts)
        #[clap(long, value_parser, alias = "path-flag")]
        path_flag: Option<PathBuf>,
    },
    #[clap(after_help = r#"
SUPPORTED SHELLS:
  bash     - Bash shell completions
  zsh      - Zsh shell completions
  fish     - Fish shell completions
  powershell - PowerShell completions
  elvish   - Elvish shell completions

EXAMPLES:
  # Generate bash completions
  rfgrep completions bash > ~/.local/share/bash-completion/completions/rfgrep

  # Generate zsh completions
  rfgrep completions zsh > ~/.zsh/completions/_rfgrep

  # Generate fish completions
  rfgrep completions fish > ~/.config/fish/completions/rfgrep.fish

SETUP:
  Add the generated completion script to your shell's completion directory
  and restart your shell or source the completion file.
"#)]
    Completions {
        #[clap(value_enum)]
        shell: Shell,
    },
    #[clap(after_help = r#"
PLUGIN MANAGEMENT:
  list     - List all available plugins
  stats    - Show plugin statistics
  info     - Show detailed plugin information
  enable   - Enable a plugin
  disable  - Disable a plugin
  priority - Set plugin priority
  config   - Show plugin configuration options
  test     - Test plugin with specific file

EXAMPLES:
  # List all plugins
  rfgrep plugins list

  # Show plugin statistics
  rfgrep plugins stats

  # Get info about text plugin
  rfgrep plugins info enhanced_text

  # Enable binary plugin
  rfgrep plugins enable enhanced_binary

  # Test text plugin on a file
  rfgrep plugins test enhanced_text README.md "example"
"#)]
    Plugins {
        #[clap(subcommand)]
        command: PluginCommands,
    },
    /// Interactive TUI mode
    #[clap(after_help = r#"
INTERACTIVE TUI MODE:
  Launch an interactive terminal user interface for searching files.

EXAMPLES:
  # Start TUI with a pattern
  rfgrep tui "search pattern"

  # Start TUI and enter pattern interactively
  rfgrep tui

  # Start TUI with specific algorithm
  rfgrep tui "pattern" --algorithm boyer-moore

  # Start TUI with case-sensitive search
  rfgrep tui "pattern" --case-sensitive

CONTROLS:
  h         - Toggle help
  q         - Quit
  ↑/↓, j/k  - Navigate matches
  ←/→, h/l  - Navigate files
  n/N       - Next/Previous match
  c         - Toggle case sensitivity
  m         - Cycle search mode
  a         - Cycle algorithm
  r         - Refresh search
  Enter     - Open file in editor
"#)]
    Tui {
        /// Search pattern
        pattern: Option<String>,
        /// Search algorithm to use
        #[clap(long, value_enum, default_value = "boyer-moore")]
        algorithm: SearchAlgorithm,
        /// Enable case-sensitive search
        #[clap(long)]
        case_sensitive: bool,
        /// Search mode
        #[clap(long, value_enum, default_value = "text")]
        mode: SearchMode,
        /// Number of context lines to show
        #[clap(long, default_value = "0")]
        context_lines: usize,
        /// Search path
        #[clap(long, default_value = ".")]
        path: String,
    },
    #[clap(hide = true)]
    Worker {
        path: std::path::PathBuf,
        pattern: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum PluginCommands {
    /// List all available plugins
    List,
    /// Show plugin statistics
    Stats,
    /// Show detailed plugin information
    Info {
        /// Plugin name
        name: String,
    },
    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },
    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
    /// Set plugin priority
    Priority {
        /// Plugin name
        name: String,
        /// Priority value (lower = higher priority)
        priority: u32,
    },
    /// Show plugin configuration options
    Config {
        /// Plugin name
        name: String,
    },
    /// Test plugin with specific file
    Test {
        /// Plugin name
        name: String,
        /// File path to test
        file: String,
        /// Search pattern
        pattern: String,
    },
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchMode {
    #[default]
    Text,
    Word,
    Regex,
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileTypeStrategy {
    #[default]
    /// Default behavior - smart classification (recommended)
    Default,
    /// Search everything possible (comprehensive)
    Comprehensive,
    /// Only search safe text files (conservative)
    Conservative,
    /// Performance-first - skip potentially problematic files
    Performance,
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafetyPolicy {
    #[default]
    /// Default safety policy - balanced approach
    Default,
    /// Conservative safety - strict file type checking and size limits
    Conservative,
    /// Performance mode - relaxed safety for speed
    Performance,
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractiveAlgorithm {
    #[default]
    BoyerMoore,
    Regex,
    Simple,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SearchAlgorithm {
    BoyerMoore,
    Regex,
    Simple,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SortCriteria {
    Name,
    Size,
    Date,
    Type,
    Path,
}

#[derive(ValueEnum, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Xml,
    Html,
    Markdown,
    Csv,
    Tsv,
}

impl fmt::Display for SearchMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchMode::Text => write!(f, "text"),
            SearchMode::Word => write!(f, "word"),
            SearchMode::Regex => write!(f, "regex"),
        }
    }
}
