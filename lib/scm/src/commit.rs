use std::hash::{Hash, Hasher};

use serde_derive::{Deserialize, Serialize};

// TODO: could possibly make this an enum with an associated trait
// TODO: should we have a CommitId type?
// TODO: not sure this will work generically across SCM providers but will use for now
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScmCommit {
    /// Commit ID
    pub id: String,

    /// Commit message
    pub message: String,

    /// Description/Summary of the commit
    pub description: String,

    /// Body of the commit
    pub body: String,

    /// The author of the commit
    pub author: ScmSignature,

    /// Committer.
    pub committer: ScmSignature,

    /// The date of the commit
    pub timestamp: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScmSignature {
    pub name: Option<String>,
    pub email: Option<String>,
    pub timestamp: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ScmBranch {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ScmTag {
    // TODO: might not need this
    pub id: Option<String>,

    /// The name of the tag
    pub name: String,

    /// The message of the tag (only if it was annotated).
    pub message: Option<String>,

    // TODO: optional or remove?
    pub timestamp: i64,
}

impl Hash for ScmTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.message.hash(state);
        self.timestamp.hash(state);
    }
}

pub struct ScmCommitRange(pub String, pub Option<String>);
