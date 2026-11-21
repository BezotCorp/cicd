use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use tempfile::tempdir;

use crate::git::{GitCommands, GitWorkflow};
use crate::init::{InitAction, InitGit};

type TestResult = Result<(), Box<dyn Error>>;

fn run_git(root: &Path, args: &[&str]) -> TestResult {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .status()?;

    if !status.success() {
        return Err(format!("Git failed: {args:?}").into());
    }

    Ok(())
}

fn commit_empty(root: &Path) -> TestResult {
    run_git(
        root,
        &[
            "-c",
            "user.name=CI Test",
            "-c",
            "user.email=ci-test@example.invalid",
            "commit",
            "--allow-empty",
            "-m",
            "initial",
        ],
    )
}

#[test]
fn new_project_dry_run_does_not_create_git() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let plan = InitGit::plan(root, None)?;

    assert!(
        plan.steps()
            .iter()
            .any(|step| { step.action() == InitAction::InitializeGit })
    );
    assert!(!root.join(".git").exists());

    Ok(())
}

#[test]
fn apply_initializes_main_without_commit() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();
    fs::write(root.join("existing.txt"), "preserved")?;

    let plan = InitGit::plan(root, None)?;
    InitGit::apply(root, &plan, None)?;

    let git = GitCommands::new(root);
    let info = git.inspect()?;

    assert_eq!(info.branch(), Some("main"));
    assert_eq!(info.head(), None);
    assert!(root.join(".git").exists());
    assert!(!git.branches()?.local().iter().any(|name| name == "dev"));
    assert_eq!(fs::read_to_string(root.join("existing.txt"))?, "preserved");

    Ok(())
}

#[test]
fn existing_main_creates_dev_after_first_commit() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;

    let head_before = git.inspect()?;

    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;
    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;

    let head_after = git.inspect()?;
    let branches = git.branches()?;

    assert_eq!(head_after.branch(), Some("main"));
    assert_eq!(head_before.head(), head_after.head());
    assert!(branches.local().iter().any(|name| name == "main"));
    assert!(branches.local().iter().any(|name| name == "dev"));

    Ok(())
}

#[test]
fn keeps_existing_main_and_dev() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;
    git.create_branch("dev", "main")?;

    let before = git.inspect()?;
    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;

    assert!(!plan.has_changes());

    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;

    let after = git.inspect()?;

    assert_eq!(before.head(), after.head());
    assert_eq!(before.branch(), after.branch());

    Ok(())
}

#[test]
fn existing_feature_branch_is_not_switched() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;

    run_git(root, &["checkout", "-b", "feature/test"])?;

    let before = git.inspect()?;
    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;
    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;
    let after = git.inspect()?;

    assert_eq!(before.branch(), Some("feature/test"));
    assert_eq!(after.branch(), Some("feature/test"));
    assert_eq!(before.head(), after.head());

    Ok(())
}

#[test]
fn defers_main_when_base_is_ambiguous() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    run_git(root, &["init", "-b", "legacy"])?;
    commit_empty(root)?;

    let git = GitCommands::new(root);
    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;

    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;

    let info = git.inspect()?;
    let branches = git.branches()?;

    assert_eq!(info.branch(), Some("legacy"));
    assert!(!branches.local().iter().any(|name| name == "main"));
    assert!(!branches.local().iter().any(|name| name == "dev"));

    assert!(plan.steps().iter().any(|step| {
        step.action() == InitAction::Deferred && step.path() == Path::new("refs/heads/main")
    }));

    Ok(())
}

#[test]
fn empty_existing_branch_does_not_create_commits() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    run_git(root, &["init", "-b", "legacy"])?;

    let git = GitCommands::new(root);
    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;

    assert!(
        plan.steps()
            .iter()
            .any(|step| { step.action() == InitAction::Deferred })
    );

    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;

    let info = git.inspect()?;

    assert_eq!(info.branch(), Some("legacy"));
    assert_eq!(info.head(), None);

    Ok(())
}

#[test]
fn refuses_to_initialize_nested_repository() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;

    let nested = root.join("nested");
    fs::create_dir_all(&nested)?;

    assert!(InitGit::plan(&nested, None).is_err());
    assert!(!nested.join(".git").exists());

    Ok(())
}

#[test]
fn refuses_nonexistent_project() -> TestResult {
    let directory = tempdir()?;
    let missing = directory.path().join("missing");

    assert!(InitGit::plan(&missing, None).is_err());
    assert!(!missing.exists());

    Ok(())
}

#[test]
fn refuses_file_instead_of_directory() -> TestResult {
    let directory = tempdir()?;
    let file = directory.path().join("file.txt");

    fs::write(&file, "not a directory")?;

    assert!(InitGit::plan(&file, None).is_err());

    Ok(())
}

#[test]
fn rejects_outdated_initialization_plan() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let old_plan = InitGit::plan(root, None)?;

    let git = GitCommands::new(root);
    git.initialize_main()?;

    assert!(InitGit::apply(root, &old_plan, None).is_err());

    Ok(())
}

#[test]
fn rejects_unresolved_merge_conflicts() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;

    fs::write(root.join("conflict.txt"), "base\n")?;
    run_git(root, &["add", "conflict.txt"])?;
    commit_empty(root)?;

    run_git(root, &["checkout", "-b", "other"])?;
    fs::write(root.join("conflict.txt"), "other\n")?;
    run_git(root, &["add", "conflict.txt"])?;
    commit_empty(root)?;

    run_git(root, &["checkout", "main"])?;
    fs::write(root.join("conflict.txt"), "main\n")?;
    run_git(root, &["add", "conflict.txt"])?;
    commit_empty(root)?;

    let merge = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["merge", "other"])
        .status()?;

    assert!(!merge.success());

    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;

    assert!(InitGit::apply(root, &plan, Some(GitWorkflow::Pro)).is_err());

    Ok(())
}

#[test]
fn existing_main_without_profile_does_not_create_dev() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;

    let before = git.branches()?;
    let plan = InitGit::plan(root, None)?;

    assert!(!plan.has_changes());

    InitGit::apply(root, &plan, None)?;

    assert_eq!(git.branches()?, before);
    assert!(!git.branches()?.local().iter().any(|name| name == "dev"));

    Ok(())
}

#[test]
fn simple_profile_preserves_existing_branches() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;

    let before = git.branches()?;
    let plan = InitGit::plan(root, Some(GitWorkflow::Simple))?;

    assert!(!plan.has_changes());

    InitGit::apply(root, &plan, Some(GitWorkflow::Simple))?;

    assert_eq!(git.branches()?, before);

    Ok(())
}

#[test]
fn pro_profile_creates_missing_dev_from_main() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    let git = GitCommands::new(root);
    git.initialize_main()?;
    commit_empty(root)?;

    let main_head = git.inspect()?;
    let plan = InitGit::plan(root, Some(GitWorkflow::Pro))?;

    assert!(plan.has_changes());

    InitGit::apply(root, &plan, Some(GitWorkflow::Pro))?;

    let branches = git.branches()?;

    assert!(branches.local().iter().any(|name| name == "main"));
    assert!(branches.local().iter().any(|name| name == "dev"));
    assert_eq!(git.inspect()?.head(), main_head.head());

    Ok(())
}

#[test]
fn bare_repository_is_not_treated_as_missing() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    run_git(root, &["init", "--bare"])?;

    let git = GitCommands::new(root);

    assert!(git.repository_root().is_err());
    assert!(InitGit::plan(root, None).is_err());

    Ok(())
}

#[test]
fn malformed_git_metadata_is_not_treated_as_missing() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    fs::write(root.join(".git"), "invalid git metadata")?;

    let git = GitCommands::new(root);

    assert!(git.repository_root().is_err());
    assert!(InitGit::plan(root, None).is_err());

    Ok(())
}

#[test]
fn main_can_be_created_later_from_an_explicit_reference() -> TestResult {
    let directory = tempdir()?;
    let root = directory.path();

    run_git(root, &["init", "-b", "legacy"])?;
    commit_empty(root)?;

    let git = GitCommands::new(root);

    let first = InitGit::plan(root, Some(GitWorkflow::Pro))?;
    assert!(!first.has_changes());

    // A separate, explicit decision establishes main's base.
    git.create_branch("main", "legacy")?;

    let second = InitGit::plan(root, Some(GitWorkflow::Pro))?;
    assert!(second.has_changes());

    InitGit::apply(root, &second, Some(GitWorkflow::Pro))?;

    let branches = git.branches()?;

    assert!(branches.local().iter().any(|name| name == "main"));
    assert!(branches.local().iter().any(|name| name == "dev"));

    Ok(())
}
