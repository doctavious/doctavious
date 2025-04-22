use std::collections::HashMap;
use std::io::Write;
use std::path::{PathBuf};
use std::{fs, io};

use scm::providers::ScmProviders;
use thiserror::Error;
use tracing::info;

use crate::parser;

const CODEOWNERS: &'static str = "CODEOWNERS";

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
}

impl CodeOwners {
    pub fn new(location: PathBuf) -> Option<CodeOwners> {
        if location.exists() {
            return Some(Self { location });
        }

        None
    }

    pub fn discover(root: PathBuf) -> Option<CodeOwners> {
        let root_codeowners = root.join(CODEOWNERS);
        if root_codeowners.exists() {
            return Some(Self {
                location: root_codeowners,
            });
        }

        let docs_codeowners = root.join("docs").join(CODEOWNERS);
        if docs_codeowners.exists() {
            return Some(Self {
                location: docs_codeowners,
            });
        }

        for dot_directory in ScmProviders::dot_directories() {
            if dot_directory.exists() {
                return Some(Self {
                    location: dot_directory,
                });
            }
        }

        None
    }

    pub fn owners(&self) -> CodeOwnersResult<HashMap<String, Vec<String>>> {
        let mut owners = HashMap::new();
        for line in fs::read_to_string(&self.location)?.lines() {
            if let Some((rule_pattern, pattern_owners)) = parser::parse_line(line) {
                if pattern_owners.is_empty() {
                    info!(
                        "expected subscribers for rule in {}: {}",
                        &self.location.to_string_lossy(),
                        line
                    );
                    continue;
                }

                owners.insert(rule_pattern.to_string(), pattern_owners.to_vec());
            }
        }

        Ok(owners)
    }

    pub fn render<W: Write>(&self, writer: &mut W, format: &'static str) -> CodeOwnersResult<()> {
        let owners = self.owners()?;
        match format {
            "markdown" => {
                writeln!(writer, "| File(s) | Owners |")?;
                writeln!(writer, "|-|-|")?;
                for (pattern, code_owners) in owners {
                    writeln!(writer, "| {} | {} |", pattern, code_owners.join("<br>"))?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use testing::guard::TempDirGuard;

    use crate::owners::CodeOwners;

    #[test]
    fn new() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        fs::write(temp_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();

        let code_owners =
            CodeOwners::new(temp_dir.join("CODEOWNERS")).expect("Should have found CODEOWNER file");

        let owners = code_owners.owners().unwrap();
        assert_eq!(
            HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    #[test]
    fn discover_root() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        fs::write(temp_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();

        let code_owners = CodeOwners::discover(temp_dir).expect("Should have found CODEOWNER file");

        let owners = code_owners.owners().unwrap();

        assert_eq!(
            HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    #[test]
    fn discover_docs() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        let docs_dir = temp_dir.join("docs");
        fs::create_dir_all(temp_dir.join("docs")).unwrap();
        fs::write(docs_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();
        fs::write(docs_dir.join("file.md"), "").unwrap();

        let code_owners = CodeOwners::discover(docs_dir).expect("Should have found CODEOWNER file");

        let owners = code_owners.owners().unwrap();

        assert_eq!(
            HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    #[test]
    fn discover_scm_provider_dot_directory() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        let dot_dir = temp_dir.join(".github");
        fs::create_dir_all(temp_dir.join(".github")).unwrap();
        fs::write(dot_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();
        fs::write(dot_dir.join("file.md"), "").unwrap();

        let code_owners = CodeOwners::discover(dot_dir).expect("Should have found CODEOWNER file");

        let owners = code_owners.owners().unwrap();

        assert_eq!(
            HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    // #[test]
    // fn render_markdown() {
    //     let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();
    //
    //     fs::write(temp_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();
    //     fs::write(temp_dir.join("file.md"), "").unwrap();
    //
    //     let code_owners = CodeOwners::discover(temp_dir).expect("Should have found CODEOWNER file");
    //
    //     let mut writer = Vec::<u8>::new();
    //     let rendered = code_owners.render(&mut writer, "markdown");
    // }
}
