use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use scm::providers::ScmProviders;
use thiserror::Error;
use tracing::info;

use crate::parser::pattern_to_regex;

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
    location: PathBuf,
    owners: HashMap<String, Vec<String>>,
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

    // TODO: probably belongs in parse...
    fn parse(location: &Path) -> CodeOwnersResult<HashMap<String, Vec<String>>> {
        let mut owners = HashMap::new();
        for line in fs::read_to_string(&location)?.lines() {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with("#") {
                continue;
            }
            let fields: Vec<String> = line.split_whitespace().map(str::to_string).collect();
            if fields.len() == 1 {
                info!(
                    "expected at least two fields for rule in {}: {}",
                    &location.to_string_lossy(),
                    line
                );
                continue;
            }

            let (rule_pattern, rest) = fields.split_first().expect("Rule should have a pattern");
            owners.insert(rule_pattern.to_string(), rest.to_vec());
        }

        Ok(owners)
    }

    pub fn render(&self, format: &'static str) -> CodeOwnersResult<String> {
        todo!()
    }
}
