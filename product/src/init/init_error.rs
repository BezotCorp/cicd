use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::git::GitError;

#[derive(Debug)]
pub(crate) enum InitError {
    Git(GitError),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(error) => write!(f, "{error}"),
            Self::Io { path, source } => write!(f, "Cannot read {}: {source}", path.display()),
        }
    }
}

impl std::error::Error for InitError {}

impl From<GitError> for InitError {
    fn from(error: GitError) -> Self {
        Self::Git(error)
    }
}
