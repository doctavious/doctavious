use std::sync::Arc;

use octocrab::models::CommentId;
use octocrab::{params, Octocrab};

use crate::providers::ScmProvider;

struct GithubProvider {
    client: Arc<Octocrab>,
}

impl GithubProvider {
    pub fn new() -> Self {
        Self {
            client: octocrab::instance(),
        }
    }
}

pub struct GithubRepositoryIdentifier {
    owner: String,
    repository: String,
}

#[async_trait::async_trait]
impl ScmProvider for GithubProvider {
    type RepositoryIdentifier = GithubRepositoryIdentifier;

    async fn list_all_merge_requests_notes(&self, repo_id: Self::RepositoryIdentifier, pr: u64) {
        let comments = self
            .client
            .pulls(repo_id.owner, repo_id.repository)
            .list_comments(Some(pr))
            .sort(params::pulls::comments::Sort::Created)
            .direction(params::Direction::Ascending)
            .send()
            .await;
    }

    async fn create_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        pr: u64,
        body: String,
    ) {
        let comment = self
            .client
            .issues(repo_id.owner, repo_id.repository)
            .create_comment(pr, body)
            .await;
    }

    // TODO: should we try and avoid forcing pr argument if its not used?
    async fn update_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        _pr: u64,
        note_id: u64,
        body: String,
    ) {
        let comment = self
            .client
            .pulls(repo_id.owner, repo_id.repository)
            .comment(CommentId(note_id))
            .update(&body)
            .await;
    }
}
