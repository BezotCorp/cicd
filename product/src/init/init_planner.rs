use std::path::PathBuf;

use crate::init::{InitAction, InitDiagnosis, InitPlan, InitStep};
use crate::paths::CONFIG_PATH;

pub(crate) struct InitPlanner;

impl InitPlanner {
    /// Derives what initialization would do; nothing is written.
    pub(crate) fn plan(diagnosis: &InitDiagnosis) -> InitPlan {
        let mut steps = Vec::new();

        let config = PathBuf::from(CONFIG_PATH);
        steps.push(if diagnosis.already_initialized() {
            InitStep::new(
                InitAction::Preserve,
                config,
                "cicd configuration already present".to_owned(),
            )
        } else {
            InitStep::new(
                InitAction::Create,
                config,
                "cicd configuration for this repository".to_owned(),
            )
        });

        for file in diagnosis.ci_files() {
            steps.push(InitStep::new(
                InitAction::Preserve,
                file.path().to_path_buf(),
                format!("existing {} configuration", file.provider()),
            ));
        }

        if let Some(provider) = diagnosis.target_provider()
            && !diagnosis
                .ci_files()
                .iter()
                .any(|file| file.provider() == provider)
        {
            steps.push(InitStep::new(
                InitAction::Create,
                PathBuf::from(provider.default_config_path()),
                format!("initial {provider} pipeline (inferred from remote)"),
            ));
        }

        InitPlan::new(steps)
    }
}
