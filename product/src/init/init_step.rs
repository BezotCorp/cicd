use std::path::{Path, PathBuf};

use crate::init::InitAction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitStep {
    action: InitAction,
    path: PathBuf,
    reason: String,
}

impl InitStep {
    pub(crate) fn new(action: InitAction, path: PathBuf, reason: String) -> Self {
        Self {
            action,
            path,
            reason,
        }
    }

    pub(crate) fn action(&self) -> InitAction {
        self.action
    }

    /// Path relative to the repository root.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}
