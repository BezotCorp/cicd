#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GitBranches {
    local: Vec<String>,
    remote: Vec<String>,
}

impl GitBranches {
    pub(crate) fn new(local: Vec<String>, remote: Vec<String>) -> Self {
        Self { local, remote }
    }

    pub(crate) fn local(&self) -> &[String] {
        &self.local
    }

    /// Remote-tracking branches, as `remote/branch`.
    pub(crate) fn remote(&self) -> &[String] {
        &self.remote
    }
}
