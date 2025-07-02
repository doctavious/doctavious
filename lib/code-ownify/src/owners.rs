use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::{fs, io};

use scm::platforms::ScmPlatform;
use thiserror::Error;
use tracing::info;

use crate::parser;

const CODEOWNERS: &'static str = "CODEOWNERS";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CodeOwnersError {
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

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
    pub fn new(location: PathBuf) -> CodeOwnersResult<CodeOwners> {
        Ok(Self {
            owners: CodeOwners::parse(&location)?,
            location,
        })
    }

    pub fn discover(root: PathBuf) -> CodeOwnersResult<Option<CodeOwners>> {
        let root_codeowners = root.join(CODEOWNERS);
        if root_codeowners.exists() {
            return Ok(Some(Self {
                owners: CodeOwners::parse(&root_codeowners)?,
                location: root_codeowners,
            }));
        }

        let docs_codeowners = root.join("docs").join(CODEOWNERS);
        if docs_codeowners.exists() {
            return Ok(Some(Self {
                owners: CodeOwners::parse(&docs_codeowners)?,
                location: docs_codeowners,
            }));
        }

        for dot_directory in ScmPlatform::dot_directories() {
            if dot_directory.exists() {
                return Ok(Some(Self {
                    owners: CodeOwners::parse(&dot_directory)?,
                    location: dot_directory,
                }));
            }
        }

        Ok(None)
    }

    pub fn location(&self) -> &Path {
        self.location.as_path()
    }

    pub fn owners(&self) -> &HashMap<String, Vec<String>> {
        &self.owners
    }

    fn parse(path: &Path) -> CodeOwnersResult<HashMap<String, Vec<String>>> {
        let mut owners = HashMap::new();
        for line in fs::read_to_string(path)?.lines() {
            if let Some((rule_pattern, pattern_owners)) = parser::parse_line(line) {
                if pattern_owners.is_empty() {
                    info!(
                        "expected subscribers for rule in {}: {}",
                        path.to_string_lossy(),
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
        let mut writer = Vec::<u8>::new();
        let owners = self.owners();
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

        Ok(String::from_utf8(writer)?)
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

        let owners = code_owners.owners();
        assert_eq!(
            &HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    #[test]
    fn discover_root() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        fs::write(temp_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();

        let code_owners = CodeOwners::discover(temp_dir)
            .expect("Should have found CODEOWNER file")
            .unwrap();

        let owners = code_owners.owners();

        assert_eq!(
            &HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
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

        let code_owners = CodeOwners::discover(docs_dir)
            .expect("Should have found CODEOWNER file")
            .unwrap();

        let owners = code_owners.owners();

        assert_eq!(
            &HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
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

        let code_owners = CodeOwners::discover(dot_dir)
            .expect("Should have found CODEOWNER file")
            .unwrap();

        let owners = code_owners.owners();

        assert_eq!(
            &HashMap::from([("**/*.md".to_string(), vec!["@markdown".to_string()])]),
            owners
        );
    }

    #[test]
    fn render_markdown() {
        let (temp_dir, _tempdir_guard) = TempDirGuard::new().unwrap();

        fs::write(temp_dir.join("CODEOWNERS"), "**/*.md @markdown").unwrap();
        fs::write(temp_dir.join("file.md"), "").unwrap();

        let code_owners = CodeOwners::discover(temp_dir)
            .expect("Should have found CODEOWNER file")
            .unwrap();

        let rendered = code_owners.render("markdown").unwrap();

        assert_eq!(
            "| File(s) | Owners |\n|-|-|\n| **/*.md | @markdown |\n",
            rendered.as_str()
        )
    }
}
