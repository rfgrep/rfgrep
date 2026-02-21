use crate::cli::SearchMode;
use crate::error::{Result as RfgrepResult, RfgrepError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration precedence (highest to lowest):
/// 1. Command-line arguments
/// 2. Environment variables (RFGREP_*)
/// 3. Project-level config (.rfgreprc in project root)
/// 4. User-level config (~/.config/rfgrep/config.toml)
/// 5. System-level config (/etc/rfgrep/config.toml)
/// 6. Built-in defaults

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub search: SearchConfig,
    pub output: OutputConfig,
    pub filters: FilterConfig,
    pub performance: PerformanceConfig,
    pub git: GitConfig,
    pub compression: CompressionConfig,
    pub logging: LoggingConfig,
    pub type_definitions: TypeDefinitions,
    pub shortcuts: HashMap<String, SearchShortcut>,
    pub ui: UIConfig,
    pub experimental: ExperimentalConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub mode: SearchMode,
    pub case_sensitive: bool,
    pub smart_case: bool,
    pub max_file_size_mb: u64,
    pub skip_binary: bool,
    pub context_before: usize,
    pub context_after: usize,
    pub threads: usize,
    pub chunk_size: usize,
    pub algorithms: AlgorithmConfig,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            mode: SearchMode::Regex,
            case_sensitive: false,
            smart_case: true,
            max_file_size_mb: 100, // 10MB
            skip_binary: true,
            context_before: 2,
            context_after: 2,
            threads: 0, // 0 = auto
            chunk_size: 100,
            algorithms: AlgorithmConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AlgorithmConfig {
    pub simple: String,
    pub regex: String,
    pub multi_pattern: String,
}

impl Default for AlgorithmConfig {
    fn default() -> Self {
        Self {
            simple: "boyer-moore".to_string(),
            regex: "regex-automaton".to_string(),
            multi_pattern: "aho-corasick".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OutputConfig {
    pub format: OutputFormat,
    pub color: ColorMode,
    pub line_numbers: bool,
    pub show_filenames: bool,
    pub show_columns: bool,
    pub syntax_highlighting: bool,
    pub colors: ColorScheme,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Text,
            color: ColorMode::Auto,
            line_numbers: true,
            show_filenames: true,
            show_columns: false,
            syntax_highlighting: true,
            colors: ColorScheme::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
    Ndjson,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub r#match: String,
    pub line_number: String,
    pub filename: String,
    pub separator: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            r#match: "red".to_string(),
            line_number: "green".to_string(),
            filename: "blue".to_string(),
            separator: "cyan".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterConfig {
    pub extensions: Vec<String>,
    pub exclude_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub ignore_directories: Vec<String>,
    pub ignore_files: Vec<String>,
    pub size: SizeFilter,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            extensions: Vec::new(),
            exclude_extensions: vec![
                "o".to_string(),
                "so".to_string(),
                "dylib".to_string(),
                "exe".to_string(),
                "dll".to_string(),
                "class".to_string(),
                "pyc".to_string(),
            ],
            exclude_patterns: Vec::new(),
            ignore_directories: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "target".to_string(),
                "build".to_string(),
                "dist".to_string(),
                ".next".to_string(),
                "__pycache__".to_string(),
                ".cache".to_string(),
            ],
            ignore_files: vec![
                ".gitignore".to_string(),
                ".ignore".to_string(),
                ".rfgrepignore".to_string(),
            ],
            size: SizeFilter::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeFilter {
    pub min_size: u64,
    pub max_size: u64,
}

impl Default for SizeFilter {
    fn default() -> Self {
        Self {
            min_size: 0,
            max_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    pub mmap_threshold_mb: u64,
    pub max_memory_usage_mb: u64,
    pub chunk_size_multiplier: f64,
    pub adaptive_memory: bool,
    pub simd: String,
    pub parallel: bool,
    pub buffer_size: usize,
    pub regex_cache_size: usize,
    pub optimization: OptimizationConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            mmap_threshold_mb: 16,
            max_memory_usage_mb: 512,
            chunk_size_multiplier: 1.0,
            adaptive_memory: true,
            simd: "auto".to_string(),
            parallel: true,
            buffer_size: 65536,
            regex_cache_size: 100,
            optimization: OptimizationConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OptimizationConfig {
    pub literal_extraction: bool,
    pub pre_filter: bool,
    pub auto_dfa: bool,
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            literal_extraction: true,
            pre_filter: true,
            auto_dfa: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GitConfig {
    pub respect_gitignore: bool,
    pub respect_global_gitignore: bool,
    pub respect_git_exclude: bool,
    pub search_dot_git: bool,
    pub submodules: GitSubmoduleConfig,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            respect_gitignore: true,
            respect_global_gitignore: true,
            respect_git_exclude: true,
            search_dot_git: false,
            submodules: GitSubmoduleConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GitSubmoduleConfig {
    pub follow: bool,
    pub search: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CompressionConfig {
    pub enabled: bool,
    pub formats: Vec<String>,
    pub max_decompressed_size_mb: u64,
    pub cache_decompressed: bool,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            formats: vec![
                "gzip".to_string(),
                "bzip2".to_string(),
                "xz".to_string(),
                "zstd".to_string(),
                "lz4".to_string(),
            ],
            max_decompressed_size_mb: 100,
            cache_decompressed: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    pub level: String,
    pub file: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: "".to_string(),
            format: "text".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TypeDefinitions {
    pub rust: Vec<String>,
    pub python: Vec<String>,
    pub javascript: Vec<String>,
    pub typescript: Vec<String>,
    pub web: Vec<String>,
    pub config: Vec<String>,
    pub markdown: Vec<String>,
    pub sql: Vec<String>,
    #[serde(flatten)]
    pub custom: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchShortcut {
    pub pattern: String,
    pub mode: SearchMode,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub exclude_extensions: Vec<String>,
    #[serde(default)]
    pub case_sensitive: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UIConfig {
    pub interactive: bool,
    pub show_progress: bool,
    pub pager: String,
    pub pager_command: String,
}

impl Default for UIConfig {
    fn default() -> Self {
        Self {
            interactive: true,
            show_progress: true,
            pager: "auto".to_string(),
            pager_command: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ExperimentalConfig {
    pub semantic_search: bool,
    pub ai_suggestions: bool,
}

impl Config {
    /// Merge another config into this one, with other taking precedence
    pub fn merge(&mut self, other: &Config) {
        self.search = other.search.clone();
        self.output = other.output.clone();
        self.performance = other.performance.clone();

        if !other.filters.extensions.is_empty() {
            self.filters.extensions = other.filters.extensions.clone();
        }
    }

    /// Load from TOML file
    pub fn from_toml_file(path: &Path) -> RfgrepResult<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            RfgrepError::Other(format!("Failed to read config file {:?}: {}", path, e))
        })?;

        toml::from_str(&content).map_err(|e| {
            RfgrepError::Other(format!("Failed to parse TOML config {:?}: {}", path, e))
        })
    }

    /// Auto-detect format and load
    pub fn load_from_path(path: &Path) -> RfgrepResult<Self> {
        // Only TOML supported for now
        Self::from_toml_file(path)
    }

    /// Auto-detect format and load default config
    pub fn load() -> RfgrepResult<Self> {
        let config_path =
            Self::find_config_path().map_err(|e| RfgrepError::Other(e.to_string()))?;
        if let Some(path) = config_path {
            Self::from_toml_file(&path)
        } else {
            Ok(Self::default())
        }
    }

    fn find_config_path() -> RfgrepResult<Option<PathBuf>> {
        if let Some(xdg_config) = dirs::config_dir() {
            let xdg_path = xdg_config.join("rfgrep/config.toml");
            if xdg_path.exists() {
                return Ok(Some(xdg_path));
            }
        }

        if let Some(home) = dirs::home_dir() {
            let home_path = home.join(".rfgrep.toml");
            if home_path.exists() {
                return Ok(Some(home_path));
            }
        }

        let current_path = Path::new(".rfgrep.toml");
        if current_path.exists() {
            return Ok(Some(current_path.to_path_buf()));
        }

        Ok(None)
    }

    pub fn validate(&self) -> RfgrepResult<()> {
        if self.search.threads > 1024 {
            return Err(RfgrepError::Other(format!(
                "Thread count too high: {}",
                self.search.threads
            )));
        }
        Ok(())
    }
}

pub struct ConfigManager {
    pub system_config: Option<Config>,
    pub user_config: Option<Config>,
    pub project_config: Option<Config>,
    pub env_config: Config,
    pub merged_config: Config,
}

impl ConfigManager {
    pub fn new() -> RfgrepResult<Self> {
        let mut manager = Self {
            system_config: Self::load_system_config()?,
            user_config: Self::load_user_config()?,
            project_config: None, // Loaded later if needed
            env_config: Self::load_env_config()?,
            merged_config: Config::default(),
        };

        manager.merge_configs()?;
        Ok(manager)
    }

    pub fn load_for_directory(&mut self, dir: &Path) -> RfgrepResult<()> {
        if let Some(project_root) = Self::find_project_root(dir)? {
            self.project_config = Self::load_project_config(&project_root)?;
            self.merge_configs()?;
        }
        Ok(())
    }

    fn merge_configs(&mut self) -> RfgrepResult<()> {
        let mut merged = Config::default();

        if let Some(system) = &self.system_config {
            merged.merge(system);
        }
        if let Some(user) = &self.user_config {
            merged.merge(user);
        }
        if let Some(project) = &self.project_config {
            merged.merge(project);
        }
        merged.merge(&self.env_config);

        self.merged_config = merged;
        Ok(())
    }

    fn load_system_config() -> RfgrepResult<Option<Config>> {
        let path = PathBuf::from("/etc/rfgrep/config.toml");
        if path.exists() {
            Ok(Some(Config::load_from_path(&path)?))
        } else {
            Ok(None)
        }
    }

    fn load_user_config() -> RfgrepResult<Option<Config>> {
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("rfgrep/config.toml");
            if path.exists() {
                return Ok(Some(Config::load_from_path(&path)?));
            }
        }
        Ok(None)
    }

    fn load_project_config(root: &Path) -> RfgrepResult<Option<Config>> {
        let path = root.join(".rfgreprc");
        if path.exists() {
            // Try TOML parsing for now, assuming .rfgreprc is TOML
            return Ok(Some(Config::load_from_path(&path)?));
        }
        Ok(None)
    }

    fn load_env_config() -> RfgrepResult<Config> {
        let mut config = Config::default();

        if let Ok(val) = env::var("RFGREP_THREADS") {
            if let Ok(v) = val.parse() {
                config.search.threads = v;
            }
        }
        // ... load other env vars
        Ok(config)
    }

    fn find_project_root(start_dir: &Path) -> RfgrepResult<Option<PathBuf>> {
        let mut current = start_dir.to_path_buf();
        loop {
            for marker in &[".git", ".rfgreprc", "Cargo.toml"] {
                if current.join(marker).exists() {
                    return Ok(Some(current));
                }
            }
            if !current.pop() {
                break;
            }
        }
        Ok(None)
    }
}
