pub mod cmd;
pub mod enums;

use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DoctaviousCliError {

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;
