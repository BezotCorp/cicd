use std::path::Path;

use crate::ci::{CiProvider, CiScanner};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_severity::DiagnosticSeverity;
use crate::git::{GitCommands, GitRemote};
use crate::init::{InitDiagnosis, InitError};
use crate::paths::CONFIG_PATH;

pub(crate) struct InitAnalyzer;

impl InitAnalyzer {
    /// Read-only analysis of the repository containing `path`.
    pub(crate) fn analyze(path: &Path) -> Result<InitDiagnosis, InitError> {
        let git = GitCommands::new(path);
        let info = git.inspect()?;
        let status = git.status()?;
        let remotes = git.remotes()?;
        let branches = git.branches()?;
        let ci_files = CiScanner::scan(info.root())?;
        let already_initialized = info.root().join(CONFIG_PATH).is_file();
        let target_provider = Self::target_provider(&remotes);

        let mut findings = Vec::new();
        let mut warn = |message: &str| {
            findings.push(Diagnostic::new(
                DiagnosticSeverity::Warning,
                message.to_owned(),
            ));
        };

        if info.branch().is_none() {
            warn("HEAD is detached: no current branch");
        }
        if info.head().is_none() {
            warn("Repository has no commit yet");
        }
        if remotes.is_empty() {
            warn("No remote configured: the CI provider cannot be inferred");
        } else if target_provider.is_none() {
            warn("No remote points to a supported CI hosting platform");
        }
        if !status.staged().is_empty() || !status.unstaged().is_empty() {
            warn("Working tree has uncommitted changes to tracked files");
        }

        if !status.conflicted().is_empty() {
            warn("Repository contains unresolved merge conflicts");
        }

        let mut info_msgs = Vec::new();
        if already_initialized {
            info_msgs.push(format!("{CONFIG_PATH} already exists"));
        }
        if ci_files.is_empty() {
            info_msgs.push("No existing CI/CD configuration found".to_owned());
        }
        findings.extend(
            info_msgs
                .into_iter()
                .map(|message| Diagnostic::new(DiagnosticSeverity::Info, message)),
        );

        Ok(InitDiagnosis::new(
            info,
            status,
            remotes,
            branches,
            ci_files,
            target_provider,
            already_initialized,
            findings,
        ))
    }

    /// Prefers `origin`, then falls back to the first remote that matches a provider.
    fn target_provider(remotes: &[GitRemote]) -> Option<CiProvider> {
        let from_remote = |remote: &GitRemote| {
            remote
                .fetch_urls()
                .iter()
                .chain(remote.push_urls())
                .find_map(|url| CiProvider::from_remote_url(url))
        };

        remotes
            .iter()
            .find(|remote| remote.name() == "origin")
            .and_then(from_remote)
            .or_else(|| remotes.iter().find_map(from_remote))
    }
}
