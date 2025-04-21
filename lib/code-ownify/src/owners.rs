use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use scm::providers::ScmProviders;
use thiserror::Error;
use tracing::info;

use crate::parser;

const CODEOWNERS: &'static str = "codeowners";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CodeOwnersError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),
}

pub type CodeOwnersResult<T> = Result<T, CodeOwnersError>;

pub struct CodeOwners {
    pub location: PathBuf,
    pub owners: HashMap<String, Vec<String>>,
}

impl CodeOwners {
    pub fn new(location: PathBuf) -> CodeOwnersResult<Option<CodeOwners>> {
        if location.exists() {
            let owners = Self::parse(&location)?;
            return Ok(Some(Self { location, owners }));
        }

        Ok(None)
    }

    pub fn discover(root: PathBuf) -> CodeOwnersResult<Option<CodeOwners>> {
        let root_codeowners = root.join(CODEOWNERS);
        if root_codeowners.exists() {
            let owners = Self::parse(&root_codeowners)?;
            return Ok(Some(Self {
                location: root_codeowners,
                owners,
            }));
        }

        let docs_codeowners = root.join("docs").join(CODEOWNERS);
        if docs_codeowners.exists() {
            let owners = Self::parse(&docs_codeowners)?;
            return Ok(Some(Self {
                location: docs_codeowners,
                owners,
            }));
        }

        for dot_directory in ScmProviders::dot_directories() {
            if dot_directory.exists() {
                let owners = Self::parse(&dot_directory)?;
                return Ok(Some(Self {
                    location: dot_directory,
                    owners,
                }));
            }
        }

        Ok(None)
    }

    fn parse(location: &Path) -> CodeOwnersResult<HashMap<String, Vec<String>>> {
        let mut owners = HashMap::new();
        for line in fs::read_to_string(&location)?.lines() {
            if let Some((rule_pattern, pattern_owners)) = parser::parse_line(line) {
                if pattern_owners.is_empty() {
                    info!(
                        "expected subscribers for rule in {}: {}",
                        &location.to_string_lossy(),
                        line
                    );
                    continue;
                }

                owners.insert(rule_pattern.to_string(), pattern_owners.to_vec());
            }
        }

        Ok(owners)
    }

    pub fn render(&self, format: &'static str) -> CodeOwnersResult<String> {
        todo!()
    }
}
