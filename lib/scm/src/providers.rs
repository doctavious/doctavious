use chrono::{DateTime, Utc};

use crate::ScmSignature;

#[remain::sorted]
pub enum ScmGitProviders {
    BitBucket,
    Gitea,
    GitHub,
    GitLab,
    Gog,
}

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
