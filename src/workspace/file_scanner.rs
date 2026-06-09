//! File scanner — walk workspace respecting ignore rules and globs.
use std::path::PathBuf;

use crate::config::defaults::{MAX_BYTES_PER_WORKSPACE_FILE, MAX_WORKSPACE_QUERY_FILES};
use crate::parser;
use crate::shared::language::LanguageId;

pub struct ScanOptions {
    pub root: PathBuf,
    pub glob: Option<String>,
    pub max_files: usize,
    pub max_bytes_per_file: usize,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::new(),
            glob: None,
            max_files: MAX_WORKSPACE_QUERY_FILES,
            max_bytes_per_file: MAX_BYTES_PER_WORKSPACE_FILE,
        }
    }
}

/// Scan workspace for files matching the options. Returns (path, language_id) pairs.
pub fn scan_files(opts: &ScanOptions) -> Vec<(PathBuf, Option<LanguageId>)> {
    let mut results: Vec<(PathBuf, Option<LanguageId>)> = Vec::new();

    // Simple walkdir-based scan
    let walker = walkdir::WalkDir::new(&opts.root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e));

    for entry in walker {
        if results.len() >= opts.max_files {
            break;
        }
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let dotted = format!(".{}", ext);

        // Glob filter
        if let Some(ref glob) = opts.glob {
            let relative = path.strip_prefix(&opts.root).unwrap_or(path);
            let rel_str = relative.to_string_lossy().replace('\\', "/");
            if !simple_glob_match(glob, &rel_str) {
                continue;
            }
        }

        // Size filter
        if let Ok(meta) = path.metadata() {
            if meta.len() > opts.max_bytes_per_file as u64 {
                continue;
            }
        }

        let lang = parser::registry::for_extension(&dotted).map(|d| d.language);
        results.push((path.to_path_buf(), lang));
    }

    results
}

fn is_ignored_dir(entry: &walkdir::DirEntry) -> bool {
    if !entry.file_type().is_dir() {
        return false;
    }
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git"
            | "node_modules"
            | "dist"
            | "build"
            | "coverage"
            | ".target"
            | "target"
            | ".venv"
            | "venv"
            | "__pycache__"
            | ".next"
            | ".turbo"
    )
}

fn simple_glob_match(pattern: &str, path: &str) -> bool {
    // Minimal glob: support ** and *
    let parts: Vec<&str> = pattern.split("**").collect();
    if parts.len() == 1 {
        // No ** — simple prefix/suffix match
        if let Some(stripped) = pattern.strip_suffix('*') {
            return path.starts_with(stripped);
        }
        if let Some(stripped) = pattern.strip_prefix('*') {
            return path.ends_with(stripped);
        }
        return path.contains(pattern);
    }
    // Has ** — match prefix and suffix
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 && !path.starts_with(part) {
            return false;
        }
        if i == parts.len() - 1 && !path.ends_with(part) {
            return false;
        }
        if i > 0 && i < parts.len() - 1 && !path.contains(part) {
            return false;
        }
    }
    true
}
