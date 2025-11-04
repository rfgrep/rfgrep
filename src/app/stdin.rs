/// Stdin search handler for piped input
///
/// This module provides dedicated functionality for searching content from stdin,
/// enabling Unix pipeline integration like: `cat file.log | rfgrep search "pattern"`
use crate::cli::OutputFormat as CliOutputFormat;
use crate::error::{Result as RfgrepResult, RfgrepError};
use crate::output_formats::OutputFormatter;
use crate::processor::SearchMatch;
use colored::Colorize;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Configuration options for stdin search
pub struct StdinSearchOptions {
    pub search_pattern: String,
    pub original_pattern: String,
    pub case_sensitive: bool,
    pub invert_match: bool,
    pub max_matches: Option<usize>,
    pub output_format: CliOutputFormat,
    pub ndjson: bool,
    pub count: bool,
    pub files_with_matches: bool,
    pub quiet: bool,
}

/// Handler for searching stdin input
pub struct StdinSearcher;

impl StdinSearcher {
    /// Create a new stdin searcher
    pub fn new() -> Self {
        Self
    }

    /// Search stdin for patterns (handles piped input)
    ///
    /// # Arguments
    ///
    /// * `options` - Search configuration options
    ///
    /// # Returns
    ///
    /// Result indicating success or error
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rfgrep::app::StdinSearcher;
    /// use rfgrep::app::stdin::StdinSearchOptions;
    /// use rfgrep::cli::OutputFormat;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let options = StdinSearchOptions {
    ///     search_pattern: "error".to_string(),
    ///     original_pattern: "error".to_string(),
    ///     case_sensitive: true,
    ///     invert_match: false,
    ///     max_matches: None,
    ///     output_format: OutputFormat::Text,
    ///     ndjson: false,
    ///     count: false,
    ///     files_with_matches: false,
    ///     quiet: false,
    /// };
    ///
    /// let searcher = StdinSearcher::new();
    /// searcher.search(options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(&self, options: StdinSearchOptions) -> RfgrepResult<()> {
        let regex_pattern = if options.case_sensitive {
            options.search_pattern.clone()
        } else {
            format!("(?i){}", options.search_pattern)
        };

        let regex = crate::processor::get_or_compile_regex(&regex_pattern)?;
        let stdin = std::io::stdin();
        let reader = BufReader::new(stdin.lock());

        let mut matches = Vec::new();
        let mut match_count = 0;

            for (line_number, line_result) in reader.lines().enumerate() {
                let line = line_result.map_err(RfgrepError::Io)?;

            let is_match = regex.is_match(&line);
            let should_include = if options.invert_match {
                !is_match
            } else {
                is_match
            };

            if should_include {
                match_count += 1;

                if !options.count && !options.files_with_matches {
                    let (matched_text, column_start, column_end) =
                        if let Some(mat) = regex.find(&line) {
                            (mat.as_str().to_string(), mat.start(), mat.end())
                        } else {
                            (String::default(), 0, line.len())
                        };

                    let search_match = SearchMatch {
                        path: PathBuf::from("<stdin>"),
                        line_number,
                        line: line.clone(),
                        context_before: Vec::new(),
                        context_after: Vec::new(),
                        matched_text,
                        column_start,
                        column_end,
                    };
                    matches.push(search_match);
                }

                if let Some(max) = options.max_matches {
                    if match_count >= max {
                        break;
                    }
                }
            }
        }

        self.output_results(matches, match_count, &options)
    }

    /// Output search results in the appropriate format
    fn output_results(
        &self,
        matches: Vec<SearchMatch>,
        match_count: usize,
        options: &StdinSearchOptions,
    ) -> RfgrepResult<()> {
        if options.count {
            println!("{}", match_count);
        } else if options.files_with_matches {
            if match_count > 0 {
                println!("<stdin>");
            }
        } else if matches.is_empty() {
            self.output_no_matches(options);
        } else {
            self.output_matches(&matches, options)?;
        }

        Ok(())
    }

    /// Handle output when no matches are found
    fn output_no_matches(&self, options: &StdinSearchOptions) {
        if options.output_format != CliOutputFormat::Json && !options.quiet {
            println!("{}", "No matches found".yellow());
        }
    }

    /// Output the actual matches with formatting
    fn output_matches(
        &self,
        matches: &[SearchMatch],
        options: &StdinSearchOptions,
    ) -> RfgrepResult<()> {
        if !options.quiet && options.output_format != CliOutputFormat::Json && !options.ndjson {
            println!(
                "\n{} {} {}",
                "Found".green(),
                matches.len(),
                "matches:".green()
            );
        }

        let formatter = OutputFormatter::new(if options.ndjson {
            crate::output_formats::OutputFormat::Json
        } else {
            match options.output_format {
                CliOutputFormat::Text => crate::output_formats::OutputFormat::Text,
                CliOutputFormat::Json => crate::output_formats::OutputFormat::Json,
                CliOutputFormat::Xml => crate::output_formats::OutputFormat::Xml,
                CliOutputFormat::Html => crate::output_formats::OutputFormat::Html,
                CliOutputFormat::Markdown => crate::output_formats::OutputFormat::Markdown,
                CliOutputFormat::Csv => crate::output_formats::OutputFormat::Csv,
                CliOutputFormat::Tsv => crate::output_formats::OutputFormat::Tsv,
            }
        })
        .with_ndjson(options.ndjson);

        let output =
            formatter.format_results(matches, &options.original_pattern, Path::new("<stdin>"));

        if options.output_format == CliOutputFormat::Json || options.ndjson {
            print!("{output}");
        } else {
            println!("\n{output}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdin_searcher_creation() {
        let searcher = StdinSearcher::new();
        assert!(std::mem::size_of_val(&searcher) == 0); // Zero-sized type
    }
}
