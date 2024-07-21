use thiserror::Error;

#[derive(Debug, Error)]
pub enum DoctaviousStdError {
    /// Error that may occur while I/O operations.
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),

    /// Error that may occur when attempting to interpret a sequence of u8 as a
    /// string.
    #[error("UTF-8 error: `{0}`")]
    Utf8Error(#[from] std::str::Utf8Error),
}

pub type Result<T> = core::result::Result<T, DoctaviousStdError>;
