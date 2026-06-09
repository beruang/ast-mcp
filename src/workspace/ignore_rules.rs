//! Ignore rules — determine which directories/paths to skip during workspace scans.
//! V2 uses a hardcoded list. The `ignore` crate integration is deferred to a later release.

use std::path::Path;

/// Returns true if the given directory component should be skipped.
pub fn is_ignored_dir_name(name: &str) -> bool {
    matches!(
        name,
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
            | ".idea"
            | ".vscode"
    )
}

/// Returns true if a file extension is one we can parse.
pub fn is_supported_extension(ext: &str) -> bool {
    matches!(ext, "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "py" | "go" | "rs")
}

/// Quick check whether a path should be skipped entirely.
pub fn should_skip_path(path: &Path) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(s) = c {
            is_ignored_dir_name(&s.to_string_lossy())
        } else {
            false
        }
    })
}
