pub mod cmd;
mod edit;
pub mod enums;
pub mod file_structure;
mod files;
mod git;
pub mod markup_format;
pub mod settings;
mod templates;
pub mod templating;

use thiserror::Error;

use crate::cmd::design_decisions;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DoctaviousCliError {
    #[error("cas error: {0}")]
    CasError(#[from] cas::CasError),

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    #[error("design decision error: {0}")]
    DesignDecisionErrors(#[from] design_decisions::DesignDecisionErrors),

    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),

    #[error("Glob pattern error: `{0}`")]
    GlobPatternError(#[from] glob::PatternError),

    #[error("invalid settings file")]
    InvalidSettingsFile,

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    // TODO: fix this
    #[error("{0}")]
    NoConfirmation(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Enum parsing error: {0}")]
    StrumParseError(#[from] strum::ParseError),

    #[error("Creating a Context from a Value/Serialize requires it being a JSON object")]
    TemplateContextError(),

    /// Error that may occur while template operations such as parse and render.
    #[error("Template error: `{0}`")]
    TemplateError(#[from] minijinja::Error),

    /// Error that may occur while parsing the template.
    #[error("Template parse error:\n{0}")]
    TemplateParseError(String),

    /// Errors that may occur when deserializing types from TOML format.
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("toml serialization error: `{0}`")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Unknown markup extension for path: {0}")]
    UnknownMarkupExtension(String),

    #[error("walkdir error")]
    WalkdirError(#[from] walkdir::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
