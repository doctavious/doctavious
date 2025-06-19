use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoctaviousStdError {
    /// Error that may occur while I/O operations.
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),

    // #[error(transparent)]
    // ParseError(#[from] std::num::TryFromIntError),
    /// Error that may occur when attempting to interpret a sequence of u8 as a
    /// string.
    #[error("UTF-8 error: `{0}`")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}

pub type DoctaviousStdResult<T> = Result<T, DoctaviousStdError>;
