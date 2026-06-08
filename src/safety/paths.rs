use std::path::{Path, PathBuf};

use crate::config::defaults::MAX_FILE_BYTES;
use crate::config::workspace::Workspace;
use crate::shared::errors::AstToolError;

pub struct ResolvedFile {
    pub absolute: PathBuf,
    pub workspace_relative: String,
}

/// Validate and resolve a user-supplied file path against the workspace root.
///
/// Rejects absolute paths (unless they equal the workspace root), `..`
/// segments, symlink escapes, directories, and non-existent files.
pub fn resolve_file(workspace: &Workspace, input: &str) -> Result<ResolvedFile, AstToolError> {
    if input.is_empty() {
        return Err(AstToolError::FileNotFound("empty path".into()));
    }

    // Reject absolute paths.
    if input.starts_with('/') {
        return Err(AstToolError::PathOutsideWorkspace(format!(
            "absolute path not allowed: {}",
            input
        )));
    }

    // Reject path traversal.
    for segment in input.split('/') {
        if segment == ".." || segment == "." {
            // "." alone as a segment is fine; ".." is not.
            // We reject both "." and ".." as standalone to be safe.
            if segment == ".." {
                return Err(AstToolError::PathOutsideWorkspace(format!(
                    "path traversal detected: {}",
                    input
                )));
            }
        }
    }

    let joined = workspace.root().join(input);

    // Canonicalize — follows symlinks, normalizes separators.
    let canonical = joined
        .canonicalize()
        .map_err(|_| AstToolError::FileNotFound(format!("file not found: {}", input)))?;

    // Reject if canonical path escaped the workspace (symlink or traversal).
    if !canonical.starts_with(workspace.root()) {
        return Err(AstToolError::PathOutsideWorkspace(format!(
            "resolved path outside workspace: {}",
            input
        )));
    }

    // Reject directories.
    if canonical.is_dir() {
        return Err(AstToolError::FileNotFound(format!(
            "path is a directory, not a file: {}",
            input
        )));
    }

    // Build workspace-relative path with forward slashes.
    let workspace_relative =
        canonical.strip_prefix(workspace.root()).unwrap().to_string_lossy().replace('\\', "/");

    Ok(ResolvedFile { absolute: canonical, workspace_relative })
}

/// Return the size of a file in bytes.
pub fn file_size(p: &Path) -> Result<u64, AstToolError> {
    let meta = p
        .metadata()
        .map_err(|e| AstToolError::InternalError(format!("cannot stat {}: {}", p.display(), e)))?;
    Ok(meta.len())
}

/// Check that `size` does not exceed `MAX_FILE_BYTES`.
pub fn ensure_under_size(size: u64) -> Result<(), AstToolError> {
    if size > MAX_FILE_BYTES {
        return Err(AstToolError::FileTooLarge(size, MAX_FILE_BYTES));
    }
    Ok(())
}
