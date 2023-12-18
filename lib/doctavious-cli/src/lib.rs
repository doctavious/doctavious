pub mod cmd;
pub mod enums;
pub mod file_structure;
pub mod markup_format;
pub mod settings;

use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DoctaviousCliError {
    #[error("cas error: {0}")]
    CasError(#[from] cas::CasError),

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Errors that may occur when deserializing types from TOML format.
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("toml serialization error: `{0}`")]
    TomlSerializeError(#[from] toml::ser::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
