pub mod drivers;
pub mod hooks;
pub mod providers;

use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use std::{fs, io};

use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use thiserror::Error;

use crate::drivers::git::GitScmRepository;

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

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("Could not find supported SCM")]
    Unsupported,

    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
}

pub type ScmResult<T> = Result<T, ScmError>;

// #[remain::sorted]
// pub enum Scm {
//     Git,
// }

// We could have ScmType|Kind which is an enum of Git/Svn/Hg/etc
// then have a Scm trait which extends ScmRepository and ScmHookSupport

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

// TODO: not sure where this belongs but putting here for now. dont like the name
pub struct Scm;
impl Scm {
    pub fn get(cwd: &Path) -> ScmResult<Box<dyn ScmRepository>> {
        // TODO: it might be better to try and discover directory such as
        // `git rev-parse --show-toplevel` and `git rev-parse --git-dir`
        if fs::metadata(cwd.join(".git")).is_ok() {
            return Ok(Box::new(GitScmRepository::new(cwd)?));
        }

        if fs::metadata(cwd.join(".svn")).is_ok() {
            // TODO: implement
            unimplemented!()
        }

        if fs::metadata(cwd.join(".hg")).is_ok() {
            // TODO: implement
            unimplemented!()
        }

        Err(ScmError::Unsupported)
    }
}

pub trait ScmRepository {
    // fn open<P: AsRef<Path>>(path: P) -> ScmResult<Self>;

    fn checkout(&self, reference: &str) -> ScmResult<()>;

    // branches(options) -> Vec<Branch>

    // TODO: change to return branch get_branch / resolve_branch
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

    fn hook_path(&self) -> ScmResult<PathBuf>;

    fn scm(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Scm;

    #[test]
    fn hook_path() {
        let scm = Scm::get(Path::new("../..")).unwrap();
        let p = scm.hook_path().unwrap();
        assert_eq!("../../.git/hooks", p.to_string_lossy());
    }
}
