/// Application submodules for rfgrep
///
/// This module contains the decomposed components of the main application,
/// separated by responsibility for better maintainability and testability.
pub mod filters;
pub mod stdin;

pub use filters::{FileFilter, FileFilterOptions};
pub use stdin::StdinSearcher;
