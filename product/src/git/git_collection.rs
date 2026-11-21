use std::path::Path;

use crate::ci::CiProvider;
use crate::collect::CollectError;
use crate::git::{GitBranches, GitCommands, GitHosting, GitInfo, GitRemote, GitStatus};

#[derive(Debug)]
pub(crate) struct GitCollection {
    info: GitInfo,
    status: GitStatus,
    remotes: Vec<GitRemote>,
    branches: GitBranches,
    hosting: Option<GitHosting>,
    suggested_provider: Option<CiProvider>,
}

impl GitCollection {
    pub(crate) fn collect(path: &Path) -> Result<Self, CollectError> {
        let git = GitCommands::new(path);
        let info = git.inspect()?;
        let status = git.status()?;
        let remotes = git.remotes()?;
        let branches = git.branches()?;

        let hosting = Self::detect_hosting(&remotes);
        let suggested_provider = hosting.map(CiProvider::from_host);

        Ok(Self {
            info,
            status,
            remotes,
            branches,
            hosting,
            suggested_provider,
        })
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

    pub(crate) fn hosting(&self) -> Option<GitHosting> {
        self.hosting
    }

    pub(crate) fn suggested_provider(&self) -> Option<CiProvider> {
        self.suggested_provider
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        GitInfo,
        GitStatus,
        Vec<GitRemote>,
        GitBranches,
        Option<GitHosting>,
        Option<CiProvider>,
    ) {
        (
            self.info,
            self.status,
            self.remotes,
            self.branches,
            self.hosting,
            self.suggested_provider,
        )
    }

    /// Prefer origin, then any recognized remote.
    fn detect_hosting(remotes: &[GitRemote]) -> Option<GitHosting> {
        let from_remote = |remote: &GitRemote| {
            remote
                .fetch_urls()
                .iter()
                .chain(remote.push_urls())
                .find_map(|url| GitHosting::from_remote_url(url))
        };

        remotes
            .iter()
            .find(|remote| remote.name() == "origin")
            .and_then(from_remote)
            .or_else(|| remotes.iter().find_map(from_remote))
    }
}
