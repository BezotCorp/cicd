#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GitHosting {
    GitHub,
    GitLab,
    Gitea,
    Bitbucket,
    AzureDevOps,
}

impl GitHosting {
    /// Recognizes known hosting domains, not arbitrary servers.
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
            "github.com" => Some(Self::GitHub),
            "gitlab.com" => Some(Self::GitLab),
            host if host.starts_with("gitlab.") => Some(Self::GitLab),
            "gitea.com" => Some(Self::Gitea),
            "bitbucket.org" => Some(Self::Bitbucket),
            "dev.azure.com" => Some(Self::AzureDevOps),
            host if host.ends_with(".visualstudio.com") => Some(Self::AzureDevOps),
            _ => None,
        }
    }
}
