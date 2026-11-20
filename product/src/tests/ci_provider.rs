use crate::ci_provider::CiProvider;

#[cfg(test)]
#[test]
fn infers_provider_from_remote_url() {
    assert_eq!(
        CiProvider::from_remote_url("git@github.com:a/b.git"),
        Some(CiProvider::GitHubActions)
    );
    assert_eq!(
        CiProvider::from_remote_url("https://gitlab.example.org/a/b"),
        Some(CiProvider::GitLabCi)
    );
    assert_eq!(CiProvider::from_remote_url("ssh://host/a/b"), None);
}

#[test]
fn refuses_misleading_hostnames() {
    assert_eq!(CiProvider::from_remote_url("https://github.com.evil.test/a/b"), None);
    assert_eq!(CiProvider::from_remote_url("https://evil.test/github.com/a"), None);
    assert_eq!(CiProvider::from_remote_url("git@github.com:a/b.git"), Some(CiProvider::GitHubActions));
}
