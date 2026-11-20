use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GitInfo {
    root: PathBuf,
    branch: Option<String>,
    head: Option<String>,
}

impl GitInfo {
    pub(crate) fn new(root: PathBuf, branch: Option<String>, head: Option<String>) -> Self {
        Self { root, branch, head }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn branch(&self) -> Option<&str> {
        self.branch.as_deref()
    }

    pub(crate) fn head(&self) -> Option<&str> {
        self.head.as_deref()
    }
}
