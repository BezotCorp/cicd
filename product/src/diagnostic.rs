use crate::diagnostic_severity::DiagnosticSeverity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Diagnostic {
    severity: DiagnosticSeverity,
    message: String,
}

impl Diagnostic {
    pub(crate) fn new(severity: DiagnosticSeverity, message: String) -> Self {
        Self { severity, message }
    }

    pub(crate) fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}
