use std::path::Path;

use crate::collect::ProjectCollector;
use crate::diagnostic_severity::DiagnosticSeverity;
use crate::git::GitWorkflow;
use crate::init::{InitAction, InitError, InitGit, InitPlanner};

pub(crate) struct InitCommand;

impl InitCommand {
    /// Inspects the project and optionally applies safe Git initialization.
    pub(crate) fn run(
        path: &Path,
        apply: bool,
        profile: Option<GitWorkflow>,
    ) -> Result<(), InitError> {
        let git_plan = InitGit::plan(path, profile)?;

        println!("=== Git initialization plan ===");

        for step in git_plan.steps() {
            let action = match step.action() {
                InitAction::InitializeGit => "git-init",
                InitAction::CreateBranch => "branch",
                InitAction::Preserve => "preserve",
                InitAction::Deferred => "deferred",
                InitAction::Create => "create",
            };

            println!("{action:9} {} — {}", step.path().display(), step.reason());
        }

        if apply {
            InitGit::apply(path, &git_plan, profile)?;
            println!("Git initialization applied.");
        } else if git_plan
            .steps()
            .iter()
            .any(|step| step.action() == InitAction::InitializeGit)
        {
            println!("Git is absent. Use --apply to initialize it.");
            println!("No files or branches were changed.");
            return Ok(());
        }

        let diagnosis = ProjectCollector::analyze(path)?;
        let plan = InitPlanner::plan(&diagnosis);
        let info = diagnosis.info();

        println!("=== Diagnostic ===");
        println!("Repository: {}", info.root().display());
        println!("Branch: {}", info.branch().unwrap_or("(detached HEAD)"));
        println!("HEAD: {}", info.head().unwrap_or("(no commit)"));

        let status = diagnosis.status();
        println!(
            "Working tree: {} staged, {} unstaged, {} untracked, {} deleted",
            status.staged().len(),
            status.unstaged().len(),
            status.untracked().len(),
            status.deleted().len()
        );

        println!("Conflicted files: {}", status.conflicted().len());

        println!("Remotes:");
        if diagnosis.remotes().is_empty() {
            println!("  (none)");
        }
        for remote in diagnosis.remotes() {
            println!("  {}", remote.name());
            for url in remote.fetch_urls() {
                println!("    fetch: {url}");
            }
            for url in remote.push_urls() {
                println!("    push: {url}");
            }
        }

        println!(
            "Local branches: {}",
            diagnosis.branches().local().join(", ")
        );
        println!(
            "Remote branches: {}",
            diagnosis.branches().remote().join(", ")
        );

        println!("CI/CD files:");
        if diagnosis.ci_files().is_empty() {
            println!("  (none)");
        }
        for file in diagnosis.ci_files() {
            println!("  {} [{}]", file.path().display(), file.provider());
        }
        match diagnosis.target_provider() {
            Some(provider) => println!("Inferred provider: {provider}"),
            None => println!("Inferred provider: (unknown)"),
        }

        for finding in diagnosis.findings() {
            let label = match finding.severity() {
                DiagnosticSeverity::Info => "info",
                DiagnosticSeverity::Warning => "warning",
            };
            println!("[{label}] {}", finding.message());
        }

        println!();
        println!("=== CI/CD file plan (not applied) ===");
        for step in plan.steps() {
            let label = match step.action() {
                InitAction::Create => "create  ",
                InitAction::Preserve => "preserve",
                InitAction::InitializeGit => "git-init",
                InitAction::CreateBranch => "branch",
                InitAction::Deferred => "deferred",
            };
            println!("{label} {} — {}", step.path().display(), step.reason());
        }
        if !plan.has_changes() {
            println!("Nothing to do.");
        } else {
            println!("CI/CD configuration generation is not implemented yet.");
        }

        Ok(())
    }
}
