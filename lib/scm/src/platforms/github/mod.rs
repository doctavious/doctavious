pub mod provider;


type ClientResult<T> = Result<T, ClientError>;

use thiserror::Error;

/// Errors returned by the client
#[derive(Debug, Error)]
pub enum ClientError {

}
