use std::error::Error;
use std::fs;
use std::process::Command;

use tempfile::tempdir;

use crate::ci::CiProvider;
use crate::collect::{CollectSource, ProjectCollector};
use crate::git::GitHosting;

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn collects_git_and_ci_from_existing_repository() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let init = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .output()?;
    assert!(init.status.success());

    let remote = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "remote",
            "add",
            "origin",
            "https://github.com/example/project.git",
        ])
        .output()?;
    assert!(remote.status.success());

    let workflows = root.join(".github/workflows");
    fs::create_dir_all(&workflows)?;
    fs::write(workflows.join("ci.yml"), "name: CI\n")?;

    let snapshot = ProjectCollector::analyze(root)?;

    assert_eq!(snapshot.info().branch(), Some("main"));
    assert_eq!(snapshot.hosting(), Some(GitHosting::GitHub));
    assert_eq!(
        snapshot.suggested_provider(),
        Some(CiProvider::GitHubActions)
    );
    assert_eq!(snapshot.sources(), &[CollectSource::Git, CollectSource::Ci]);
    assert_eq!(snapshot.ci_files().len(), 1);
    assert_eq!(snapshot.remotes().len(), 1);

    Ok(())
}

#[test]
fn collects_existing_repository_without_remote() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let init = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .output()?;
    assert!(init.status.success());

    let snapshot = ProjectCollector::analyze(root)?;

    assert!(snapshot.remotes().is_empty());
    assert!(snapshot.ci_files().is_empty());
    assert_eq!(snapshot.hosting(), None);
    assert_eq!(snapshot.suggested_provider(), None);
    assert!(!snapshot.findings().is_empty());

    Ok(())
}

#[test]
fn collection_error_does_not_require_init() -> TestResult {
    use crate::collect::CollectError;

    let directory = tempdir()?;

    let result = ProjectCollector::analyze(directory.path());

    assert!(matches!(result, Err(CollectError::Git(_))));

    Ok(())
}
