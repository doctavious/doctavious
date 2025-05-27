pub mod github;
pub mod gitlab;

use std::hash::Hash;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde_derive::{Deserialize, Serialize};
use strum::EnumIter;

use crate::commit::ScmSignature;

// TODO: rename to ScmHostedProviders?
#[derive(Clone, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[remain::sorted]
pub enum ScmPlatforms {
    BitBucket,
    Gitea,
    GitHub,
    GitLab,
    Gogs,
}

impl ScmPlatforms {
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

pub struct ScmPlatformCommitsResponse {
    pub id: String,
    pub url: String,
    pub commit: ScmPlatformCommit,
}

pub struct ScmPlatformCommit {
    pub url: String,
    pub author: ScmSignature,
    pub message: String,
}

pub struct ScmPlatformRelease {
    pub id: String,
    pub name: String,
    pub url: String,
    pub body: String,
    pub prerelease: bool,
    pub created: DateTime<Utc>,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequestNote {}

#[async_trait::async_trait]
pub trait ScmPlatform {
    type RepositoryIdentifier;

    // fn get_commits(&self) -> Vec<ScmProviderCommitsResponse>;
    //
    // fn get_releases(&self) -> Vec<ScmProviderRelease>;

    // TODO: per_page / page, sort, direction
    // -> Vec<PullRequestNote>
    async fn list_all_merge_requests_notes(&self, repo_id: Self::RepositoryIdentifier, mr: u64);

    async fn create_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        mr: u64,
        body: String,
    );

    async fn update_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        mr: u64,
        note_id: u64,
        body: String,
    );
}
