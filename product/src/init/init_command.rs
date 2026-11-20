use std::path::Path;

use crate::diagnostic_severity::DiagnosticSeverity;
use crate::init::{InitAction, InitAnalyzer, InitError, InitPlanner};

pub(crate) struct InitCommand;

impl InitCommand {
    /// Prints the diagnosis and the initialization plan. Dry run only.
    pub(crate) fn run(path: &Path) -> Result<(), InitError> {
        let diagnosis = InitAnalyzer::analyze(path)?;
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
        println!("=== Init plan (dry run, nothing written) ===");
        for step in plan.steps() {
            let label = match step.action() {
                InitAction::Create => "create  ",
                InitAction::Preserve => "preserve",
            };
            println!("{label} {} — {}", step.path().display(), step.reason());
        }
        if !plan.has_changes() {
            println!("Nothing to do.");
        }

        Ok(())
    }
}
