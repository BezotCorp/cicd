use crate::ci::CiCollection;
use crate::git::GitCollection;
use common::data_source_kind::DataSourceKind;

#[derive(Debug)]
pub(crate) struct CollectionReport {
    git: Option<GitCollection>,
    ci: Option<CiCollection>,
    sources: Vec<DataSourceKind>,
}

impl CollectionReport {
    pub(crate) fn new(
        git: Option<GitCollection>,
        ci: Option<CiCollection>,
        sources: Vec<DataSourceKind>,
    ) -> Self {
        Self { git, ci, sources }
    }

    pub(crate) fn git(&self) -> Option<&GitCollection> {
        self.git.as_ref()
    }

    pub(crate) fn ci(&self) -> Option<&CiCollection> {
        self.ci.as_ref()
    }

    pub(crate) fn sources(&self) -> &[DataSourceKind] {
        &self.sources
    }

    pub(crate) fn into_parts(self) -> (Option<GitCollection>, Option<CiCollection>) {
        (self.git, self.ci)
    }
}
