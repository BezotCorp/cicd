use crate::git::GitHosting;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CiProvider {
    GitHubActions,
    GiteaActions,
    GitLabCi,
    CircleCi,
    Jenkins,
    AzurePipelines,
    BitbucketPipelines,
    TravisCi,
    Drone,
}

impl CiProvider {
    /// Suggests native CI for a known Git hosting platform.
    /// This does not prove that the CI is configured.
    pub(crate) fn from_host(hosting: GitHosting) -> Self {
        match hosting {
            GitHosting::GitHub => Self::GitHubActions,
            GitHosting::GitLab => Self::GitLabCi,
            GitHosting::Gitea => Self::GiteaActions,
            GitHosting::Bitbucket => Self::BitbucketPipelines,
            GitHosting::AzureDevOps => Self::AzurePipelines,
        }
    }

    /// Conventional location of the pipeline definition for this provider.
    pub(crate) fn default_config_path(self) -> &'static str {
        match self {
            Self::GitHubActions => ".github/workflows/ci.yml",
            Self::GiteaActions => ".gitea/workflows/ci.yml",
            Self::GitLabCi => ".gitlab-ci.yml",
            Self::CircleCi => ".circleci/config.yml",
            Self::Jenkins => "Jenkinsfile",
            Self::AzurePipelines => "azure-pipelines.yml",
            Self::BitbucketPipelines => "bitbucket-pipelines.yml",
            Self::TravisCi => ".travis.yml",
            Self::Drone => ".drone.yml",
        }
    }
}

impl fmt::Display for CiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::GitHubActions => "GitHub Actions",
            Self::GiteaActions => "Gitea Actions",
            Self::GitLabCi => "GitLab CI",
            Self::CircleCi => "CircleCI",
            Self::Jenkins => "Jenkins",
            Self::AzurePipelines => "Azure Pipelines",
            Self::BitbucketPipelines => "Bitbucket Pipelines",
            Self::TravisCi => "Travis CI",
            Self::Drone => "Drone",
        })
    }
}
