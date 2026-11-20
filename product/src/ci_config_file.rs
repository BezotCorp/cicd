use std::path::{Path, PathBuf};

use crate::ci_provider::CiProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CiConfigFile {
    provider: CiProvider,
    path: PathBuf,
}

impl CiConfigFile {
    pub(crate) fn new(provider: CiProvider, path: PathBuf) -> Self {
        Self { provider, path }
    }

    pub(crate) fn provider(&self) -> CiProvider {
        self.provider
    }

    /// Path relative to the repository root.
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}
