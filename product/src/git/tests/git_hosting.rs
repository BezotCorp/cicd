use crate::git::GitHosting;

#[test]
fn recognizes_known_hosts() {
    let cases = [
        ("git@github.com:owner/repo.git", GitHosting::GitHub),
        ("https://gitlab.com/a/b", GitHosting::GitLab),
        ("https://gitlab.example.org/a/b", GitHosting::GitLab),
        ("https://gitea.com/a/b", GitHosting::Gitea),
        ("https://bitbucket.org/a/b", GitHosting::Bitbucket),
        ("https://dev.azure.com/a/b", GitHosting::AzureDevOps),
    ];

    for (url, expected) in cases {
        assert_eq!(GitHosting::from_remote_url(url), Some(expected));
    }
}

#[test]
fn unknown_self_hosted_server_is_not_guessed() {
    assert_eq!(
        GitHosting::from_remote_url("https://git.example.org/owner/repo.git"),
        None
    );
}

#[test]
fn misleading_domains_are_rejected() {
    assert_eq!(
        GitHosting::from_remote_url("https://github.com.evil.example/a/b"),
        None
    );
    assert_eq!(
        GitHosting::from_remote_url("https://example.org/github.com/a"),
        None
    );
}
