pub mod cmd;
pub mod enums;

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
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
