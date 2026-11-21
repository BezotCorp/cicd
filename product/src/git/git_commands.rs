use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::git::{GitBranches, GitError, GitInfo, GitRemote, GitStatus};

pub(crate) struct GitCommands {
    path: PathBuf,
}

impl GitCommands {
    pub(crate) fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the containing repository root, if any.
    pub(crate) fn repository_root(&self) -> Result<Option<PathBuf>, GitError> {
        let output = self.execute(&["rev-parse", "--show-toplevel"])?;

        if output.status.success() {
            return Self::text(output).map(|root| Some(PathBuf::from(root)));
        }

        if self.path.join(".git").exists() {
            return Err(GitError::CommandFailed(
                "Existing Git metadata cannot be inspected".to_owned(),
            ));
        }

        Ok(None)
    }

    pub(crate) fn initialize_main(&self) -> Result<(), GitError> {
        self.required(&["init", "-b", "main"])?;
        Ok(())
    }

    pub(crate) fn create_branch(&self, name: &str, start_point: &str) -> Result<(), GitError> {
        self.required(&["branch", name, start_point])?;
        Ok(())
    }

    fn execute(&self, args: &[&str]) -> Result<Output, GitError> {
        Command::new("git")
            .arg("-C")
            .arg(&self.path)
            .args(args)
            .output()
            .map_err(|error| GitError::ExecutionFailed(format!("Cannot execute git: {error}")))
    }

    fn text(output: Output) -> Result<String, GitError> {
        String::from_utf8(output.stdout)
            .map(|value| value.trim_end_matches(['\r', '\n']).to_owned())
            .map_err(|error| GitError::InvalidOutput(format!("Invalid Git output: {error}")))
    }

    fn required(&self, args: &[&str]) -> Result<String, GitError> {
        let output = self.execute(args)?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(format!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        Self::text(output)
    }

    fn branch(&self) -> Result<Option<String>, GitError> {
        let output = self.execute(&["symbolic-ref", "--quiet", "--short", "HEAD"])?;

        if output.status.success() {
            return Self::text(output).map(Some);
        }

        if output.status.code() == Some(1) {
            return Ok(None);
        }

        Err(GitError::CommandFailed(format!(
            "Cannot determine Git branch: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }

    fn head(&self, has_branch: bool) -> Result<Option<String>, GitError> {
        let output = self.execute(&["rev-parse", "--verify", "--quiet", "HEAD"])?;

        if output.status.success() {
            return Self::text(output).map(Some);
        }

        if has_branch && output.status.code() == Some(1) {
            return Ok(None);
        }

        Err(GitError::CommandFailed(format!(
            "Cannot resolve Git HEAD: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }

    pub(crate) fn inspect(&self) -> Result<GitInfo, GitError> {
        let root = PathBuf::from(self.required(&["rev-parse", "--show-toplevel"])?);

        let branch = self.branch()?;
        let head = self.head(branch.is_some())?;

        Ok(GitInfo::new(root, branch, head))
    }

    pub(crate) fn status(&self) -> Result<GitStatus, GitError> {
        let output = self.execute(&["status", "--porcelain=v1", "-z", "--untracked-files=all"])?;

        if !output.status.success() {
            return Err(GitError::CommandFailed(format!(
                "git status failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        let mut staged = Vec::new();
        let mut unstaged = Vec::new();
        let mut untracked = Vec::new();
        let mut deleted = Vec::new();
        let mut conflicted = Vec::new();

        if !output.stdout.is_empty() && output.stdout.last() != Some(&0) {
            return Err(GitError::InvalidOutput(
                "Git status output is not NUL-terminated".into(),
            ));
        }

        let mut entries = output.stdout.split(|byte| *byte == 0).peekable();

        while let Some(entry) = entries.next() {
            if entry.is_empty() {
                if entries.peek().is_none() {
                    break;
                }

                return Err(GitError::InvalidOutput(
                    "Unexpected empty Git status entry".into(),
                ));
            }

            if entry.len() < 4 || entry[2] != b' ' {
                return Err(GitError::InvalidOutput("Malformed Git status entry".into()));
            }

            let x = entry[0];
            let y = entry[1];
            let path_bytes = &entry[3..];

            if path_bytes.is_empty() {
                return Err(GitError::InvalidOutput("Empty path in Git status".into()));
            }

            let valid_status =
                |value: u8| matches!(value, b' ' | b'M' | b'A' | b'D' | b'R' | b'C' | b'U' | b'T');

            if (x, y) != (b'?', b'?')
                && (x, y) != (b'!', b'!')
                && (!valid_status(x) || !valid_status(y))
            {
                return Err(GitError::InvalidOutput("Unknown Git status code".into()));
            }

            #[cfg(unix)]
            let path = {
                use std::os::unix::ffi::OsStringExt;

                PathBuf::from(std::ffi::OsString::from_vec(path_bytes.to_vec()))
            };

            #[cfg(not(unix))]
            let path = PathBuf::from(String::from_utf8(path_bytes.to_vec()).map_err(|error| {
                GitError::InvalidOutput(format!("Invalid path encoding: {error}"))
            })?);

            if (x, y) == (b'?', b'?') {
                untracked.push(path);
                continue;
            }

            if (x, y) == (b'!', b'!') {
                continue;
            }

            if matches!(
                (x, y),
                (b'D', b'D')
                    | (b'A', b'U')
                    | (b'U', b'D')
                    | (b'U', b'A')
                    | (b'D', b'U')
                    | (b'A', b'A')
                    | (b'U', b'U')
            ) {
                conflicted.push(path.clone());
            }

            if x != b' ' {
                staged.push(path.clone());
            }

            if y != b' ' {
                unstaged.push(path.clone());
            }

            if x == b'D' || y == b'D' {
                deleted.push(path);
            }

            // Porcelain v1 -z emits an additional source path
            // after the destination path for renames and copies.
            if matches!(x, b'R' | b'C') || matches!(y, b'R' | b'C') {
                let source = entries.next().ok_or_else(|| {
                    GitError::InvalidOutput("Missing rename/copy source path".into())
                })?;

                if source.is_empty() {
                    return Err(GitError::InvalidOutput(
                        "Empty rename/copy source path".into(),
                    ));
                }
            }
        }

        Ok(GitStatus::new(
            staged, unstaged, untracked, deleted, conflicted,
        ))
    }

    pub(crate) fn remotes(&self) -> Result<Vec<GitRemote>, GitError> {
        let output = self.required(&["remote", "-v"])?;
        // name -> (all fetch URLs, all push URLs), preserving order.
        let mut entries: Vec<(String, Vec<String>, Vec<String>)> = Vec::new();

        for line in output.lines().filter(|line| !line.is_empty()) {
            let malformed = || GitError::InvalidOutput(format!("Malformed remote line: {line}"));
            let (name, rest) = line.split_once('\t').ok_or_else(malformed)?;
            let (url, kind) = rest.rsplit_once(' ').ok_or_else(malformed)?;

            let position = match entries.iter().position(|entry| entry.0 == name) {
                Some(position) => position,
                None => {
                    entries.push((name.to_owned(), Vec::new(), Vec::new()));
                    entries.len() - 1
                }
            };
            let entry = &mut entries[position];

            match kind {
                "(fetch)" => entry.1.push(url.to_owned()),
                "(push)" => entry.2.push(url.to_owned()),
                _ => return Err(malformed()),
            }
        }

        Ok(entries
            .into_iter()
            .map(|(name, fetch, push)| GitRemote::new(name, fetch, push))
            .collect())
    }

    pub(crate) fn branches(&self) -> Result<GitBranches, GitError> {
        let output = self.required(&[
            "for-each-ref",
            "--format=%(refname)",
            "refs/heads",
            "refs/remotes",
        ])?;
        let mut local = Vec::new();
        let mut remote = Vec::new();

        for reference in output.lines() {
            if let Some(name) = reference.strip_prefix("refs/heads/") {
                local.push(name.to_owned());
            } else if let Some(name) = reference.strip_prefix("refs/remotes/")
                && !name.ends_with("/HEAD")
            {
                remote.push(name.to_owned());
            }
        }

        Ok(GitBranches::new(local, remote))
    }
}
