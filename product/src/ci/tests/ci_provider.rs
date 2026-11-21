use crate::ci::CiProvider;
use crate::git::GitHosting;

#[test]
fn suggests_github_actions() {
    assert_eq!(
        CiProvider::from_host(GitHosting::GitHub),
        CiProvider::GitHubActions
    );
}

#[test]
fn suggests_gitea_actions() {
    assert_eq!(
        CiProvider::from_host(GitHosting::Gitea),
        CiProvider::GiteaActions
    );
}

#[test]
fn suggests_gitlab_ci() {
    assert_eq!(
        CiProvider::from_host(GitHosting::GitLab),
        CiProvider::GitLabCi
    );
}

#[test]
fn gitea_has_its_own_workflow_location() {
    assert_eq!(
        CiProvider::GiteaActions.default_config_path(),
        ".gitea/workflows/ci.yml"
    );
}
