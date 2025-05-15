use std::sync::Arc;

use crate::providers::ScmProvider;

struct GitLabProvider {
    client: Arc<gitlab_client::Client>,
}

struct GitlabRepositoryIdentifier {
    project_id: String,
}

#[async_trait::async_trait]
impl ScmProvider for GitLabProvider {
    type RepositoryIdentifier = GitlabRepositoryIdentifier;

    async fn list_all_merge_requests_notes(&self, repo_id: Self::RepositoryIdentifier, pr: u64) {
        let n = self
            .client
            .merge_requests()
            .list_all_merge_request_notes(&repo_id.project_id, pr)
            .await;
    }

    async fn create_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        pr: u64,
        body: String,
    ) {
        let n = self
            .client
            .merge_requests()
            .create_merge_request_note(&repo_id.project_id, pr, body)
            .await;
    }

    async fn update_merge_request_note(
        &self,
        repo_id: Self::RepositoryIdentifier,
        pr: u64,
        note_id: u64,
        body: String,
    ) {
        let n = self
            .client
            .merge_requests()
            .update_merge_request_note(&repo_id.project_id, pr, note_id, body)
            .await
            .unwrap()
            .body;
    }
}
