use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CiProvider {
    GitHubActions,
    GitLabCi,
    CircleCi,
    Jenkins,
    AzurePipelines,
    BitbucketPipelines,
    TravisCi,
    Drone,
}

impl CiProvider {
    /// Guesses the hosting platform (and thus its native CI) from a remote URL.
    pub(crate) fn from_remote_url(url: &str) -> Option<Self> {
        let authority = if let Some((_, rest)) = url.split_once("://") {
            rest.split('/').next()?
        } else if let Some((host, _)) = url.split_once(':') {
            host
        } else {
            return None;
        };

        let host = authority
            .rsplit('@')
            .next()?
            .split(':')
            .next()?
            .to_ascii_lowercase();

        match host.as_str() {
            "github.com" => Some(Self::GitHubActions),
            "gitlab.com" => Some(Self::GitLabCi),
            host if host.starts_with("gitlab.") => Some(Self::GitLabCi),
            "bitbucket.org" => Some(Self::BitbucketPipelines),
            "dev.azure.com" => Some(Self::AzurePipelines),
            host if host.ends_with(".visualstudio.com") => Some(Self::AzurePipelines),
            _ => None,
        }
    }

    /// Conventional location of the pipeline definition for this provider.
    pub(crate) fn default_config_path(self) -> &'static str {
        match self {
            Self::GitHubActions => ".github/workflows/ci.yml",
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
