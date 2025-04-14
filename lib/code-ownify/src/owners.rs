// location: in the .github/, root, or docs/
// need to swap .github for provider agnostic

use std::io;

use regex::Regex;
use scm::errors::ScmError;
use thiserror::Error;
use tracing::{debug, info};
use walkdir::{DirEntry, WalkDir};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CodeOwnersError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),
}

pub type CodeOwnersResult<T> = Result<T, CodeOwnersError>;

pub struct CodeOwners {}

impl CodeOwners {}
