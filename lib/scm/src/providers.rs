use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};

use crate::ScmSignature;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[remain::sorted]
pub enum ScmProviders {
    BitBucket,
    Gitea,
    GitHub,
    GitLab,
    Gog,
}

// impl Hash for ScmProviders {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         match self {
//             ScmProviders::BitBucket => {}
//             ScmProviders::Gitea => {}
//             ScmProviders::GitHub => {}
//             ScmProviders::GitLab => {}
//             ScmProviders::Gog => {}
//         }
//     }
// }

pub struct ScmProviderCommitsResponse {
    pub id: String,
    pub url: String,
    pub commit: ScmProviderCommit,
}

pub struct ScmProviderCommit {
    pub url: String,
    pub author: ScmSignature,
    pub message: String,
}

pub struct ScmProviderRelease {
    pub id: String,
    pub name: String,
    pub url: String,
    pub body: String,
    pub prerelease: bool,
    pub created: DateTime<Utc>,
    pub published: DateTime<Utc>,
}

pub trait ScmProvider {
    fn get_commits(&self) -> Vec<ScmProviderCommitsResponse>;

    fn get_releases(&self) -> Vec<ScmProviderRelease>;
}
