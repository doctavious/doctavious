pub mod github;
pub mod gitlab;

use std::env;
use std::hash::Hash;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use gitlab_client::merge_requests::MergeRequestNoteAuthor;
use serde_derive::{Deserialize, Serialize};
use strum::{EnumIter, EnumString, VariantNames};

use crate::commit::ScmSignature;
use crate::platforms;

// TODO: rename to ScmHostedProviders?
#[derive(
    Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, EnumIter, EnumString, VariantNames,
)]
#[serde(rename_all = "lowercase")]
#[remain::sorted]
#[non_exhaustive]
pub enum ScmPlatform {
    Azure,
    BitBucket,
    Gitea,
    GitHub,
    GitLab,
    Gogs,
}

impl ScmPlatform {
    pub fn dot_directories() -> Vec<PathBuf> {
        // TODO: these values maybe belong to the individual provider mods but fine for now
        vec![
            PathBuf::from(".bitbucket"),
            PathBuf::from(".gitea"),
            PathBuf::from(".github"),
            PathBuf::from(".gitlab"),
        ]
    }

    pub fn get_client_from_env<R>(&self) -> Box<dyn ScmPlatformClient<R>> {
        let env_var_credentials_key = match self {
            ScmPlatform::Azure => "",
            ScmPlatform::BitBucket => "",
            ScmPlatform::Gitea => "",
            ScmPlatform::GitHub => "",
            ScmPlatform::GitLab => "",
            ScmPlatform::Gogs => "",
        };

        match self {
            ScmPlatform::Azure => {}
            ScmPlatform::BitBucket => {}
            ScmPlatform::Gitea => {}
            ScmPlatform::GitHub => {
                let creds = env::var(env_var_credentials_key).unwrap();
                let github_provider = platforms::github::provider::GithubProvider::new(&creds);
            }
            ScmPlatform::GitLab => {}
            ScmPlatform::Gogs => {}
        }

        todo!()
    }

    pub fn get_client_from_webhook(&self, data: serde_json::Value) {}

    // pub fn get_client<R>(&self) -> Box<dyn ScmPlatformClient<R>> {
    //     match &self {
    //         ScmPlatforms::GitHub => return Box::new(github::provider::GithubProvider::new()),
    //         _ => todo!(),
    //     }
    // }
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

pub struct ScmPlatformMergeRequestComment {
    pub id: u64,
    pub body: String,
}

pub enum Credentials {
    Environment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequestNote {}

// TODO: what if we have a concept of a "BoundScmPlatformClient" in which the client is bound
// to an repository based on env, webhook, or other context?
// Why? This way construction would be through a specific context that would internally the
// associated repository identifier and it wouldnt have to be constructed externally and passed in

// How about auth? We could get auth from env var, context object, Doctavious configuration, etc
pub trait ScmPlatformClientBuilder {
    fn from_env();

    fn from_webhook();

    fn auth_from_env(&self);

    // TODO: pass in auth. What do we need to support?
    fn auth(&self);

    fn build(&self);
}

#[async_trait::async_trait]
pub trait ScmPlatformClient<R> {
    // TODO: input so perhaps better as generic type parameter?
    // type RepositoryIdentifier;

    // fn get_commits(&self) -> Vec<ScmProviderCommitsResponse>;
    //
    // fn get_releases(&self) -> Vec<ScmProviderRelease>;

    // TODO: per_page / page, sort, direction
    // -> Vec<PullRequestNote>
    // async fn list_all_merge_requests_notes(&self, repo_id: Self::RepositoryIdentifier, mr: u64);
    async fn list_all_merge_requests_notes(&self, repo_id: R, mr: u64);

    // async fn create_merge_request_note(
    //     &self,
    //     repo_id: Self::RepositoryIdentifier,
    //     mr: u64,
    //     body: String,
    // );
    async fn create_merge_request_note(&self, repo_id: R, mr: u64, body: String);

    // async fn update_merge_request_note(
    //     &self,
    //     repo_id: Self::RepositoryIdentifier,
    //     mr: u64,
    //     note_id: u64,
    //     body: String,
    // );
    async fn update_merge_request_note(&self, repo_id: R, mr: u64, note_id: u64, body: String);
}

// TODO: probably could just be named ScmPlatformRepositoryClient
// A SCM platform client that is bound to a specific repository
#[async_trait::async_trait]
pub trait ScmPlatformRepositoryBoundedClient: Send {
    // TODO: per_page / page, sort, direction
    // -> Vec<PullRequestNote>
    // async fn list_all_merge_requests_notes(&self, mr: u64);
    async fn list_all_merge_requests_notes(&self, mr: u64) -> Vec<ScmPlatformMergeRequestComment>;

    // async fn create_merge_request_note(
    //     &self,
    //     mr: u64,
    //     body: String,
    // );
    async fn create_merge_request_note(&self, mr: u64, body: String);

    // async fn update_merge_request_note(
    //     &self,
    //     mr: u64,
    //     note_id: u64,
    //     body: String,
    // );
    async fn update_merge_request_note(&self, mr: u64, note_id: u64, body: String);
}
