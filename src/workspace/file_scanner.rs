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
            | "vendor"
            | ".venv"
            | "venv"
            | "__pycache__"
            | ".next"
            | ".nuxt"
            | ".turbo"
    )
}

fn simple_glob_match(pattern: &str, path: &str) -> bool {
    // Convert a glob pattern with ** and * into a simple matcher.
    // Split on ** to handle recursive parts, then match each segment with * wildcard support.
    let segments: Vec<&str> = pattern.split("**").collect();

    if segments.len() == 1 {
        return single_star_match(segments[0], path);
    }

    // Has ** — match prefix, intermediate segments, and suffix
    let mut remaining = path;
    for (i, seg) in segments.iter().enumerate() {
        if seg.is_empty() {
            continue;
        }
        if i == 0 {
            // First segment: must match prefix
            if let Some(tail) = single_star_prefix_match(seg, remaining) {
                remaining = tail;
            } else {
                return false;
            }
        } else if i == segments.len() - 1 {
            // Last segment: must match suffix
            return single_star_suffix_match(seg, remaining);
        } else {
            // Middle segment: must appear somewhere
            if let Some(pos) = find_star_match(seg, remaining) {
                remaining = &remaining[pos..];
            } else {
                return false;
            }
        }
    }
    true
}

/// Match a pattern that may contain * as a wildcard (no **).
fn single_star_match(pat: &str, path: &str) -> bool {
    let Some(star_idx) = pat.find('*') else {
        return path == pat || path.contains(pat);
    };
    let prefix = &pat[..star_idx];
    let suffix = &pat[star_idx + 1..];
    path.starts_with(prefix)
        && path[prefix.len()..].ends_with(suffix)
        && path.len() >= prefix.len() + suffix.len()
}

/// Match pattern as a prefix of path, returning the remaining tail. Pattern may contain *.
fn single_star_prefix_match<'a>(pat: &str, path: &'a str) -> Option<&'a str> {
    let star_idx = pat.find('*')?;
    let prefix = &pat[..star_idx];
    let suffix = &pat[star_idx + 1..];
    let rest = path.strip_prefix(prefix)?;
    if suffix.is_empty() {
        return Some(rest);
    }
    let pos = rest.find(suffix)?;
    Some(&rest[pos + suffix.len()..])
}

/// Match pattern as a suffix of path. Pattern may contain *.
fn single_star_suffix_match(pat: &str, path: &str) -> bool {
    let Some(star_idx) = pat.find('*') else {
        return path.ends_with(pat);
    };
    let prefix = &pat[..star_idx];
    let suffix = &pat[star_idx + 1..];
    path.ends_with(suffix)
        && path.len() >= prefix.len() + suffix.len()
        && path[..path.len() - suffix.len()].ends_with(prefix)
}

/// Find the position where a star-pattern matches within path.
fn find_star_match(pat: &str, path: &str) -> Option<usize> {
    let star_idx = pat.find('*')?;
    let prefix = &pat[..star_idx];
    let suffix = &pat[star_idx + 1..];
    let start = path.find(prefix)?;
    let after_prefix = &path[start + prefix.len()..];
    if suffix.is_empty() {
        return Some(start);
    }
    let end = after_prefix.find(suffix)?;
    Some(start + prefix.len() + end + suffix.len())
}
