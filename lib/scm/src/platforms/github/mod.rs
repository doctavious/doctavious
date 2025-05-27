pub mod provider;

use octocrab::Error as OctocrabError;

type ClientResult<T> = Result<T, ClientError>;

use thiserror::Error;

/// Errors returned by the client
#[derive(Debug, Error)]
pub enum ClientError {
    #[error(transparent)]
    OctocrabError(#[from] OctocrabError),
}
