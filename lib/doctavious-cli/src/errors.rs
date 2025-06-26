use std::str::Utf8Error;

use changelog::errors::ChangelogErrors;
use scm::errors::ScmError;
use thiserror::Error;

use crate::cmd::design_decisions;
use crate::settings::SettingErrors;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DoctaviousCliError {
    #[error("cas error: {0}")]
    CasError(#[from] cas::CasError),

    #[error(transparent)]
    ChangelogError(#[from] ChangelogErrors),

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    #[error("design decision error: {0}")]
    DesignDecisionErrors(#[from] design_decisions::DesignDecisionErrors),

    #[error(transparent)]
    DoctaviousStdError(#[from] doctavious_std::error::DoctaviousStdError),

    #[error(transparent)]
    DoctaviousTemplatingError(#[from] doctavious_templating::TemplatingError),

    // TODO: adhoc might be a better name
    #[error("{0}")]
    GeneralError(String),

    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),

    #[error("Glob pattern error: `{0}`")]
    GlobPatternError(#[from] glob::PatternError),

    /// Error that may occur while parsing integers.
    #[error("Failed to parse integer: `{0}`")]
    IntParseError(#[from] std::num::TryFromIntError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    MarkupError(#[from] markup::MarkupError),

    // TODO: fix this
    #[error("{0}")]
    NoConfirmation(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("SCM error: {0}")]
    ScmError(#[from] ScmError),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    SettingError(#[from] SettingErrors),

    #[error("Enum parsing error: {0}")]
    StrumParseError(#[from] strum::ParseError),

    /// Error that may occur due to system time related anomalies.
    #[error("System time error: `{0}`")]
    SystemTimeError(#[from] std::time::SystemTimeError),

    #[error("TIL already exists")]
    TilAlreadyExists,

    /// Errors that may occur when deserializing types from TOML format.
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("toml serialization error: `{0}`")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),

    #[error("walkdir error")]
    WalkdirError(#[from] walkdir::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
