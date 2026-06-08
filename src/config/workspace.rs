use std::path::{Path, PathBuf};

use crate::shared::errors::AstToolError;

pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Read WORKSPACE_PATH from the environment, falling back to the current
    /// working directory.  The path is canonicalised so later containment
    /// checks are reliable.
    pub fn from_env() -> Result<Self, AstToolError> {
        let raw = std::env::var("WORKSPACE_PATH")
            .ok()
            .or_else(|| {
                std::env::current_dir()
                    .ok()
                    .map(|p| p.to_string_lossy().into_owned())
            })
            .ok_or_else(|| {
                AstToolError::WorkspaceNotFound("WORKSPACE_PATH not set and CWD unavailable".into())
            })?;

        let path = Path::new(&raw);

        if !path.exists() {
            return Err(AstToolError::WorkspaceNotFound(format!(
                "path does not exist: {}",
                path.display()
            )));
        }
        if !path.is_dir() {
            return Err(AstToolError::WorkspaceNotFound(format!(
                "path is not a directory: {}",
                path.display()
            )));
        }

        let canonical = path.canonicalize().map_err(|e| {
            AstToolError::WorkspaceNotFound(format!(
                "cannot canonicalize workspace path {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(Workspace { root: canonical })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }
}
