use std::fs;
use std::path::{Path, PathBuf};

use crate::ci::{CiConfigFile, CiScanner};
use crate::collect::CollectError;
use crate::paths::CONFIG_PATH;

#[derive(Debug)]
pub(crate) struct CiCollection {
    root: PathBuf,
    files: Vec<CiConfigFile>,
    already_initialized: bool,
}

impl CiCollection {
    pub(crate) fn collect(path: &Path) -> Result<Self, CollectError> {
        let root = fs::canonicalize(path).map_err(|source| CollectError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        if !root.is_dir() {
            return Err(CollectError::InvalidProject(format!(
                "Collection path is not a directory: {}",
                root.display()
            )));
        }

        let files = CiScanner::scan(&root)?;
        let already_initialized = root.join(CONFIG_PATH).is_file();

        Ok(Self {
            root,
            files,
            already_initialized,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn files(&self) -> &[CiConfigFile] {
        &self.files
    }

    pub(crate) fn already_initialized(&self) -> bool {
        self.already_initialized
    }

    pub(crate) fn into_parts(self) -> (Vec<CiConfigFile>, bool) {
        (self.files, self.already_initialized)
    }
}
