use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

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

    #[error(
        "Can't rename {0} to {0}.old as file already exists. If you wish to overwrite use 'force' option"
    )]
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
