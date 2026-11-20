use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitStatus {
    staged: Vec<PathBuf>,
    unstaged: Vec<PathBuf>,
    untracked: Vec<PathBuf>,
    deleted: Vec<PathBuf>,
    conflicted: Vec<PathBuf>,
}

impl GitStatus {
    pub(crate) fn new(
        staged: Vec<PathBuf>,
        unstaged: Vec<PathBuf>,
        untracked: Vec<PathBuf>,
        deleted: Vec<PathBuf>,
        conflicted: Vec<PathBuf>,
    ) -> Self {
        Self {
            staged,
            unstaged,
            untracked,
            deleted,
            conflicted,
        }
    }

    pub(crate) fn staged(&self) -> &Vec<PathBuf> {
        &self.staged
    }

    pub(crate) fn unstaged(&self) -> &Vec<PathBuf> {
        &self.unstaged
    }

    pub(crate) fn untracked(&self) -> &Vec<PathBuf> {
        &self.untracked
    }

    pub(crate) fn deleted(&self) -> &[PathBuf] {
        &self.deleted
    }

    pub(crate) fn conflicted(&self) -> &[PathBuf] {
        &self.conflicted
    }
}
