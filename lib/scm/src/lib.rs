extern crate core;

use std::io;
use std::string::FromUtf8Error;

use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use thiserror::Error;

pub mod drivers;
pub mod hooks;
pub mod providers;

pub const HOOK_TEMPLATE: &[u8; 252] = include_bytes!("hooks/hook.tmpl");
lazy_static! {
    pub static ref DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX: Regex = Regex::new("DOCTAVIOUS").unwrap();
    pub static ref HOOK_TEMPLATE_CHECKSUM: String = format!("{:x}", md5::compute(HOOK_TEMPLATE));
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
// TODO: not sure this will work generically across SCM providers but will use for now
#[derive(Debug, Deserialize, Serialize)]
pub struct ScmCommit {
    /// Commit ID
    pub id: String,

    /// Commit message
    pub message: Option<String>,

    /// The author of the commit
    pub author: ScmSignature,

    /// Committer.
    pub committer: ScmSignature,

    /// The date of the commit
    pub timestamp: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScmSignature {
    pub name: Option<String>,
    pub email: Option<String>,
    pub timestamp: i64,
}

pub struct ScmBranch {
    pub name: String,
}

pub struct ScmTag {
    pub name: String,
    pub commit_id: String,
}

pub enum ScmCommitRange {
    // Current,
    // Latest,
    // Untagged,
    Tuple((String, Option<String>)),
    String(String),
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use crate::drivers::{Scm, ScmRepository};

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
