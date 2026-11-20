use crate::init::{InitAction, InitStep};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InitPlan {
    steps: Vec<InitStep>,
}

impl InitPlan {
    pub(crate) fn new(steps: Vec<InitStep>) -> Self {
        Self { steps }
    }

    pub(crate) fn steps(&self) -> &[InitStep] {
        &self.steps
    }

    /// True when applying the plan would modify the repository.
    pub(crate) fn has_changes(&self) -> bool {
        self.steps
            .iter()
            .any(|step| step.action() == InitAction::Create)
    }
}
