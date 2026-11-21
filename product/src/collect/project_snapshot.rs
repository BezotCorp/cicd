use crate::ci::{CiConfigFile, CiProvider};
use crate::diagnostic::Diagnostic;
use crate::git::{GitBranches, GitHosting, GitInfo, GitRemote, GitStatus};
use common::data_source_kind::DataSourceKind;

#[derive(Debug)]
pub(crate) struct ProjectSnapshot {
    info: GitInfo,
    status: GitStatus,
    remotes: Vec<GitRemote>,
    branches: GitBranches,
    ci_files: Vec<CiConfigFile>,
    hosting: Option<GitHosting>,
    suggested_provider: Option<CiProvider>,
    sources: Vec<DataSourceKind>,
    already_initialized: bool,
    findings: Vec<Diagnostic>,
}

impl ProjectSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        info: GitInfo,
        status: GitStatus,
        remotes: Vec<GitRemote>,
        branches: GitBranches,
        ci_files: Vec<CiConfigFile>,
        hosting: Option<GitHosting>,
        suggested_provider: Option<CiProvider>,
        sources: Vec<DataSourceKind>,
        already_initialized: bool,
        findings: Vec<Diagnostic>,
    ) -> Self {
        Self {
            info,
            status,
            remotes,
            branches,
            ci_files,
            hosting,
            suggested_provider,
            sources,
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

    pub(crate) fn hosting(&self) -> Option<GitHosting> {
        self.hosting
    }

    /// Suggested CI, not a confirmed CI configuration.
    pub(crate) fn suggested_provider(&self) -> Option<CiProvider> {
        self.suggested_provider
    }

    pub(crate) fn sources(&self) -> &[DataSourceKind] {
        &self.sources
    }

    /// Whether the cicd configuration file already exists.
    pub(crate) fn already_initialized(&self) -> bool {
        self.already_initialized
    }

    pub(crate) fn findings(&self) -> &[Diagnostic] {
        &self.findings
    }
}
