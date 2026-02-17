//! Streaming search pipeline for efficient file processing
use crate::error::{Result as RfgrepResult, RfgrepError};
use crate::processor::SearchMatch as ProcessorSearchMatch;
use crate::search_algorithms::{SearchAlgorithm, SearchAlgorithmTrait, SearchMatch};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task;

/// Configuration for streaming search
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    pub algorithm: SearchAlgorithm,
    pub context_lines: usize,
    pub case_sensitive: bool,
    pub invert_match: bool,
    pub max_matches: Option<usize>,
    pub timeout_per_file: Option<u64>,
    pub chunk_size: usize,
    pub buffer_size: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            algorithm: SearchAlgorithm::BoyerMoore,
            context_lines: 2,
            case_sensitive: true,
            invert_match: false,
            max_matches: None,
            timeout_per_file: None,
            chunk_size: 8192,   // 8KB chunks
            buffer_size: 65536, // 64KB buffer
        }
    }
}

/// Streaming search pipeline
#[derive(Clone)]
pub struct StreamingSearchPipeline {
    config: StreamingConfig,
}

impl StreamingSearchPipeline {
    /// Fast-exit search: returns true if any match is found, exits early
    pub async fn search_file_fast_exit(&self, path: &Path, pattern: &str) -> RfgrepResult<bool> {
        if crate::processor::is_binary(path) {
            return Ok(false);
        }
        use memchr::memmem;
        use memmap2::Mmap;
        let file = std::fs::File::open(path).map_err(crate::error::RfgrepError::Io)?;
        let metadata = file.metadata().map_err(crate::error::RfgrepError::Io)?;
        let mmap_threshold = crate::processor::get_adaptive_mmap_threshold();
        let finder = memmem::Finder::new(pattern.as_bytes());
        let found = if metadata.len() >= mmap_threshold {
            // Use mmap for large files
            let mmap = unsafe { Mmap::map(&file).map_err(crate::error::RfgrepError::Io)? };
            // skipcq: RS-W1033 - Finder::find() only returns Option<usize>, .is_some() is correct
            finder.find(&mmap).is_some()
        } else {
            // Zero-copy: read file into buffer, avoid extra allocations
            let buf = std::fs::read(path).map_err(crate::error::RfgrepError::Io)?;
            // skipcq: RS-W1033 - Finder::find() only returns Option<usize>, .is_some() is correct
            finder.find(&buf).is_some()
        };
        Ok(found)
    }
    pub fn new(config: StreamingConfig) -> Self {
        Self { config }
    }

    /// Search a single file using streaming approach
    pub async fn search_file(
        &self,
        path: &Path,
        pattern: &str,
    ) -> RfgrepResult<Vec<ProcessorSearchMatch>> {
        // Early binary check
        if crate::processor::is_binary(path) {
            return Ok(vec![]);
        }

        // Helper future that performs the actual search
        let do_search = async {
            if let Some(
                crate::compression::CompressionType::Zip | crate::compression::CompressionType::Tar,
            ) = crate::compression::CompressionType::from_extension(path)
            {
                let pat_str = if !self.config.case_sensitive {
                    format!("(?i){}", pattern)
                } else {
                    pattern.to_string()
                };
                let regex = crate::processor::get_or_compile_regex(&pat_str)?;
                let matches = crate::archive::search_archive(path, &regex)?;

                // Archive matching uses processor::SearchMatch directly.
                // Post-processing (invert match) is skipped as find_matches_streaming only returns positive matches.
                let mut final_matches = matches;

                if let Some(max_matches) = self.config.max_matches {
                    if final_matches.len() > max_matches {
                        final_matches.truncate(max_matches);
                    }
                }
                return RfgrepResult::Ok(final_matches);
            }

            let reader: Box<dyn Read + Send> = if let Some(compression) =
                crate::compression::CompressionType::from_extension(path)
            {
                crate::compression::open_compressed_stream(path, compression)
                    .map_err(RfgrepError::Io)?
            } else {
                let file = File::open(path).map_err(RfgrepError::Io)?;
                Box::new(file)
            };

            let reader = BufReader::with_capacity(self.config.buffer_size, reader);

            // Create search algorithm instance
            let search_algo = self.create_search_algorithm(pattern)?;

            // Process file in chunks
            let matches = self
                .process_file_streaming(reader, search_algo.as_ref(), pattern, path)
                .await?;

            // Apply post-processing
            let mut final_matches = self.apply_post_processing(matches, path)?;

            // Apply max_matches limit
            if let Some(max_matches) = self.config.max_matches {
                if final_matches.len() > max_matches {
                    final_matches.truncate(max_matches);
                }
            }

            RfgrepResult::Ok(final_matches)
        };

        if let Some(timeout_secs) = self.config.timeout_per_file {
            // If test env variable is set, simulate work taking time
            if let Ok(s) = std::env::var("RFGREP_WORKER_SLEEP") {
                if let Ok(sec) = s.parse::<u64>() {
                    // Sleep inside the timed section to simulate long-running work
                    let sleep_fut = async {
                        tokio::time::sleep(std::time::Duration::from_secs(sec)).await;
                        do_search.await
                    };
                    return match tokio::time::timeout(
                        std::time::Duration::from_secs(timeout_secs),
                        sleep_fut,
                    )
                    .await
                    {
                        Ok(res) => res,
                        Err(_elapsed) => Ok(vec![]),
                    };
                }
            }
            // Enforce per-file timeout: on timeout, return no matches
            match tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), do_search)
                .await
            {
                Ok(res) => res,
                Err(_elapsed) => Ok(vec![]),
            }
        } else {
            do_search.await
        }
    }

    /// Search multiple files in parallel
    pub async fn search_files_parallel(
        &self,
        files: &[&Path],
        pattern: &str,
        max_concurrent: usize,
    ) -> RfgrepResult<Vec<ProcessorSearchMatch>> {
        let (tx, mut rx) = mpsc::channel::<RfgrepResult<Vec<ProcessorSearchMatch>>>(files.len());
        let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
        let config = Arc::new(self.config.clone());
        let pattern = Arc::new(pattern.to_string());

        // Spawn tasks for each file
        for file_path in files {
            let tx = tx.clone();
            let semaphore = semaphore.clone();
            let config = config.clone();
            let pattern = pattern.clone();
            let file_path = (*file_path).to_path_buf();

            task::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                let pipeline = StreamingSearchPipeline::new((*config).clone());
                let result = pipeline.search_file(&file_path, &pattern).await;
                let _ = tx.send(result).await;
            });
        }

        drop(tx); // Close the sender

        // Collect results
        let mut all_matches = Vec::new();
        while let Some(result) = rx.recv().await {
            match result {
                Ok(matches) => all_matches.extend(matches),
                Err(e) => {
                    eprintln!("Error in parallel search: {e}");
                }
            }
        }

        // Sort results
        all_matches.sort();
        Ok(all_matches)
    }

    fn create_search_algorithm(
        &self,
        pattern: &str,
    ) -> RfgrepResult<Box<dyn SearchAlgorithmTrait>> {
        use crate::search_algorithms::SearchAlgorithmFactory;

        // For non-regex algorithms, we need to handle case sensitivity differently
        let processed_pattern = match self.config.algorithm {
            crate::search_algorithms::SearchAlgorithm::Regex => {
                if self.config.case_sensitive {
                    pattern.to_string()
                } else {
                    format!("(?i){pattern}")
                }
            }
            _ => {
                // For Boyer-Moore, Simple, and Simd, keep the original pattern
                // Case sensitivity will be handled in the search process
                pattern.to_string()
            }
        };

        Ok(SearchAlgorithmFactory::create_with_case_sensitivity(
            self.config.algorithm.clone(),
            &processed_pattern,
            self.config.case_sensitive,
        ))
    }

    async fn process_file_streaming<R: Read + Send + 'static>(
        &self,
        reader: BufReader<R>,
        search_algo: &dyn SearchAlgorithmTrait,
        pattern: &str,
        _path: &Path,
    ) -> RfgrepResult<Vec<SearchMatch>> {
        let mut matches = Vec::new();
        let mut lines = reader.lines();
        let mut line_number = 0;
        let mut context_buffer = Vec::new();

        while let Some(line_result) = lines.next() {
            line_number += 1;
            let line = match line_result {
                Ok(line) => line,
                Err(e) => {
                    // Skip lines that can't be read as UTF-8 (likely binary content)
                    if e.kind() == std::io::ErrorKind::InvalidData {
                        continue;
                    }
                    return Err(RfgrepError::Io(e));
                }
            };

            // Add to context buffer
            context_buffer.push((line_number, line.clone()));
            if context_buffer.len() > self.config.context_lines * 2 + 1 {
                context_buffer.remove(0);
            }

            // Search in current line
            let line_matches = search_algo.search(&line, pattern);

            for match_pos in line_matches {
                let context_before = self.get_context_before(&context_buffer, line_number);
                let context_after =
                    self.get_context_after(&context_buffer, line_number, &mut lines)?;

                let matched_text = if match_pos + 1 < line.len() {
                    line[match_pos..].chars().take(50).collect::<String>()
                } else {
                    line.clone()
                };

                matches.push(SearchMatch {
                    line_number,
                    line: line.clone(),
                    context_before,
                    context_after,
                    matched_text,
                    column_start: match_pos,
                    column_end: match_pos + 1,
                });
            }
        }

        Ok(matches)
    }

    fn get_context_before(
        &self,
        context_buffer: &[(usize, String)],
        current_line: usize,
    ) -> Vec<(usize, String)> {
        let start = current_line.saturating_sub(self.config.context_lines);
        context_buffer
            .iter()
            .filter(|(line_num, _)| *line_num >= start && *line_num < current_line)
            .cloned()
            .collect()
    }

    fn get_context_after<R: Read + Send + 'static>(
        &self,
        _context_buffer: &[(usize, String)],
        current_line: usize,
        lines: &mut std::io::Lines<BufReader<R>>,
    ) -> RfgrepResult<Vec<(usize, String)>> {
        let mut context_after = Vec::new();
        let mut line_number = current_line;

        for _ in 0..self.config.context_lines {
            if let Some(line_result) = lines.next() {
                line_number += 1;
                let line = line_result.map_err(RfgrepError::Io)?;
                context_after.push((line_number, line));
            } else {
                break;
            }
        }

        Ok(context_after)
    }

    fn apply_post_processing(
        &self,
        matches: Vec<SearchMatch>,
        path: &Path,
    ) -> RfgrepResult<Vec<ProcessorSearchMatch>> {
        let mut processor_matches = Vec::new();

        for search_match in matches {
            // Apply invert_match logic
            let should_include = if self.config.invert_match {
                search_match.matched_text.is_empty()
            } else {
                !search_match.matched_text.is_empty()
            };

            if should_include {
                processor_matches.push(ProcessorSearchMatch {
                    path: path.to_path_buf(),
                    line_number: search_match.line_number,
                    line: search_match.line,
                    context_before: search_match.context_before,
                    context_after: search_match.context_after,
                    matched_text: search_match.matched_text,
                    column_start: search_match.column_start,
                    column_end: search_match.column_end,
                });
            }
        }

        Ok(processor_matches)
    }
}

/// Utility functions for streaming search
pub mod utils {
    use super::*;
    use std::collections::HashMap;

    /// Analyze file patterns to optimize search strategy
    pub fn analyze_file_patterns(files: &[&Path]) -> HashMap<String, usize> {
        let mut extensions = HashMap::new();

        for file in files {
            if let Some(ext) = file.extension().and_then(|s| s.to_str()) {
                *extensions.entry(ext.to_lowercase()).or_insert(0) += 1;
            }
        }

        extensions
    }

    /// Suggest optimal algorithm based on file characteristics
    pub fn suggest_algorithm(
        pattern: &str,
        file_count: usize,
        avg_file_size: Option<u64>,
    ) -> SearchAlgorithm {
        // For very short patterns, SIMD is usually fastest
        if pattern.len() <= 4 {
            return SearchAlgorithm::BoyerMoore; // We'll use BoyerMoore as SIMD isn't in CLI yet
        }

        // For regex patterns, use regex algorithm
        if pattern.contains("\\") || pattern.contains("[") || pattern.contains("(") {
            return SearchAlgorithm::Regex;
        }

        // For many small files, simple search might be faster
        if file_count > 1000 && avg_file_size.is_none_or(|size| size < 1024) {
            return SearchAlgorithm::Simple;
        }

        // Default to Boyer-Moore for most cases
        SearchAlgorithm::BoyerMoore
    }

    /// Estimate search performance
    pub fn estimate_performance(
        file_count: usize,
        avg_file_size: u64,
        algorithm: &SearchAlgorithm,
    ) -> (f64, String) {
        let base_time_per_mb = match algorithm {
            SearchAlgorithm::BoyerMoore => 0.1, // 100ms per MB
            SearchAlgorithm::Regex => 0.5,      // 500ms per MB
            SearchAlgorithm::Simple => 0.2,     // 200ms per MB
            SearchAlgorithm::Simd => 0.05,      // 50ms per MB (fastest)
        };

        let total_size_mb = (file_count as f64 * avg_file_size as f64) / (1024.0 * 1024.0);
        let estimated_seconds = total_size_mb * base_time_per_mb;

        let time_str = if estimated_seconds < 1.0 {
            format!("{:.0}ms", estimated_seconds * 1000.0)
        } else if estimated_seconds < 60.0 {
            format!("{estimated_seconds:.1}s")
        } else {
            format!("{:.1}m", estimated_seconds / 60.0)
        };

        (estimated_seconds, time_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[tokio::test]
    async fn test_streaming_search() {
        let config = StreamingConfig::default();
        let _pipeline = StreamingSearchPipeline::new(config);

        let content = "line1\nline2 with test\nline3\nline4 with test again\nline5";
        let cursor = Cursor::new(content);
        let _reader = BufReader::new(cursor);

        // This would need to be adapted for the actual test
        // let matches = pipeline.process_file_streaming(reader, &search_algo, Path::new("test.txt")).await.unwrap();
        // assert!(!matches.is_empty());
    }
}
