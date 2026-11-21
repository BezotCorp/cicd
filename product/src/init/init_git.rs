use std::fs;
use std::path::{Path, PathBuf};

use crate::git::{GitCommands, GitWorkflow};
use crate::init::{InitAction, InitError, InitPlan, InitStep};

pub(crate) struct InitGit;

impl InitGit {
    pub(crate) fn plan(path: &Path, profile: Option<GitWorkflow>) -> Result<InitPlan, InitError> {
        let root = fs::canonicalize(path).map_err(|source| InitError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        if !root.is_dir() {
            return Err(InitError::InvalidProject(format!(
                "Project path is not a directory: {}",
                root.display()
            )));
        }

        let git = GitCommands::new(&root);
        let repository = git.repository_root()?;
        let mut steps = Vec::new();

        if let Some(existing_root) = repository {
            if existing_root != root {
                return Err(InitError::InvalidProject(format!(
                    "{} belongs to the repository {}; use its root",
                    root.display(),
                    existing_root.display()
                )));
            }

            // Existing repositories keep their organization unless
            // the Pro workflow was explicitly selected.
            if profile != Some(GitWorkflow::Pro) {
                steps.push(Self::step(
                    InitAction::Preserve,
                    ".git",
                    "existing repository and branches preserved",
                ));

                return Ok(InitPlan::new(steps));
            }

            let info = git.inspect()?;
            let branches = git.branches()?;
            let has_main = branches.local().iter().any(|branch| branch == "main");
            let has_dev = branches.local().iter().any(|branch| branch == "dev");

            if has_main {
                steps.push(Self::step(
                    InitAction::Preserve,
                    "refs/heads/main",
                    "main already exists",
                ));

                steps.push(Self::step(
                    if has_dev {
                        InitAction::Preserve
                    } else {
                        InitAction::CreateBranch
                    },
                    "refs/heads/dev",
                    if has_dev {
                        "dev already exists"
                    } else {
                        "create dev from existing main"
                    },
                ));
            } else {
                let unborn_main = info.head().is_none() && info.branch() == Some("main");

                steps.push(Self::step(
                    if unborn_main {
                        InitAction::Preserve
                    } else {
                        InitAction::Deferred
                    },
                    "refs/heads/main",
                    if unborn_main {
                        "main is the unborn current branch"
                    } else {
                        "cannot determine a reliable base for main"
                    },
                ));

                steps.push(Self::step(
                    if has_dev {
                        InitAction::Preserve
                    } else {
                        InitAction::Deferred
                    },
                    "refs/heads/dev",
                    if has_dev {
                        "dev already exists"
                    } else {
                        "dev requires a valid main reference"
                    },
                ));
            }
        } else {
            steps.push(Self::step(
                InitAction::InitializeGit,
                ".git",
                "initialize Git with main as the initial branch",
            ));

            steps.push(Self::step(
                InitAction::Deferred,
                "refs/heads/dev",
                "dev will be created after the first commit",
            ));
        }

        Ok(InitPlan::new(steps))
    }

    pub(crate) fn apply(
        path: &Path,
        plan: &InitPlan,
        profile: Option<GitWorkflow>,
    ) -> Result<(), InitError> {
        let root = fs::canonicalize(path).map_err(|source| InitError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        // Recompute immediately before applying, to avoid using
        // an outdated plan.
        let current = Self::plan(&root, profile)?;

        if &current != plan {
            return Err(InitError::InvalidProject(
                "Git initialization plan changed; inspect again".to_owned(),
            ));
        }

        let git = GitCommands::new(&root);

        if git.repository_root()?.is_some() {
            let status = git.status()?;

            if !status.conflicted().is_empty() {
                return Err(InitError::InvalidProject(
                    "Cannot initialize branches while merge conflicts remain".to_owned(),
                ));
            }
        }

        for step in current.steps() {
            match step.action() {
                InitAction::InitializeGit => {
                    git.initialize_main()?;
                }
                InitAction::CreateBranch => {
                    if step.path() == Path::new("refs/heads/main") {
                        git.create_branch("main", "HEAD")?;
                    } else if step.path() == Path::new("refs/heads/dev") {
                        git.create_branch("dev", "main")?;
                    } else {
                        return Err(InitError::InvalidProject(format!(
                            "Unsupported Git branch step: {}",
                            step.path().display()
                        )));
                    }
                }
                InitAction::Preserve | InitAction::Deferred => {}
                InitAction::Create => {
                    return Err(InitError::InvalidProject(
                        "Unexpected file creation in Git initialization plan".to_owned(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn step(action: InitAction, path: &str, reason: &str) -> InitStep {
        InitStep::new(action, PathBuf::from(path), reason.to_owned())
    }
}
