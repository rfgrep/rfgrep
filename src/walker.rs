use ignore::{DirEntry, WalkBuilder};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct WalkerOptions {
    pub recursive: bool,
    pub show_hidden: bool,
    pub respect_gitignore: bool,
    pub respect_global_gitignore: bool,
    pub respect_git_exclude: bool,
    pub search_dot_git: bool,
    pub ignore_hidden: bool,
    pub max_depth: Option<usize>,
    pub follow_links: bool,
    pub overrides: Vec<String>,
}

impl Default for WalkerOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            show_hidden: false,
            respect_gitignore: true,
            respect_global_gitignore: true,
            respect_git_exclude: true,
            search_dot_git: false,
            ignore_hidden: true, // ignore hidden by default in 'ignore' crate logic often means 'hidden: true' excludes them
            max_depth: None,
            follow_links: false,
            overrides: Vec::new(),
        }
    }
}

pub fn walk_dir(path: &Path, recursive: bool, show_hidden: bool) -> impl Iterator<Item = DirEntry> {
    let options = WalkerOptions {
        recursive,
        show_hidden,
        respect_gitignore: !show_hidden,
        respect_global_gitignore: !show_hidden,
        respect_git_exclude: !show_hidden,
        search_dot_git: false,
        ignore_hidden: !show_hidden,
        max_depth: if recursive { None } else { Some(1) },
        follow_links: false,
        overrides: Vec::new(),
    };
    walk_dir_with_options(path, options)
}

pub fn walk_dir_with_options(
    path: &Path,
    options: WalkerOptions,
) -> impl Iterator<Item = DirEntry> {
    let mut builder = WalkBuilder::new(path);

    // Configure ignoring logic
    builder
        .hidden(options.ignore_hidden) // false means "don't ignore hidden" (i.e. show them)
        .git_global(options.respect_global_gitignore)
        .git_ignore(options.respect_gitignore)
        .git_exclude(options.respect_git_exclude)
        .ignore(options.respect_gitignore) // .ignore files usually serve same purpose
        .parents(options.respect_gitignore) // look for .gitignore in parent dirs
        .max_depth(options.max_depth)
        .follow_links(options.follow_links);

    // Explicitly handle .git directory searching if requested, otherwise default ignore logic usually skips it
    // But 'ignore' crate skips .git by default if hidden() is true (default).

    if !options.overrides.is_empty() {
        let mut override_builder = ignore::overrides::OverrideBuilder::new(path);
        for pattern in &options.overrides {
            let _ = override_builder.add(pattern);
        }
        if let Ok(ov) = override_builder.build() {
            builder.overrides(ov);
        }
    }

    builder.build().filter_map(Result::ok)
}
