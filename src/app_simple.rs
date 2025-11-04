//! Simplified application structure
use crate::app::{stdin::StdinSearchOptions, FileFilter, FileFilterOptions, StdinSearcher};
use crate::cli::{
    Cli, Commands, PluginCommands, SearchAlgorithm as CliSearchAlgorithm, SearchMode,
};
use crate::error::{Result as RfgrepResult, RfgrepError};
use crate::output_formats::OutputFormatter;
use crate::plugin_cli::PluginCli;
use crate::plugin_system::{EnhancedPluginManager, PluginRegistry};
use crate::processor::search_file;
use crate::search_algorithms::SearchAlgorithm;
use crate::streaming_search::{StreamingConfig, StreamingSearchPipeline};
use crate::tui::{init_terminal, restore_terminal, TuiApp};
use crate::walker::walk_dir;
use colored::Colorize;
use std::path::Path;
use std::sync::Arc;

/// Simplified application that uses existing components
pub struct RfgrepApp {
    plugin_manager: Arc<EnhancedPluginManager>,
}

impl RfgrepApp {
    /// Create a new application instance
    pub fn new() -> RfgrepResult<Self> {
        let plugin_manager = Arc::new(EnhancedPluginManager::new());
        let registry = PluginRegistry::new(plugin_manager.clone());

        // Use the existing tokio runtime if available, otherwise create a new one
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                // We're already in an async context, use the current runtime
                handle.block_on(async { registry.load_plugins().await })?;
                return Ok(Self { plugin_manager });
            }
            Err(_) => {
                // No current runtime, create a new one
                tokio::runtime::Runtime::new().map_err(|e| {
                    crate::error::RfgrepError::Other(format!("Failed to create runtime: {}", e))
                })?
            }
        };

        rt.block_on(async { registry.load_plugins().await })?;
        Ok(Self { plugin_manager })
    }

    /// Create a new application instance with async support
    pub async fn new_async() -> RfgrepResult<Self> {
        let plugin_manager = Arc::new(EnhancedPluginManager::new());
        let registry = PluginRegistry::new(plugin_manager.clone());

        // Load plugins asynchronously
        registry.load_plugins().await?;

        Ok(Self { plugin_manager })
    }

    /// Run the application with the given CLI arguments
    pub async fn run(&self, cli: Cli) -> RfgrepResult<()> {
        if let Some(log_path) = &cli.log {
            std::fs::write(log_path, "rfgrep log file created\n").map_err(RfgrepError::Io)?;
        }

        let is_piped = !is_terminal::is_terminal(&std::io::stdout());
        let quiet = cli.quiet || is_piped;

        match &cli.command {
            Commands::Search {
                pattern,
                mode,
                algorithm,
                recursive,
                context_lines,
                case_sensitive,
                invert_match,
                max_matches,
                timeout_per_file,
                path: cmd_path,
                path_flag: cmd_path_flag,
                output_format,
                file_types,
                include_extensions,
                exclude_extensions,
                search_all_files,
                text_only,
                ndjson,
                count,
                files_with_matches,
                ..
            } => {
                self.handle_search(
                    pattern,
                    mode.clone(),
                    algorithm.clone(),
                    *recursive,
                    *context_lines,
                    *case_sensitive,
                    *invert_match,
                    *max_matches,
                    *timeout_per_file,
                    cmd_path
                        .as_ref()
                        .or(cmd_path_flag.as_ref())
                        .map(|p| p.as_path())
                        .unwrap_or(&cli.path),
                    cli.max_size,
                    cli.skip_binary,
                    output_format.clone(),
                    file_types.clone(),
                    include_extensions.clone(),
                    exclude_extensions.clone(),
                    *search_all_files,
                    *text_only,
                    cli.safety_policy.clone(),
                    cli.threads,
                    *ndjson,
                    *count,
                    *files_with_matches,
                    quiet,
                )
                .await
            }
            Commands::List {
                extensions,
                long,
                recursive,
                show_hidden,
                max_size,
                min_size,
                detailed,
                simple,
                stats,
                sort,
                reverse,
                limit,
                copy,
                output_format,
                path: cmd_path,
                path_flag: cmd_path_flag,
            } => {
                self.handle_list(
                    extensions.as_deref(),
                    *long,
                    *recursive,
                    *show_hidden,
                    *max_size,
                    *min_size,
                    *detailed,
                    *simple,
                    *stats,
                    sort.clone(),
                    *reverse,
                    *limit,
                    *copy,
                    output_format.clone(),
                    cmd_path.as_ref().map(|p| p.as_path()),
                    cmd_path_flag.as_ref().map(|p| p.as_path()),
                    &cli.path,
                )
                .await
            }
            Commands::Interactive { .. } => {
                println!("Interactive command not yet implemented in simplified version");
                Ok(())
            }
            Commands::Completions { shell } => self.handle_completions(*shell),
            Commands::Simulate {} => {
                use std::fs;
                use std::time::Instant;
                let results_dir = cli.path.join("results");
                if let Err(e) = fs::create_dir_all(&results_dir) {
                    return Err(RfgrepError::Io(e));
                }

                let search_root = cli.path.join("bench_data");
                let search_root = if search_root.exists() {
                    search_root
                } else {
                    cli.path.clone()
                };

                let entries: Vec<_> = crate::walker::walk_dir(&search_root, true, true).collect();
                let files: Vec<_> = entries
                    .into_iter()
                    .filter(|e| e.path().is_file())
                    .map(|e| e.path().to_path_buf())
                    .collect();

                if files.is_empty() {
                    println!(
                        "Warning: No files found in search directory: {}",
                        search_root.display()
                    );
                    println!("Creating a small test file for simulation...");

                    let test_file = search_root.join("test_simulation.txt");
                    let test_content = "This is a test file for simulation.\nIt contains some error messages.\nTODO: Add more test cases.\nThe quick brown fox jumps over the lazy dog.\n";
                    fs::write(&test_file, test_content).map_err(RfgrepError::Io)?;

                    let entries: Vec<_> =
                        crate::walker::walk_dir(&search_root, true, true).collect();
                    let files: Vec<_> = entries
                        .into_iter()
                        .filter(|e| e.path().is_file())
                        .map(|e| e.path().to_path_buf())
                        .collect();

                    if files.is_empty() {
                        return Err(RfgrepError::Other(
                            "No files available for simulation".to_string(),
                        ));
                    }
                }

                println!(
                    "Running simulations on {} files in {}",
                    files.len(),
                    search_root.display()
                );

                let scenarios = vec![
                    ("regex_short", r"error".to_string()),
                    ("word_boundary", r"\bTODO\b".to_string()),
                    ("literal_long", "the quick brown fox jumps over".to_string()),
                ];

                let mut report = String::from("Scenario,Millis,Matches,Files\n");
                for (name, pat) in scenarios {
                    let start = Instant::now();
                    let mut total = 0usize;
                    let mut files_processed = 0usize;

                    let regex = crate::processor::get_or_compile_regex(&pat)?;
                    for f in &files {
                        if let Ok(matches) = crate::processor::search_file(f, &regex) {
                            total += matches.len();
                            files_processed += 1;
                        }
                    }
                    let elapsed = start.elapsed().as_millis();
                    report.push_str(&format!(
                        "{},{},{},{}\n",
                        name, elapsed, total, files_processed
                    ));
                }

                let report_path = results_dir.join("simulations.csv");
                fs::write(&report_path, &report).map_err(RfgrepError::Io)?;
                println!("Simulations complete. Report: {}", report_path.display());
                println!("\n{}", report);
                Ok(())
            }
            Commands::Worker { path, pattern } => self.handle_worker(path, pattern).await,
            Commands::Plugins { command } => self.handle_plugin_command(command).await,
            Commands::Tui {
                pattern,
                algorithm,
                case_sensitive,
                mode,
                context_lines,
                path,
            } => {
                self.handle_tui_command(
                    pattern.as_deref(),
                    algorithm,
                    *case_sensitive,
                    mode,
                    *context_lines,
                    path,
                )
                .await
            }
        }
    }

    async fn handle_search(
        &self,
        pattern: &str,
        mode: crate::cli::SearchMode,
        algorithm: CliSearchAlgorithm,
        recursive: bool,
        context_lines: usize,
        case_sensitive: bool,
        invert_match: bool,
        max_matches: Option<usize>,
        timeout_per_file: Option<u64>,
        search_path: &Path,
        max_size: Option<usize>,
        _skip_binary: bool,
        output_format: crate::cli::OutputFormat,
        file_types: crate::cli::FileTypeStrategy,
        include_extensions: Option<Vec<String>>,
        exclude_extensions: Option<Vec<String>>,
        search_all_files: bool,
        text_only: bool,
        safety_policy: crate::cli::SafetyPolicy,
        threads: Option<usize>,
        ndjson: bool,
        count: bool,
        files_with_matches: bool,
        quiet: bool,
    ) -> RfgrepResult<()> {
        let search_pattern = self.build_search_pattern(pattern, mode);
        let search_algorithm = self.map_search_algorithm(algorithm);

        // Check if stdin has data (piped input)
        // Only search stdin if it's not a terminal AND the search path is explicitly NOT provided
        // This prevents false positives in test environments where stdin might be redirected but empty
        let stdin_is_piped = !is_terminal::is_terminal(&std::io::stdin());
        let search_path_str = search_path.to_string_lossy();
    let is_default_path = search_path_str == "." || search_path_str.is_empty();

        // Only use stdin if it's piped AND we're searching the default path
        // If a specific path is given, prefer file search even if stdin is piped
        if stdin_is_piped && is_default_path {
            // Handle piped input from stdin using dedicated stdin module
            let stdin_searcher = StdinSearcher::new();
            let options = StdinSearchOptions {
                search_pattern,
                original_pattern: pattern.to_string(),
                case_sensitive,
                invert_match,
                max_matches,
                output_format,
                ndjson,
                count,
                files_with_matches,
                quiet,
            };
            return stdin_searcher.search(options).await;
        }

        let files = self.collect_files(search_path, recursive);

        // Use the FileFilter module for filtering
        let filter_options = FileFilterOptions {
            max_size,
            skip_binary: _skip_binary,
            safety_policy,
            include_extensions,
            exclude_extensions,
            search_all_files,
            text_only,
            file_types,
        };
        let file_filter = FileFilter::new(filter_options);
        let filtered_files = file_filter.filter_files(files);

        if !quiet && output_format != crate::cli::OutputFormat::Json && !ndjson {
            println!("Searching {} files...", filtered_files.len());
        }

        let all_matches = self
            .perform_search(
                &filtered_files,
                &search_pattern,
                search_algorithm,
                context_lines,
                case_sensitive,
                invert_match,
                max_matches,
                timeout_per_file,
                threads,
            )
            .await?;

        self.output_results(
            &all_matches,
            pattern,
            search_path,
            output_format,
            ndjson,
            count,
            files_with_matches,
            quiet,
        )
    }

    /// Build search pattern based on mode
    fn build_search_pattern(&self, pattern: &str, mode: crate::cli::SearchMode) -> String {
        match mode {
            crate::cli::SearchMode::Text => pattern.to_string(),
            crate::cli::SearchMode::Word => format!(r"\b{}\b", regex::escape(pattern)),
            crate::cli::SearchMode::Regex => pattern.to_string(),
        }
    }

    /// Map CLI search algorithm to internal algorithm
    fn map_search_algorithm(&self, algorithm: CliSearchAlgorithm) -> SearchAlgorithm {
        match algorithm {
            CliSearchAlgorithm::BoyerMoore => SearchAlgorithm::BoyerMoore,
            CliSearchAlgorithm::Regex => SearchAlgorithm::Regex,
            CliSearchAlgorithm::Simple => SearchAlgorithm::Simple,
        }
    }

    /// Collect files from directory
    fn collect_files(&self, search_path: &Path, recursive: bool) -> Vec<std::path::PathBuf> {
        let entries: Vec<_> = walk_dir(search_path, recursive, true).collect();
        entries
            .into_iter()
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path().to_path_buf())
            .collect()
    }

    /// Perform the actual search
    async fn perform_search(
        &self,
        filtered_files: &[std::path::PathBuf],
        search_pattern: &str,
        search_algorithm: SearchAlgorithm,
        context_lines: usize,
        case_sensitive: bool,
        invert_match: bool,
        max_matches: Option<usize>,
        timeout_per_file: Option<u64>,
        threads: Option<usize>,
    ) -> RfgrepResult<Vec<crate::processor::SearchMatch>> {
        let config = StreamingConfig {
            algorithm: search_algorithm,
            context_lines,
            case_sensitive,
            invert_match,
            max_matches,
            timeout_per_file,
            chunk_size: 8192,
            buffer_size: 65536,
        };

        let thread_count = threads.unwrap_or_else(|| num_cpus::get().min(8));

        let pipeline = StreamingSearchPipeline::new(config);
        let file_refs: Vec<&Path> = filtered_files.iter().map(|p| p.as_path()).collect();

        if file_refs.len() > 10 {
            pipeline
                .search_files_parallel(&file_refs, search_pattern, thread_count)
                .await
        } else {
            let mut all_matches = Vec::new();
            for file in filtered_files {
                match pipeline.search_file(file, search_pattern).await {
                    Ok(matches) => all_matches.extend(matches),
                    Err(e) => {
                        eprintln!("Error searching {}: {}", file.display(), e);
                    }
                }
            }
            Ok(all_matches)
        }
    }

    /// Output the search results
    fn output_results(
        &self,
        all_matches: &[crate::processor::SearchMatch],
        pattern: &str,
        search_path: &Path,
        output_format: crate::cli::OutputFormat,
        ndjson: bool,
        count: bool,
        files_with_matches: bool,
        quiet: bool,
    ) -> RfgrepResult<()> {
        if all_matches.is_empty() {
            self.output_no_matches(count, files_with_matches, output_format)
        } else if count {
            println!("{}", all_matches.len());
        } else if files_with_matches {
            self.output_files_with_matches(all_matches)
        } else {
            self.output_matches(
                all_matches,
                pattern,
                search_path,
                output_format,
                ndjson,
                quiet,
            )
        }

        Ok(())
    }

    /// Handle case when no matches are found
    fn output_no_matches(
        &self,
        count: bool,
        files_with_matches: bool,
        output_format: crate::cli::OutputFormat,
    ) {
        if count {
            println!("0");
        } else if files_with_matches {
            // Print nothing
        } else if output_format != crate::cli::OutputFormat::Json {
            println!("{}", "No matches found".yellow());
        }
    }

    /// Output list of files containing matches
    fn output_files_with_matches(&self, all_matches: &[crate::processor::SearchMatch]) {
        use std::collections::HashSet;
        let mut unique_files: HashSet<String> = HashSet::new();
        for m in all_matches {
            unique_files.insert(m.path.to_string_lossy().to_string());
        }
        let mut files: Vec<_> = unique_files.into_iter().collect();
        files.sort();
        for file in files {
            println!("{}", file);
        }
    }

    /// Output the actual matches
    fn output_matches(
        &self,
        all_matches: &[crate::processor::SearchMatch],
        pattern: &str,
        search_path: &Path,
        output_format: crate::cli::OutputFormat,
        ndjson: bool,
        quiet: bool,
    ) {
        if !quiet && output_format != crate::cli::OutputFormat::Json && !ndjson {
            println!(
                "\n{} {} {}",
                "Found".green(),
                all_matches.len(),
                "matches:".green()
            );
        }

        let formatter = OutputFormatter::new(if ndjson {
            crate::output_formats::OutputFormat::Json
        } else {
            match output_format {
                crate::cli::OutputFormat::Text => crate::output_formats::OutputFormat::Text,
                crate::cli::OutputFormat::Json => crate::output_formats::OutputFormat::Json,
                crate::cli::OutputFormat::Xml => crate::output_formats::OutputFormat::Xml,
                crate::cli::OutputFormat::Html => crate::output_formats::OutputFormat::Html,
                crate::cli::OutputFormat::Markdown => crate::output_formats::OutputFormat::Markdown,
                crate::cli::OutputFormat::Csv => crate::output_formats::OutputFormat::Csv,
                crate::cli::OutputFormat::Tsv => crate::output_formats::OutputFormat::Tsv,
            }
        })
        .with_ndjson(ndjson);

        let output = formatter.format_results(all_matches, pattern, search_path);

        if output_format == crate::cli::OutputFormat::Json || ndjson {
            print!("{output}");
        } else {
            println!("\n{output}");
        }
    }

    fn handle_completions(&self, shell: clap_complete::Shell) -> RfgrepResult<()> {
        use clap::CommandFactory;
        let mut cmd = Cli::command();
        clap_complete::generate(shell, &mut cmd, "rfgrep", &mut std::io::stdout());
        Ok(())
    }

    async fn handle_worker(&self, path: &std::path::Path, pattern: &str) -> RfgrepResult<()> {
        if let Ok(s) = std::env::var("RFGREP_WORKER_SLEEP") {
            if let Ok(sec) = s.parse::<u64>() {
                std::thread::sleep(std::time::Duration::from_secs(sec));
            }
        }

        let regex = crate::processor::get_or_compile_regex(pattern)?;
        let matches = search_file(path, &regex)?;

        for m in matches {
            if let Ok(json) = serde_json::to_string(&m) {
                println!("{json}");
            }
        }

        Ok(())
    }

    async fn handle_plugin_command(&self, command: &PluginCommands) -> RfgrepResult<()> {
        let plugin_cli = PluginCli::new(self.plugin_manager.clone());

        match command {
            PluginCommands::List => plugin_cli.list_plugins().await,
            PluginCommands::Stats => plugin_cli.show_stats().await,
            PluginCommands::Info { name } => plugin_cli.show_plugin_info(name).await,
            PluginCommands::Enable { name } => plugin_cli.enable_plugin(name).await,
            PluginCommands::Disable { name } => plugin_cli.disable_plugin(name).await,
            PluginCommands::Priority { name, priority } => {
                plugin_cli.set_priority(name, *priority).await
            }
            PluginCommands::Config { name } => plugin_cli.show_config_options(name).await,
            PluginCommands::Test {
                name,
                file,
                pattern,
            } => plugin_cli.test_plugin(name, file, pattern).await,
        }
    }

    async fn handle_tui_command(
        &self,
        pattern: Option<&str>,
        algorithm: &CliSearchAlgorithm,
        case_sensitive: bool,
        mode: &SearchMode,
        context_lines: usize,
        _path: &str,
    ) -> RfgrepResult<()> {
        let mut terminal = init_terminal()?;
        let mut app = TuiApp::new().await?;

        if let Some(p) = pattern {
            app.set_pattern(p.to_string());
        }

        let tui_algorithm = match algorithm {
            CliSearchAlgorithm::BoyerMoore => SearchAlgorithm::BoyerMoore,
            CliSearchAlgorithm::Regex => SearchAlgorithm::Regex,
            CliSearchAlgorithm::Simple => SearchAlgorithm::Simple,
        };

        let tui_mode = match mode {
            SearchMode::Text => crate::tui::SearchMode::Text,
            SearchMode::Word => crate::tui::SearchMode::Word,
            SearchMode::Regex => crate::tui::SearchMode::Regex,
        };

        app.state.algorithm = tui_algorithm;
        app.state.case_sensitive = case_sensitive;
        app.state.context_lines = context_lines;
        app.state.search_mode = tui_mode;

        if let Some(p) = pattern {
            app.state.status_message = format!("Searching for: {}", p);
            let mut all_matches = Vec::new();
            let search_root = std::path::PathBuf::from(_path);
            let search_root = if search_root.as_os_str().is_empty() {
                std::path::PathBuf::from(".")
            } else {
                search_root
            };
            let entries: Vec<_> = walk_dir(&search_root, true, false).collect();
            for entry in entries {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(mut matches) = self.plugin_manager.search_file(path, p).await {
                        all_matches.append(&mut matches);
                    }
                }
            }
            app.set_matches(all_matches);
        }

        let result = app.run(&mut terminal).await;

        restore_terminal(&mut terminal)?;

        result
    }

    async fn handle_list(
        &self,
        extensions: Option<&[String]>,
        long: bool,
        recursive: bool,
        show_hidden: bool,
        max_size: Option<usize>,
        min_size: Option<usize>,
        _detailed: bool,
        simple: bool,
        stats: bool,
        sort: crate::cli::SortCriteria,
        reverse: bool,
        limit: Option<usize>,
        _copy: bool,
        _output_format: crate::cli::OutputFormat,
        cmd_path: Option<&Path>,
        cmd_path_flag: Option<&Path>,
        default_path: &Path,
    ) -> RfgrepResult<()> {
        let search_path = cmd_path_flag.or(cmd_path).unwrap_or(default_path);

        let entries: Vec<_> = walk_dir(search_path, recursive, show_hidden).collect();
        let mut files: Vec<_> = entries
            .into_iter()
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.path().to_path_buf())
            .collect();

        files.retain(|path| {
            if let Some(exts) = extensions {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if !exts.iter().any(|e| e.eq_ignore_ascii_case(ext_str)) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if let Ok(metadata) = path.metadata() {
                let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
                if let Some(max) = max_size {
                    if size_mb > max as f64 {
                        return false;
                    }
                }
                if let Some(min) = min_size {
                    if size_mb < min as f64 {
                        return false;
                    }
                }
            }

            true
        });

        match sort {
            crate::cli::SortCriteria::Name => {
                files.sort_by(|a, b| a.file_name().cmp(&b.file_name()))
            }
            crate::cli::SortCriteria::Size => {
                files.sort_by(|a, b| {
                    let size_a = a.metadata().map(|m| m.len()).unwrap_or(0);
                    let size_b = b.metadata().map(|m| m.len()).unwrap_or(0);
                    size_a.cmp(&size_b)
                });
            }
            crate::cli::SortCriteria::Date => {
                files.sort_by(|a, b| {
                    let time_a = a
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    let time_b = b
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::UNIX_EPOCH);
                    time_a.cmp(&time_b)
                });
            }
            crate::cli::SortCriteria::Type => {
                files.sort_by(|a, b| {
                    let ext_a = a.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let ext_b = b.extension().and_then(|e| e.to_str()).unwrap_or("");
                    ext_a.cmp(ext_b)
                });
            }
            crate::cli::SortCriteria::Path => {
                files.sort();
            }
        }

        if reverse {
            files.reverse();
        }

        if let Some(limit) = limit {
            files.truncate(limit);
        }

        if stats {
            println!("Summary: {} files found", files.len());
        } else if simple {
            for file in &files {
                println!("{}", file.display());
            }
        } else {
            for file in &files {
                if long {
                    if let Ok(metadata) = file.metadata() {
                        let size = metadata.len();
                        let modified = metadata.modified().unwrap_or(std::time::UNIX_EPOCH);
                        println!(
                            "{} {} {}",
                            size,
                            modified
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            file.display()
                        );
                    } else {
                        println!("{}", file.display());
                    }
                } else {
                    println!("{}", file.display());
                }
            }
            println!("Summary: {} files found", files.len());

            if long {
                let mut extensions: std::collections::HashMap<String, usize> =
                    std::collections::HashMap::new();
                for file in &files {
                    if let Some(ext) = file.extension() {
                        if let Some(ext_str) = ext.to_str() {
                            *extensions.entry(ext_str.to_string()).or_insert(0) += 1;
                        }
                    }
                }
                if !extensions.is_empty() {
                    println!("Extensions:");
                    let mut ext_vec: Vec<_> = extensions.iter().collect();
                    ext_vec.sort_by(|a, b| a.0.cmp(b.0));
                    for (ext, count) in ext_vec {
                        println!("  .{}: {} files", ext, count);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for RfgrepApp {
    fn default() -> Self {
        Self::new().expect("Failed to create RfgrepApp")
    }
}
