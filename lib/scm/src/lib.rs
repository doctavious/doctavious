pub mod drivers;
pub mod hooks;
pub mod providers;

use std::io;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use regex::Regex;
use thiserror::Error;

pub const HOOK_TEMPLATE: &[u8; 252] = include_bytes!("hooks/hook.tmpl");
lazy_static! {
    pub static ref DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX: Regex = Regex::new("DOCTAVIOUS").unwrap();
    pub static ref HOOK_TEMPLATE_CHECKSUM: u32 = crc32c::crc32c(HOOK_TEMPLATE);
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ScmError {
    #[error("Branch `{0}` already exists")]
    BranchAlreadyExists(String),

    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("Can't rename {0} to {0}.old as file already exists. If you wish to overwrite use 'force' option")]
    OldHookExists(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Could not find supported SCM")]
    Unsupported,

    #[error("Hook {0} is not supported")]
    UnsupportedHook(String),

    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
}

pub type ScmResult<T> = Result<T, ScmError>;

pub const GIT: &str = "git";
pub const HG: &str = "hg";
pub const SVN: &str = "svn";

// TODO: should we have a CommitId type?

pub struct ScmCommit {
    /// Commit ID
    pub id: String,

    /// Commit message
    pub message: Option<String>,

    // probably should be signature
    /// The author of the commit
    pub author: ScmSignature,

    /// The date of the commit
    pub timestamp: Option<DateTime<Utc>>,
}

pub struct ScmSignature {
    pub name: Option<String>,
    pub email: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
}

pub struct ScmBranch {
    pub name: String,
}

pub struct ScmTag {
    pub name: String,
    pub commit_id: String,
}

pub trait ScmRepository {
    fn checkout(&self, reference: &str) -> ScmResult<()>;

    // TODO: change to return branch get_branch / resolve_branch
    // TODO: For our use cases (ADR/RFD reservation) branches aren't always useful so it might be
    // better to have a specific trait for those use cases
    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool>;

    fn write(&self, path: &Path, message: &str) -> ScmResult<()>;

    // get_commit -> ScmCommit

    // TODO: this probably doesn't belong on this trait.
    // Maybe should be be able to use get_commit with some option
    fn last_commit(&self) -> ScmResult<ScmCommit>;

    // options
    fn commits(&self, range: Option<String>) -> ScmResult<Vec<ScmCommit>>;

    fn tags(&self, pattern: &Option<String>) -> ScmResult<IndexMap<String, String>>;

    /// Determines if the working directory has changes
    fn is_dirty(&self) -> ScmResult<bool>;

    // head return commit/revision

    fn supported_hooks(&self) -> Vec<&'static str>;

    fn supports_hook(&self, hook: &str) -> bool;

    fn hooks_path(&self) -> ScmResult<PathBuf>;

    fn is_hook_file_sample(&self, path: &Path) -> bool;

    fn all_files(&self) -> ScmResult<Vec<PathBuf>>;

    // TODO: better name than staged. What do you call files in SVN that are added but not committed?
    fn staged_files(&self) -> ScmResult<Vec<PathBuf>>;

    // TODO: push files?
    // for SVN files that are added would be staged and files that are added and have modifications would be pushed?
    // or we just say this isnt used for all SCMs
    fn push_files(&self) -> ScmResult<Vec<PathBuf>>;

    fn files_by_command(&self, cmd: &String) -> ScmResult<Vec<PathBuf>>;

    fn scm(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::drivers::Scm;
    use crate::ScmRepository;

    #[test]
    fn hooks_path() {
        let base_path = Path::new("../..");
        let scm = Scm::get(base_path).unwrap();
        let hooks_path = scm.hooks_path().unwrap();

        assert_eq!(
            ".git/hooks",
            hooks_path
                .strip_prefix(fs::canonicalize(&Path::new("../..")).unwrap())
                .unwrap()
                .to_string_lossy()
        );
    }
}
