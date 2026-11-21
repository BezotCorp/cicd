use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::git::GitError;

#[derive(Debug)]
pub(crate) enum CollectError {
    Git(GitError),
    InvalidProject(String),
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Git(error) => write!(f, "{error}"),
            Self::InvalidProject(message) => f.write_str(message),
            Self::Io { path, source } => {
                write!(f, "Cannot read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CollectError {}

impl From<GitError> for CollectError {
    fn from(error: GitError) -> Self {
        Self::Git(error)
    }
}
