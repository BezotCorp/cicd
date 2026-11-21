use std::path::{Path, PathBuf};

use crate::ci::{CiConfigFile, CiProvider};

#[test]
fn new_ci_config_file() {
    let provider = CiProvider::GitHubActions;
    let path = PathBuf::from(".github/workflows/ci.yml");

    let config_file = CiConfigFile::new(provider, path);

    assert_eq!(config_file.provider(), provider);
    assert_eq!(config_file.path(), Path::new(".github/workflows/ci.yml"));
}

#[test]
fn preserves_relative_path() {
    let config_file = CiConfigFile::new(
        CiProvider::GitHubActions,
        PathBuf::from("workflows/build.yml"),
    );

    assert_eq!(config_file.path(), Path::new("workflows/build.yml"));
    assert!(!config_file.path().is_absolute());
}

#[test]
fn preserves_absolute_path() {
    let path = std::env::current_dir()
        .expect("Cannot determine current directory")
        .join("ci.yml");

    let config_file = CiConfigFile::new(CiProvider::GitHubActions, path.clone());

    assert_eq!(config_file.path(), path);
    assert!(config_file.path().is_absolute());
}

#[test]
fn supports_paths_with_spaces() {
    let path = PathBuf::from("ci configurations/build pipeline.yml");

    let config_file = CiConfigFile::new(CiProvider::GitHubActions, path.clone());

    assert_eq!(config_file.path(), path);
}

#[test]
fn supports_unicode_paths() {
    let path = PathBuf::from("intégration/déploiement.yml");

    let config_file = CiConfigFile::new(CiProvider::GitHubActions, path.clone());

    assert_eq!(config_file.path(), path);
}

#[test]
fn preserves_provider_for_different_ci_systems() {
    let cases = [
        (CiProvider::GitHubActions, ".github/workflows/ci.yml"),
        (CiProvider::GitLabCi, ".gitlab-ci.yml"),
        (CiProvider::Jenkins, "Jenkinsfile"),
        (CiProvider::CircleCi, ".circleci/config.yml"),
        (CiProvider::AzurePipelines, "azure-pipelines.yml"),
        (CiProvider::BitbucketPipelines, "bitbucket-pipelines.yml"),
        (CiProvider::TravisCi, ".travis.yml"),
        (CiProvider::Drone, ".drone.yml"),
    ];

    for (provider, path) in cases {
        let config_file = CiConfigFile::new(provider, PathBuf::from(path));

        assert_eq!(config_file.provider(), provider);
        assert_eq!(config_file.path(), Path::new(path));
    }
}

#[test]
fn cloned_config_preserves_values() {
    let original = CiConfigFile::new(
        CiProvider::GitHubActions,
        PathBuf::from(".github/workflows/release.yml"),
    );

    let cloned = original.clone();

    assert_eq!(original, cloned);
}
