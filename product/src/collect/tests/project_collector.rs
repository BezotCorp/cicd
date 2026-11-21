use std::error::Error;
use std::fs;
use std::process::Command;

use tempfile::tempdir;

use crate::ci::CiProvider;
use crate::collect::{CollectError, ProjectCollector};
use crate::git::GitHosting;
use common::data_source_kind::DataSourceKind;

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
    assert_eq!(
        snapshot.sources(),
        &[DataSourceKind::Git, DataSourceKind::Ci]
    );
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

#[test]
fn ci_collection_works_without_git() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();
    let workflows = root.join(".github/workflows");

    fs::create_dir_all(&workflows)?;
    fs::write(workflows.join("ci.yml"), "name: CI\n")?;

    let report = ProjectCollector::collect(root, &[DataSourceKind::Ci])?;

    assert!(report.git().is_none());
    assert_eq!(report.sources(), &[DataSourceKind::Ci]);

    let ci = report.ci().ok_or("CI collection absent")?;
    assert_eq!(ci.files().len(), 1);
    assert_eq!(ci.files()[0].provider(), CiProvider::GitHubActions);

    Ok(())
}

#[test]
fn git_collection_does_not_scan_ci() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let init = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .output()?;
    assert!(init.status.success());

    let report = ProjectCollector::collect(root, &[DataSourceKind::Git])?;

    assert!(report.git().is_some());
    assert!(report.ci().is_none());
    assert_eq!(report.sources(), &[DataSourceKind::Git]);

    Ok(())
}

#[test]
fn repeated_sources_are_collected_once() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let report = ProjectCollector::collect(root, &[DataSourceKind::Ci, DataSourceKind::Ci])?;

    assert_eq!(report.sources(), &[DataSourceKind::Ci]);
    assert!(report.git().is_none());
    assert!(report.ci().is_some());

    Ok(())
}

#[test]
fn requested_collection_order_is_preserved() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let init = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .output()?;
    assert!(init.status.success());

    let report = ProjectCollector::collect(root, &[DataSourceKind::Ci, DataSourceKind::Git])?;

    assert_eq!(report.sources(), &[DataSourceKind::Ci, DataSourceKind::Git]);
    assert!(report.git().is_some());
    assert!(report.ci().is_some());

    Ok(())
}

#[test]
fn empty_collection_request_is_rejected() -> TestResult {
    let directory = tempdir()?;

    let result = ProjectCollector::collect(directory.path(), &[]);

    assert!(matches!(result, Err(CollectError::InvalidProject(_))));

    Ok(())
}

#[test]
fn ci_collection_rejects_a_file_path() -> TestResult {
    let directory = tempdir()?;
    let file = directory.path().join("project.txt");

    fs::write(&file, "project")?;

    let result = ProjectCollector::collect(&file, &[DataSourceKind::Ci]);

    assert!(matches!(result, Err(CollectError::InvalidProject(_))));

    Ok(())
}

#[test]
fn full_analysis_preserves_existing_behavior() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let init = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-b", "main"])
        .output()?;
    assert!(init.status.success());

    let snapshot = ProjectCollector::analyze(root)?;

    assert_eq!(snapshot.info().branch(), Some("main"));
    assert_eq!(snapshot.sources(), &DataSourceKind::ALL);
    assert!(snapshot.ci_files().is_empty());

    Ok(())
}
