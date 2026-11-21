use std::path::Path;

use crate::ci::CiCollection;
use crate::collect::{CollectError, CollectionReport, ProjectSnapshot};
use crate::data_sources::DataSources;
use crate::git::GitCollection;
use common::data_source_kind::DataSourceKind;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_severity::DiagnosticSeverity;
use crate::paths::CONFIG_PATH;

pub(crate) struct ProjectCollector;

impl ProjectCollector {
    /// Runs only the requested collection domains.
    /// CI can be collected without a Git repository.
    pub(crate) fn collect(
        path: &Path,
        sources: &[DataSourceKind],
    ) -> Result<CollectionReport, CollectError> {
        if sources.is_empty() {
            return Err(CollectError::InvalidProject(
                "At least one collection source is required".to_owned(),
            ));
        }

        let mut git = None;
        let mut ci = None;
        let mut collected = Vec::new();

        for source in sources {
            if collected.contains(source) {
                continue;
            }

            match source {
                DataSourceKind::Git => {
                    git = Some(GitCollection::collect(path)?);
                }
                DataSourceKind::Ci => {
                    let root = git
                        .as_ref()
                        .map(|collection| collection.info().root())
                        .unwrap_or(path);

                    ci = Some(CiCollection::collect(root)?);
                }
            }

            collected.push(*source);
        }

        Ok(CollectionReport::new(git, ci, collected))
    }

    /// Full diagnostic for initialization.
    /// Both Git and CI must be available.
    pub(crate) fn analyze(path: &Path) -> Result<ProjectSnapshot, CollectError> {
        let configured = DataSources::load().map_err(|error| {
            CollectError::InvalidProject(format!("Invalid data sources configuration: {error}"))
        })?;

        let selected: Vec<DataSourceKind> = configured.enabled_sources().collect();

        let report = Self::collect(path, &selected)?;
        let sources = report.sources().to_vec();
        let (git, ci) = report.into_parts();

        let git = git.ok_or_else(|| {
            CollectError::InvalidProject("Git collection missing from full diagnostic".to_owned())
        })?;

        let ci = ci.ok_or_else(|| {
            CollectError::InvalidProject("CI collection missing from full diagnostic".to_owned())
        })?;

        let (info, status, remotes, branches, hosting, suggested_provider) = git.into_parts();

        let (ci_files, already_initialized) = ci.into_parts();

        let mut findings = Vec::new();
        let mut warn = |message: &str| {
            findings.push(Diagnostic::new(
                DiagnosticSeverity::Warning,
                message.to_owned(),
            ));
        };

        if info.branch().is_none() {
            warn("HEAD is detached: no current branch");
        }
        if info.head().is_none() {
            warn("Repository has no commit yet");
        }
        if remotes.is_empty() {
            warn("No remote configured: Git hosting cannot be inferred");
        } else if hosting.is_none() {
            warn("Git hosting platform could not be identified");
        }
        if !status.staged().is_empty() || !status.unstaged().is_empty() {
            warn("Working tree has uncommitted changes to tracked files");
        }
        if !status.conflicted().is_empty() {
            warn("Repository contains unresolved merge conflicts");
        }

        if already_initialized {
            findings.push(Diagnostic::new(
                DiagnosticSeverity::Info,
                format!("{CONFIG_PATH} already exists"),
            ));
        }

        if ci_files.is_empty() {
            findings.push(Diagnostic::new(
                DiagnosticSeverity::Info,
                "No existing CI/CD configuration found".to_owned(),
            ));
        }

        Ok(ProjectSnapshot::new(
            info,
            status,
            remotes,
            branches,
            ci_files,
            hosting,
            suggested_provider,
            sources,
            already_initialized,
            findings,
        ))
    }
}
