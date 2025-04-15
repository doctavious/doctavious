use std::hash::{Hash, Hasher};
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::commit::ScmSignature;

// TODO: rename to ScmHostedProviders?
#[derive(Clone, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[remain::sorted]
pub enum ScmProviders {
    BitBucket,
    Gitea,
    GitHub,
    GitLab,
    Gogs,
}

impl ScmProviders {
    pub fn dot_directories() -> Vec<PathBuf> {
        // TODO: these values maybe belong to the individual provider mods but fine for now
        vec![
            PathBuf::from(".bitbucket"),
            PathBuf::from(".gitea"),
            PathBuf::from(".github"),
            PathBuf::from(".gitlab"),
        ]
    }
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

    // TODO: get comment
    // TODO: comment on MR/PR
}
