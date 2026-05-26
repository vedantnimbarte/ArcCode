use arccode_config::PermissionMode;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ToolCtx {
    pub mode: PermissionMode,
    pub cwd: PathBuf,
    pub project_root: PathBuf,
}

impl ToolCtx {
    pub fn new(mode: PermissionMode, cwd: PathBuf, project_root: PathBuf) -> Self {
        Self {
            mode,
            cwd,
            project_root,
        }
    }

    /// Resolve a tool-supplied path against the cwd. Returns canonicalized
    /// form when possible, but accepts non-existent paths too (callers may
    /// be about to create them).
    pub fn resolve(&self, p: &str) -> PathBuf {
        let path = PathBuf::from(p);
        if path.is_absolute() {
            path
        } else {
            self.cwd.join(path)
        }
    }

    pub fn is_inside_project(&self, path: &Path) -> bool {
        let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let root =
            std::fs::canonicalize(&self.project_root).unwrap_or_else(|_| self.project_root.clone());
        path.starts_with(&root)
    }

    /// Permission for a write/edit operation on `path`.
    pub fn allows_write(&self, path: &Path) -> bool {
        match self.mode {
            PermissionMode::Yolo => true,
            PermissionMode::AutoEdit => self.is_inside_project(path),
            PermissionMode::ReadOnly => false,
        }
    }

    /// Permission for any shell execution.
    pub fn allows_shell(&self) -> bool {
        matches!(self.mode, PermissionMode::AutoEdit | PermissionMode::Yolo)
    }
}
