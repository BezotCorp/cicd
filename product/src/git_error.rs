use std::fmt;

#[derive(Debug)]
pub enum GitError {
    ExecutionFailed(String),
    CommandFailed(String),
    InvalidOutput(String),
    NotFound,
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::ExecutionFailed(msg) => write!(f, "Git binary execution failed: {}", msg),
            GitError::CommandFailed(msg) => write!(f, "Git command failed: {}", msg),
            GitError::InvalidOutput(msg) => write!(f, "Invalid Git output: {}", msg),
            GitError::NotFound => write!(f, "Git resource not found"),
        }
    }
}

impl std::error::Error for GitError {}
