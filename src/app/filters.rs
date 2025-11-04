/// File filtering logic for rfgrep
///
/// This module provides comprehensive file filtering capabilities including:
/// - Extension-based filtering (include/exclude)
/// - Size-based filtering
/// - Safety policy enforcement
/// - File type strategy application
/// - Binary file detection
use crate::cli::{FileTypeStrategy, SafetyPolicy};
use crate::file_types::{FileTypeClassifier, SearchDecision};
use std::path::Path;

/// Configuration options for file filtering
#[derive(Debug, Clone)]
pub struct FileFilterOptions {
    pub max_size: Option<usize>,
    pub skip_binary: bool,
    pub safety_policy: SafetyPolicy,
    pub include_extensions: Option<Vec<String>>,
    pub exclude_extensions: Option<Vec<String>>,
    pub search_all_files: bool,
    pub text_only: bool,
    pub file_types: FileTypeStrategy,
}

impl Default for FileFilterOptions {
    fn default() -> Self {
        Self {
            max_size: None,
            skip_binary: false,
            safety_policy: SafetyPolicy::Default,
            include_extensions: None,
            exclude_extensions: None,
            search_all_files: false,
            text_only: false,
            file_types: FileTypeStrategy::Default,
        }
    }
}

/// Handler for filtering files based on various criteria
pub struct FileFilter {
    options: FileFilterOptions,
}

impl FileFilter {
    /// Create a new file filter with the given options
    pub fn new(options: FileFilterOptions) -> Self {
        Self { options }
    }

    /// Filter a list of files based on configured criteria
    ///
    /// # Arguments
    ///
    /// * `files` - Vector of file paths to filter
    ///
    /// # Returns
    ///
    /// Filtered vector containing only files that pass all filter criteria
    pub fn filter_files(&self, files: Vec<std::path::PathBuf>) -> Vec<std::path::PathBuf> {
        files
            .into_iter()
            .filter(|path| self.should_search_file(path))
            .collect()
    }

    /// Determine if a specific file should be searched
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to check
    ///
    /// # Returns
    ///
    /// `true` if the file passes all filter criteria, `false` otherwise
    pub fn should_search_file(&self, path: &Path) -> bool {
        let metadata = match path.metadata() {
            Ok(m) => m,
            Err(_) => return false,
        };

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();

        // Check binary files
        if self.options.skip_binary && crate::processor::is_binary(path) {
            return false;
        }

        // Apply safety policy
        if !self.apply_safety_policy(&metadata, &ext) {
            return false;
        }

        // Check extension filters
        if !self.apply_extension_filters(&ext) {
            return false;
        }

        // Check file type strategy
        if !self.should_search_by_file_type(path, &metadata, &ext) {
            return false;
        }

        // Check size limits
        if !self.apply_size_limits(&metadata) {
            return false;
        }

        true
    }

    /// Apply safety policy constraints
    fn apply_safety_policy(&self, metadata: &std::fs::Metadata, ext: &str) -> bool {
        match self.options.safety_policy {
            SafetyPolicy::Conservative => {
                let file_size = metadata.len();
                if file_size > 10 * 1024 * 1024 {
                    return false;
                }

                let classifier = FileTypeClassifier::new();
                classifier.is_always_search(ext)
            }
            SafetyPolicy::Performance => {
                let file_size = metadata.len();
                file_size <= 500 * 1024 * 1024
            }
            SafetyPolicy::Default => true,
        }
    }

    /// Apply extension filters (include/exclude)
    fn apply_extension_filters(&self, ext: &str) -> bool {
        // Handle include extensions
        if let Some(ref include_exts) = self.options.include_extensions {
            if !include_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                return false;
            }
        }

        // Handle exclude extensions
        if let Some(ref exclude_exts) = self.options.exclude_extensions {
            if exclude_exts.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                return false;
            }
        }

        true
    }

    /// Determine if file should be searched based on file type strategy
    fn should_search_by_file_type(
        &self,
        path: &Path,
        metadata: &std::fs::Metadata,
        ext: &str,
    ) -> bool {
        if self.options.search_all_files {
            return true;
        }

        if self.options.text_only {
            let classifier = FileTypeClassifier::new();
            return classifier.is_always_search(ext);
        }

        let classifier = FileTypeClassifier::new();
        match self.options.file_types {
            FileTypeStrategy::Comprehensive => !classifier.is_never_search(ext),
            FileTypeStrategy::Conservative => classifier.is_always_search(ext),
            FileTypeStrategy::Performance => {
                classifier.is_always_search(ext) || classifier.is_conditional_search(ext)
            }
            FileTypeStrategy::Default => {
                matches!(
                    classifier.should_search(path, metadata),
                    SearchDecision::Search(_) | SearchDecision::Conditional(_, _)
                )
            }
        }
    }

    /// Apply size limits
    fn apply_size_limits(&self, metadata: &std::fs::Metadata) -> bool {
        if let Some(max_size) = self.options.max_size {
            let size_mb = metadata.len() as f64 / (1024.0 * 1024.0);
            if size_mb > max_size as f64 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_file_filter_creation() {
        let options = FileFilterOptions::default();
        let _filter = FileFilter::new(options);
    }

    #[test]
    fn test_extension_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let test_file_rs = temp_dir.path().join("test.rs");
        let test_file_txt = temp_dir.path().join("test.txt");

        File::create(&test_file_rs)
            .unwrap()
            .write_all(b"test")
            .unwrap();
        File::create(&test_file_txt)
            .unwrap()
            .write_all(b"test")
            .unwrap();

        let options = FileFilterOptions {
            include_extensions: Some(vec!["rs".to_string()]),
            ..Default::default()
        };

        let filter = FileFilter::new(options);

        assert!(filter.should_search_file(&test_file_rs));
        assert!(!filter.should_search_file(&test_file_txt));
    }

    #[test]
    fn test_size_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let small_file = temp_dir.path().join("small.txt");
        let large_file = temp_dir.path().join("large.txt");

        File::create(&small_file)
            .unwrap()
            .write_all(b"small")
            .unwrap();
        File::create(&large_file)
            .unwrap()
            .write_all(&vec![b'x'; 2_000_000])
            .unwrap();

        let options = FileFilterOptions {
            max_size: Some(1), // 1MB limit
            ..Default::default()
        };

        let filter = FileFilter::new(options);

        assert!(filter.should_search_file(&small_file));
        assert!(!filter.should_search_file(&large_file));
    }
}
