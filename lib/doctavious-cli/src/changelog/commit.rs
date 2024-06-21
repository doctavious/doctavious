use git2::{Commit as GitCommit, Signature as CommitSignature};
use scm::{ScmCommit, ScmSignature};
use serde::{Deserialize, Serialize};
use crate::CliResult;

/// Common commit object that is parsed from a repository.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Commit {
    // TODO: should this be Oid?
    /// Commit ID.
    pub id: String,
    /// Commit message including title, description and summary.
    pub message: String,
    // /// Conventional commit.
    // #[serde(skip_deserializing)]
    // pub conv:          Option<ConventionalCommit<'a>>,
    // /// Commit group based on a commit parser or its conventional type.
    // pub group: Option<String>,
    // /// Default commit scope based on (inherited from) conventional type or a
    // /// commit parser.
    // pub default_scope: Option<String>,
    // /// Commit scope for overriding the default one.
    // pub scope: Option<String>,
    /// A list of links found in the commit
    pub links: Vec<Link>,
    /// Commit author.
    pub author: Signature,
    /// Committer.
    pub committer: Signature,
    /// Whether if the commit has two or more parents.
    pub merge_commit: bool,
}

impl Commit {

    /// Processes the commit.
    ///
    /// * converts commit to a conventional commit
    /// * sets the group for the commit
    /// * extacts links and generates URLs
    pub fn process(&self) -> CliResult<()> {
        Ok(())
    }

    // into_conventional

    // preprocess

    // skip_commit

    // parse

    // parse_links
}

impl From<GitCommit<'_>> for Commit {
    fn from(value: GitCommit) -> Self {
        Self {
            id: value.id().to_string(),
            message: value.message().unwrap_or_default().to_string(),
            author: value.author().into(),
            committer: value.committer().into(),
            merge_commit: value.parents().count() > 1,
            ..Default::default()
        }
    }
}

impl From<&ScmCommit> for Commit {
    fn from(value: &ScmCommit) -> Self {
        Self {
            id: value.id.to_string(),
            // TODO: way to avoid clones?
            message: value.message.clone().unwrap_or_default(),
            author: value.author.clone().into(),
            committer: value.committer.clone().into(),
            // TODO: merge_commit
            merge_commit: false,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Link {
    /// Text of the link.
    pub text: String,
    /// URL of the link
    pub href: String,
}

/// Commit signature that indicates authorship.
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct Signature {
    /// Name on the signature.
    pub name: Option<String>,
    /// Email on the signature.
    pub email: Option<String>,
    /// Time of the signature.
    pub timestamp: i64,
}

impl From<CommitSignature<'_>> for Signature {
    fn from(signature: CommitSignature) -> Self {
        Self {
            name: signature.name().map(String::from),
            email: signature.email().map(String::from),
            timestamp: signature.when().seconds(),
        }
    }
}

impl From<ScmSignature> for Signature {
    fn from(signature: ScmSignature) -> Self {
        Self {
            name: signature.name,
            email: signature.email,
            timestamp: signature.timestamp,
        }
    }
}
