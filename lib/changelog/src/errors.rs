use somever::SomeverError;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangelogErrors {
    /// Error that may occur while generating changelog.
    #[error("Changelog error: `{0}`")]
    ChangelogError(String),

    #[error("")]
    CommitParser,

    #[error(transparent)]
    DoctaviousTemplatingError(#[from] doctavious_templating::TemplatingError),

    /// Error that may occur while I/O operations.
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    SomeverError(#[from] SomeverError),
}

pub type ChangelogResult<T> = Result<T, ChangelogErrors>;
