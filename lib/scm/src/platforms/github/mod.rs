pub mod provider;

type ClientResult<T> = Result<T, ClientError>;

use thiserror::Error;

/// Errors returned by the client
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("GitHub error: `{0}`")]
    GitHubClientError(#[from] github_client::client::ClientError),
}
