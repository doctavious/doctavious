pub mod cmd;
mod edit;
pub mod enums;
pub mod file_structure;
mod files;
mod git;
pub mod markup_format;
pub mod settings;
mod templates;

use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DoctaviousCliError {
    #[error("cas error: {0}")]
    CasError(#[from] cas::CasError),

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    // TODO: fix this
    #[error("{0}")]
    NoConfirmation(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Error that may occur while reserving ADR/RFD number.
    #[error("{0} has already been reserved")]
    ReservedNumberError(i32),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

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

    #[error("Unknown design document: {0}")]
    UnknownDesignDocument(String),

    #[error("walkdir error")]
    WalkdirError(#[from] walkdir::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
