use std::collections::HashMap;

/// SIMD-optimized string search using CPU-specific intrinsics (AVX2/SSE4.2)
pub struct SimdSearch {
    engine: crate::simd::SimdSearchEngine,
    pattern: Vec<u8>,
    pattern_str: String,
}

impl SimdSearch {
    pub fn new(pattern: &str) -> Self {
        Self {
            engine: crate::simd::SimdSearchEngine::new(pattern),
            pattern: pattern.as_bytes().to_vec(),
            pattern_str: pattern.to_string(),
        }
    }

    /// Ultra-fast SIMD search using hardware acceleration
    pub fn search(&self, text: &str, _pattern: &str) -> Vec<usize> {
        self.engine.search(text)
    }

    /// Search with context lines
    pub fn search_with_context(
        &self,
        text: &str,
        _pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let matches = self.search(text, "");
        let lines: Vec<&str> = text.lines().collect();
        let mut results = Vec::new();

        for &match_pos in &matches {
            let pre_lines = text[..match_pos].lines().count();
            let line_number = pre_lines.max(1);
            let line_index = line_number - 1;

            if line_index < lines.len() {
                let line = lines[line_index];
                let context_before = self.get_context_before(&lines, line_index, context_lines);
                let context_after = self.get_context_after(&lines, line_index, context_lines);

                let column_start = match_pos - text[..match_pos].rfind('\n').unwrap_or(0);
                let column_end = column_start + self.pattern.len();
                let matched_text = if column_start < line.len() && column_end <= line.len() {
                    line[column_start..column_end].to_string()
                } else {
                    self.pattern_str.clone()
                };

                results.push(SearchMatch {
                    line_number,
                    line: line.to_string(),
                    context_before,
                    context_after,
                    matched_text,
                    column_start,
                    column_end,
                });
            }
        }

        results
    }

    fn get_context_before(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let start = current_line.saturating_sub(context_lines);
        (start..current_line)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }

    fn get_context_after(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let end = (current_line + context_lines + 1).min(lines.len());
        ((current_line + 1)..end)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }
}

/// Boyer-Moore string search algorithm for efficient text matching
pub struct BoyerMoore {
    pattern: Vec<u8>,
    bad_char_table: HashMap<u8, usize>,
    good_suffix_table: Vec<usize>,
}

impl BoyerMoore {
    pub fn new(pattern: &str) -> Self {
        let pattern_bytes = pattern.as_bytes().to_vec();
        let bad_char_table = Self::build_bad_char_table(&pattern_bytes);
        let good_suffix_table = Self::build_good_suffix_table(&pattern_bytes);

        Self {
            pattern: pattern_bytes,
            bad_char_table,
            good_suffix_table,
        }
    }

    /// Build the bad character table for Boyer-Moore algorithm
    fn build_bad_char_table(pattern: &[u8]) -> HashMap<u8, usize> {
        let mut table = HashMap::new();
        let pattern_len = pattern.len();

        for (i, &byte) in pattern.iter().enumerate() {
            table.insert(byte, pattern_len - 1 - i);
        }

        table
    }

    /// Build the good suffix table for Boyer-Moore algorithm
    fn build_good_suffix_table(pattern: &[u8]) -> Vec<usize> {
        let pattern_len = pattern.len();
        let mut table = vec![1; pattern_len];

        if pattern_len > 1 {
            table[pattern_len - 2] = pattern_len;
        }

        table
    }

    /// Search for the pattern in the given text
    pub fn search(&self, text: &str, _pattern: &str) -> Vec<usize> {
        let text_bytes = text.as_bytes();
        let pattern_len = self.pattern.len();
        let text_len = text_bytes.len();
        let mut matches = Vec::new();

        if pattern_len == 0 || text_len < pattern_len {
            return matches;
        }

        let mut i = pattern_len - 1;
        while i < text_len {
            let mut j = pattern_len - 1;
            let mut k = i;

            while j > 0 && text_bytes[k] == self.pattern[j] {
                k -= 1;
                j -= 1;
            }

            if j == 0 && text_bytes[k] == self.pattern[0] {
                matches.push(k);
            }

            let bad_char_shift = self
                .bad_char_table
                .get(&text_bytes[i])
                .unwrap_or(&pattern_len);
            let good_suffix_shift = if j < pattern_len - 1 {
                self.good_suffix_table[j + 1]
            } else {
                1
            };

            let shift = bad_char_shift.max(&good_suffix_shift);
            i += shift;
        }

        matches
    }

    /// Search for all occurrences with context
    pub fn search_with_context(
        &self,
        text: &str,
        _pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let matches = self.search(text, "");
        let lines: Vec<&str> = text.lines().collect();
        let mut results = Vec::new();

        for &match_pos in &matches {
            let pre_lines = text[..match_pos].lines().count();
            let line_number = pre_lines.max(1);
            let line_index = line_number - 1;

            if line_index < lines.len() {
                let line = lines[line_index];
                let context_before = self.get_context_before(&lines, line_index, context_lines);
                let context_after = self.get_context_after(&lines, line_index, context_lines);

                let line_start = text[..match_pos].rfind('\n').unwrap_or(0);
                let column_start = match_pos - line_start;
                let column_end = column_start + self.pattern.len();
                let matched_text = if column_start < line.len() && column_end <= line.len() {
                    line[column_start..column_end].to_string()
                } else {
                    self.pattern.iter().map(|&b| b as char).collect()
                };

                results.push(SearchMatch {
                    line_number,
                    line: line.to_string(),
                    context_before,
                    context_after,
                    matched_text,
                    column_start,
                    column_end,
                });
            }
        }

        results
    }

    fn get_context_before(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let start = current_line.saturating_sub(context_lines);
        (start..current_line)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }

    fn get_context_after(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let end = (current_line + context_lines + 1).min(lines.len());
        ((current_line + 1)..end)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }
}

/// Search match result with context
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line_number: usize,
    pub line: String,
    pub context_before: Vec<(usize, String)>,
    pub context_after: Vec<(usize, String)>,
    pub matched_text: String,
    pub column_start: usize,
    pub column_end: usize,
}

/// Search algorithm types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub enum SearchAlgorithm {
    Simd, // New SIMD-optimized search
    BoyerMoore,
    Regex,
    Simple,
}

/// Search algorithm factory
pub struct SearchAlgorithmFactory;

impl SearchAlgorithmFactory {
    pub fn create(algorithm: SearchAlgorithm, pattern: &str) -> Box<dyn SearchAlgorithmTrait> {
        match algorithm {
            SearchAlgorithm::Simd => Box::new(SimdSearch::new(pattern)),
            SearchAlgorithm::BoyerMoore => Box::new(BoyerMoore::new(pattern)),
            SearchAlgorithm::Regex => Box::new(RegexSearch::new(pattern)),
            SearchAlgorithm::Simple => Box::new(SimpleSearch::new(pattern)),
        }
    }

    pub fn create_with_case_sensitivity(
        algorithm: SearchAlgorithm,
        pattern: &str,
        case_sensitive: bool,
    ) -> Box<dyn SearchAlgorithmTrait> {
        match algorithm {
            SearchAlgorithm::Simd => Box::new(SimdSearch::new(pattern)),
            SearchAlgorithm::BoyerMoore => Box::new(BoyerMoore::new(pattern)),
            SearchAlgorithm::Regex => Box::new(RegexSearch::new(pattern)),
            SearchAlgorithm::Simple => {
                if case_sensitive {
                    Box::new(SimpleSearch::new_case_sensitive(pattern))
                } else {
                    Box::new(SimpleSearch::new(pattern))
                }
            }
        }
    }
}

/// Trait for search algorithms
pub trait SearchAlgorithmTrait: Send + Sync {
    #[allow(dead_code)]
    fn search(&self, text: &str, pattern: &str) -> Vec<usize>;
    fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch>;

    fn get_context_before(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let start = current_line.saturating_sub(context_lines);
        (start..current_line)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }

    fn get_context_after(
        &self,
        lines: &[&str],
        current_line: usize,
        context_lines: usize,
    ) -> Vec<(usize, String)> {
        let end = (current_line + context_lines + 1).min(lines.len());
        ((current_line + 1)..end)
            .map(|i| (i + 1, lines[i].to_string()))
            .collect()
    }
}

impl SearchAlgorithmTrait for SimdSearch {
    fn search(&self, text: &str, pattern: &str) -> Vec<usize> {
        let _ = pattern; // pattern ignored; self.pattern is used
        SimdSearch::search(self, text, "")
    }

    fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let _ = pattern;
        SimdSearch::search_with_context(self, text, "", context_lines)
    }
}

impl SearchAlgorithmTrait for BoyerMoore {
    fn search(&self, text: &str, pattern: &str) -> Vec<usize> {
        self.search(text, pattern)
    }

    fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        self.search_with_context(text, pattern, context_lines)
    }
}

/// Simple text search implementation
pub struct SimpleSearch {
    pattern: String,
    case_sensitive: bool,
}

impl SimpleSearch {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_sensitive: true,
        }
    }

    pub fn new_case_sensitive(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            case_sensitive: true,
        }
    }
}

impl SimpleSearch {
    pub fn search(&self, text: &str, _pattern: &str) -> Vec<usize> {
        let mut matches = Vec::new();
        let mut pos = 0;

        let search_text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };

        while let Some(found_pos) = search_text[pos..].find(&self.pattern) {
            matches.push(pos + found_pos);
            pos += found_pos + 1;

            if pos >= search_text.len() {
                break;
            }
        }

        matches
    }

    pub fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let matches = self.search(text, pattern);
        let lines: Vec<&str> = text.lines().collect();
        let mut results = Vec::new();

        for &match_pos in &matches {
            let pre_lines = text[..match_pos].lines().count();
            let line_number = pre_lines.max(1);
            let line_index = line_number - 1;

            if line_index < lines.len() {
                let line = lines[line_index];
                let context_before = self.get_context_before(&lines, line_index, context_lines);
                let context_after = self.get_context_after(&lines, line_index, context_lines);

                let line_start = text[..match_pos].rfind('\n').unwrap_or(0);
                let column_start = match_pos - line_start;
                let column_end = column_start + pattern.len();
                let matched_text = if column_start < line.len() && column_end <= line.len() {
                    line[column_start..column_end].to_string()
                } else {
                    pattern.to_string()
                };

                results.push(SearchMatch {
                    line_number,
                    line: line.to_string(),
                    context_before,
                    context_after,
                    matched_text,
                    column_start,
                    column_end,
                });
            }
        }

        results
    }
}

impl SearchAlgorithmTrait for SimpleSearch {
    fn search(&self, text: &str, pattern: &str) -> Vec<usize> {
        self.search(text, pattern)
    }

    fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        self.search_with_context(text, pattern, context_lines)
    }
}

/// Regex search implementation
pub struct RegexSearch {
    #[allow(dead_code)]
    pattern: String,
    regex: regex::Regex,
}

impl RegexSearch {
    pub fn new(pattern: &str) -> Self {
        let regex = regex::Regex::new(pattern).expect("Invalid regex pattern");
        Self {
            pattern: pattern.to_string(),
            regex,
        }
    }

    pub fn search(&self, text: &str, _pattern: &str) -> Vec<usize> {
        self.regex.find_iter(text).map(|m| m.start()).collect()
    }

    pub fn search_with_context(
        &self,
        text: &str,
        _pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        let matches = self.search(text, "");
        let lines: Vec<&str> = text.lines().collect();
        let mut results = Vec::new();

        for &match_pos in &matches {
            let pre_lines = text[..match_pos].lines().count();
            let line_number = pre_lines.max(1);
            let line_index = line_number - 1;

            if line_index < lines.len() {
                let line = lines[line_index];
                let context_before = self.get_context_before(&lines, line_index, context_lines);
                let context_after = self.get_context_after(&lines, line_index, context_lines);

                let matched_text = self
                    .regex
                    .find(&text[match_pos..])
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();

                let matched_len = matched_text.len();

                results.push(SearchMatch {
                    line_number,
                    line: line.to_string(),
                    context_before,
                    context_after,
                    matched_text,
                    column_start: match_pos - text[..match_pos].rfind('\n').unwrap_or(0),
                    column_end: match_pos - text[..match_pos].rfind('\n').unwrap_or(0)
                        + matched_len,
                });
            }
        }

        results
    }
}

impl SearchAlgorithmTrait for RegexSearch {
    fn search(&self, text: &str, pattern: &str) -> Vec<usize> {
        self.search(text, pattern)
    }

    fn search_with_context(
        &self,
        text: &str,
        pattern: &str,
        context_lines: usize,
    ) -> Vec<SearchMatch> {
        self.search_with_context(text, pattern, context_lines)
    }
}
