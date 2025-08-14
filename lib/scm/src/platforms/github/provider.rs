use std::sync::Arc;

use crate::platforms::github::ClientResult;
use crate::platforms::{
    ScmPlatformClient, ScmPlatformMergeRequestComment, ScmPlatformRepositoryBoundedClient,
};

pub(crate) struct GithubProvider {
    pub client: Arc<github_client::client::Client>,
}

impl GithubProvider {
    pub fn new(credentials: &str) -> ClientResult<Self> {
        let client = github_client::client::Client::new(
            "",
            github_client::client::Credentials::PrivateToken(String::from(credentials)),
        )?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
}

pub struct GithubRepositoryIdentifier {
    owner: String,
    repository: String,
}

#[async_trait::async_trait]
impl ScmPlatformClient<GithubRepositoryIdentifier> for GithubProvider {
    // type RepositoryIdentifier = GithubRepositoryIdentifier;

    async fn list_all_merge_requests_notes(
        &self,
        // repo_id: Self::RepositoryIdentifier,
        repo_id: GithubRepositoryIdentifier,
        pr: u64,
    ) {
        let comments = self
            .client
            .pull_requests()
            .list_all_pull_request_notes(&repo_id.owner, &repo_id.repository, pr, None, None, None)
            // .await;
            // .pulls(repo_id.owner, repo_id.repository)
            // .list_comments(Some(pr))
            // .sort(params::pulls::comments::Sort::Created)
            // .direction(params::Direction::Ascending)
            // .send()
            .await;
    }

    async fn create_merge_request_note(
        &self,
        // repo_id: Self::RepositoryIdentifier,
        repo_id: GithubRepositoryIdentifier,
        pr: u64,
        body: String,
    ) {
        let comment = self
            .client
            .pull_requests()
            .create_pull_request_note(&repo_id.owner, &repo_id.repository, pr, body)
            // .issues(repo_id.owner, repo_id.repository)
            // .create_comment(pr, body)
            .await;
    }

    // TODO: should we try and avoid forcing pr argument if its not used?
    // We could try and force something common at the SCM level rather than the client
    // Could use a struct rather than individual args which could be a common struct or a generic/associative type
    async fn update_merge_request_note(
        &self,
        // repo_id: Self::RepositoryIdentifier,
        repo_id: GithubRepositoryIdentifier,
        _pr: u64,
        note_id: u64,
        body: String,
    ) {
        let comment = self
            .client
            .pull_requests()
            .update_pull_request_note(&repo_id.owner, &repo_id.repository, note_id, body)
            // .pulls(repo_id.owner, repo_id.repository)
            // .comment(CommentId(note_id))
            // .update(&body)
            .await;
    }
}

pub struct GithubRepositoryBoundedProvider {
    pub owner: String,
    pub repository: String,
    pub client: Arc<github_client::client::Client>,
}

impl GithubRepositoryBoundedProvider {
    pub fn new(owner: String, repository: String, credentials: &str) -> ClientResult<Self> {
        // TODO: include retry middleware and eventually tracing
        let client = github_client::client::ClientBuilder::new()?
            .with_credentials(github_client::client::Credentials::PrivateToken(
                String::from(credentials),
            ))
            .build()?;

        Ok(Self {
            owner,
            repository,
            client: Arc::new(client),
        })
    }
}

#[async_trait::async_trait]
impl ScmPlatformRepositoryBoundedClient for GithubRepositoryBoundedProvider {
    // TODO: sort / order by / pagination
    async fn list_all_merge_requests_notes(&self, pr: u64) -> Vec<ScmPlatformMergeRequestComment> {
        let comments = self
            .client
            .pull_requests()
            .list_all_pull_request_notes(&self.owner, &self.repository, pr, None, None, None)
            .await;

        todo!()
    }

    async fn create_merge_request_note(&self, pr: u64, body: String) {
        let comment = self
            .client
            .pull_requests()
            .create_pull_request_note(&self.owner, &self.repository, pr, body)
            .await;
    }

    // TODO: should we try and avoid forcing pr argument if its not used?
    // We could try and force something common at the SCM level rather than the client
    // Could use a struct rather than individual args which could be a common struct or a generic/associative type
    async fn update_merge_request_note(&self, _pr: u64, note_id: u64, body: String) {
        let comment = self
            .client
            .pull_requests()
            .update_pull_request_note(&self.owner, &self.repository, note_id, body)
            .await;
    }
}
