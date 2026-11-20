use crate::ci::{CiConfigFile, CiProvider};
use crate::diagnostic::Diagnostic;
use crate::git::{GitBranches, GitInfo, GitRemote, GitStatus};

#[derive(Debug)]
pub(crate) struct InitDiagnosis {
    info: GitInfo,
    status: GitStatus,
    remotes: Vec<GitRemote>,
    branches: GitBranches,
    ci_files: Vec<CiConfigFile>,
    target_provider: Option<CiProvider>,
    already_initialized: bool,
    findings: Vec<Diagnostic>,
}

impl InitDiagnosis {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        info: GitInfo,
        status: GitStatus,
        remotes: Vec<GitRemote>,
        branches: GitBranches,
        ci_files: Vec<CiConfigFile>,
        target_provider: Option<CiProvider>,
        already_initialized: bool,
        findings: Vec<Diagnostic>,
    ) -> Self {
        Self {
            info,
            status,
            remotes,
            branches,
            ci_files,
            target_provider,
            already_initialized,
            findings,
        }
    }

    pub(crate) fn info(&self) -> &GitInfo {
        &self.info
    }

    pub(crate) fn status(&self) -> &GitStatus {
        &self.status
    }

    pub(crate) fn remotes(&self) -> &[GitRemote] {
        &self.remotes
    }

    pub(crate) fn branches(&self) -> &GitBranches {
        &self.branches
    }

    pub(crate) fn ci_files(&self) -> &[CiConfigFile] {
        &self.ci_files
    }

    /// CI provider inferred from the remotes, if any.
    pub(crate) fn target_provider(&self) -> Option<CiProvider> {
        self.target_provider
    }

    /// Whether the cicd configuration file already exists.
    pub(crate) fn already_initialized(&self) -> bool {
        self.already_initialized
    }

    pub(crate) fn findings(&self) -> &[Diagnostic] {
        &self.findings
    }
}
