use std::fs;
use std::io;
use std::path::Path;

use crate::ci::{CiConfigFile, CiProvider};
use crate::collect::CollectError;

const WORKFLOWS_DIR: &str = ".github/workflows";

const SINGLE_FILES: &[(&str, CiProvider)] = &[
    (".gitlab-ci.yml", CiProvider::GitLabCi),
    (".circleci/config.yml", CiProvider::CircleCi),
    ("Jenkinsfile", CiProvider::Jenkins),
    ("azure-pipelines.yml", CiProvider::AzurePipelines),
    ("bitbucket-pipelines.yml", CiProvider::BitbucketPipelines),
    (".travis.yml", CiProvider::TravisCi),
    (".drone.yml", CiProvider::Drone),
];

pub(crate) struct CiScanner;

impl CiScanner {
    /// Lists existing CI/CD configuration files below `root`, sorted by path.
    pub(crate) fn scan(root: &Path) -> Result<Vec<CiConfigFile>, CollectError> {
        let mut found = Vec::new();

        let workflows = root.join(WORKFLOWS_DIR);
        match fs::read_dir(&workflows) {
            Ok(entries) => {
                for entry in entries {
                    let entry = entry.map_err(|source| CollectError::Io {
                        path: workflows.clone(),
                        source,
                    })?;
                    let path = entry.path();
                    let is_yaml = path
                        .extension()
                        .is_some_and(|ext| ext == "yml" || ext == "yaml");

                    if is_yaml && path.is_file() {
                        found.push(CiConfigFile::new(
                            CiProvider::GitHubActions,
                            Path::new(WORKFLOWS_DIR).join(entry.file_name()),
                        ));
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(CollectError::Io {
                    path: workflows,
                    source,
                });
            }
        }

        for (relative, provider) in SINGLE_FILES {
            if root.join(relative).is_file() {
                found.push(CiConfigFile::new(
                    *provider,
                    Path::new(relative).to_path_buf(),
                ));
            }
        }

        found.sort_by(|a, b| a.path().cmp(b.path()));
        Ok(found)
    }
}
