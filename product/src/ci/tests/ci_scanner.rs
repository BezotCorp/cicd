use std::error::Error;
use std::fs;
use std::path::Path;

use tempfile::tempdir;

use crate::ci::{CiProvider, CiScanner};
use crate::collect::CollectError;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn empty_repository_returns_no_configs() -> TestResult {
    let directory = tempdir()?;

    let found = CiScanner::scan(directory.path())?;

    assert!(found.is_empty());
    Ok(())
}

#[test]
fn missing_repository_returns_no_configs() -> TestResult {
    let directory = tempdir()?;
    let missing = directory.path().join("does-not-exist");

    let found = CiScanner::scan(&missing)?;

    assert!(found.is_empty());
    Ok(())
}

#[test]
fn discovers_all_supported_single_file_providers() -> TestResult {
    let directory = tempdir()?;

    let cases = [
        (".gitlab-ci.yml", CiProvider::GitLabCi),
        (".circleci/config.yml", CiProvider::CircleCi),
        ("Jenkinsfile", CiProvider::Jenkins),
        ("azure-pipelines.yml", CiProvider::AzurePipelines),
        ("bitbucket-pipelines.yml", CiProvider::BitbucketPipelines),
        (".travis.yml", CiProvider::TravisCi),
        (".drone.yml", CiProvider::Drone),
    ];

    for (path, _) in cases {
        let target = directory.path().join(path);

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(target, "test")?;
    }

    let found = CiScanner::scan(directory.path())?;

    assert_eq!(found.len(), cases.len());

    for (path, provider) in cases {
        assert!(
            found.iter().any(|config| {
                config.provider() == provider && config.path() == Path::new(path)
            })
        );
    }

    Ok(())
}

#[test]
fn discovers_multiple_github_workflows() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;
    fs::write(workflows.join("ci.yml"), "")?;
    fs::write(workflows.join("release.yaml"), "")?;
    fs::write(workflows.join("windows.yml"), "")?;

    let found = CiScanner::scan(directory.path())?;

    assert_eq!(found.len(), 3);
    assert!(
        found
            .iter()
            .all(|config| { config.provider() == CiProvider::GitHubActions })
    );

    Ok(())
}

#[test]
fn ignores_unrelated_files() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;

    for name in [
        "README.md",
        "config.json",
        "ci.yml.bak",
        "ci.yml.txt",
        "ci.YML",
        "release.YAML",
        "notes",
    ] {
        fs::write(workflows.join(name), "test")?;
    }

    fs::write(workflows.join("valid.yml"), "")?;

    let found = CiScanner::scan(directory.path())?;

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path(), Path::new(".github/workflows/valid.yml"));

    Ok(())
}

#[test]
fn ignores_directories_named_like_workflows() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(workflows.join("fake.yml"))?;
    fs::create_dir_all(workflows.join("fake.yaml"))?;
    fs::write(workflows.join("real.yml"), "")?;

    let found = CiScanner::scan(directory.path())?;

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path(), Path::new(".github/workflows/real.yml"));

    Ok(())
}

#[test]
fn ignores_nested_github_workflows() -> TestResult {
    let directory = tempdir()?;
    let nested = directory.path().join(".github/workflows/nested");

    fs::create_dir_all(&nested)?;
    fs::write(nested.join("hidden.yml"), "")?;

    let found = CiScanner::scan(directory.path())?;

    assert!(found.is_empty());
    Ok(())
}

#[test]
fn preserves_unicode_spaces_and_special_names() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;

    for name in [
        "déploiement été.yml",
        "日本語.yaml",
        "build [linux] #1.yml",
        "pipeline;test.yml",
    ] {
        fs::write(workflows.join(name), "")?;
    }

    let found = CiScanner::scan(directory.path())?;

    assert_eq!(found.len(), 4);

    for name in [
        "déploiement été.yml",
        "日本語.yaml",
        "build [linux] #1.yml",
        "pipeline;test.yml",
    ] {
        assert!(
            found
                .iter()
                .any(|config| { config.path() == Path::new(".github/workflows").join(name) })
        );
    }

    Ok(())
}

#[test]
fn results_are_sorted_by_path() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;

    fs::write(workflows.join("z.yml"), "")?;
    fs::write(workflows.join("a.yml"), "")?;
    fs::write(directory.path().join("Jenkinsfile"), "")?;
    fs::write(directory.path().join(".travis.yml"), "")?;

    let found = CiScanner::scan(directory.path())?;

    let paths: Vec<_> = found.iter().map(|config| config.path()).collect();

    let mut expected = paths.clone();
    expected.sort();

    assert_eq!(paths, expected);
    assert_eq!(found.len(), 4);

    Ok(())
}

#[test]
fn workflow_path_is_a_file_instead_of_directory() -> TestResult {
    let directory = tempdir()?;
    let github = directory.path().join(".github");

    fs::create_dir_all(&github)?;
    fs::write(github.join("workflows"), "not a directory")?;

    let result = CiScanner::scan(directory.path());

    match result {
        Err(CollectError::Io { path, .. }) => {
            assert_eq!(path, github.join("workflows"));
        }
        other => {
            assert!(false, "Expected an I/O error, got {other:?}");
        }
    }

    Ok(())
}

#[test]
fn ignores_existing_provider_directory_without_config_file() -> TestResult {
    let directory = tempdir()?;

    fs::create_dir_all(directory.path().join(".circleci"))?;

    let found = CiScanner::scan(directory.path())?;

    assert!(found.is_empty());
    Ok(())
}

#[test]
fn does_not_validate_yaml_contents() -> TestResult {
    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;

    fs::write(workflows.join("broken.yml"), "this is: [not valid yaml")?;

    let found = CiScanner::scan(directory.path())?;

    // Current contract: discovery only, not YAML validation.
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path(), Path::new(".github/workflows/broken.yml"));

    Ok(())
}

#[cfg(unix)]
#[test]
fn follows_symlink_to_workflow_file() -> TestResult {
    use std::os::unix::fs::symlink;

    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");
    let target = directory.path().join("outside.yml");

    fs::create_dir_all(&workflows)?;
    fs::write(&target, "test")?;

    symlink(&target, workflows.join("linked.yml"))?;

    let found = CiScanner::scan(directory.path())?;

    // Current behavior: Path::is_file follows symbolic links.
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path(), Path::new(".github/workflows/linked.yml"));

    Ok(())
}

#[cfg(unix)]
#[test]
fn ignores_broken_symlink() -> TestResult {
    use std::os::unix::fs::symlink;

    let directory = tempdir()?;
    let workflows = directory.path().join(".github/workflows");

    fs::create_dir_all(&workflows)?;

    symlink(
        directory.path().join("missing.yml"),
        workflows.join("broken.yml"),
    )?;

    let found = CiScanner::scan(directory.path())?;

    assert!(found.is_empty());
    Ok(())
}
