use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::client::{Client, ClientResult, OffsetBasedPagination, Response};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequestNote {
    id: u64,
    body: String,
    author: MergeRequestNoteAuthor,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    system: bool,
    notable_id: u64,
    // TODO: could make into an enum
    notable_type: String,
    project_id: u64,
    noteable_iid: u64,
    resolvable: bool,
    confidential: bool,
    internal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeRequestNoteAuthor {
    id: u64,
    username: String,
    email: String,
    name: String,
    // TODO: could make this an enum
    state: String,
    created_at: DateTime<Utc>,
}

pub struct ListAllMergeRequestNotesRequest {
    project_id: String,
    merge_request_iid: u64,
    pagination: OffsetBasedPagination,
}

pub struct MergeRequests {
    pub client: Client,
}

impl MergeRequests {
    pub fn new(client: Client) -> Self {
        MergeRequests { client }
    }

    /// Gets a list of all notes for a single merge request.
    ///
    /// This function performs a `GET` to `/projects/:id/merge_requests/:merge_request_iid/notes`
    /// From https://docs.gitlab.com/api/notes/#list-all-merge-request-notes
    ///
    /// **Parameters**
    ///
    /// * project_id - The ID or URL-encoded path of the project
    /// * merge_request_iid
    /// * sort - asc or desc
    /// * order_by - Return merge request notes ordered by created_at or updated_at fields. Default is created_at
    pub async fn list_all_merge_request_notes(
        &self,
        project_id: &str,
        merge_request_iid: u64,
        sort: Option<&str>,
        order_by: Option<&str>,
        pagination: Option<OffsetBasedPagination>,
    ) -> ClientResult<Response<Vec<MergeRequestNote>>> {
        let mut query_args: Vec<(String, String)> = Default::default();
        if let Some(sort) = sort {}

        if let Some(order_by) = order_by {}

        if let Some(pagination) = pagination {}

        let query_ = serde_urlencoded::to_string(&query_args).unwrap();

        let url = self.client.url(
            &format!(
                "/projects/{}/merge_requests/{}/notes",
                crate::client::support::encode_path(project_id),
                merge_request_iid,
            ),
            None,
        );
        self.client
            .get(
                &url,
                crate::client::Message {
                    body: None,
                    content_type: None,
                },
            )
            .await
    }

    /// Creates a new note for a single merge request.
    ///
    /// This function performs a `POST` to `/projects/:id/merge_requests/:merge_request_iid/notes`
    /// From https://docs.gitlab.com/api/notes/#create-new-merge-request-note
    ///
    /// **Parameters**
    ///
    /// * project_id - The ID or URL-encoded path of the project
    /// * merge_request_iid
    /// * body
    /// * created_at - Date time string, ISO 8601 formatted. Example: 2016-03-11T03:45:40Z (requires administrator or project/group owner rights)
    /// * internal
    /// * merge_request_diff_head_sha - Required for the /merge quick action. The SHA of the head commit, which ensures the merge request wasn’t updated after the API request was sent.
    pub async fn create_merge_request_note(
        &self,
        project_id: &str,
        merge_request_iid: u64,
        body: String,
    ) -> ClientResult<Response<Vec<MergeRequestNote>>> {
        let url = self.client.url(
            &format!(
                "/projects/{}/merge_requests/{}/notes",
                crate::client::support::encode_path(project_id),
                merge_request_iid,
            ),
            None,
        );

        self.client
            .post(
                &url,
                crate::client::Message {
                    body: Some(reqwest::Body::from(body)),
                    content_type: None,
                },
            )
            .await
    }

    /// Modify existing note of a merge request.
    ///
    /// This function performs a `PUT` to `/projects/:id/merge_requests/:merge_request_iid/notes/:note_id`
    /// From https://docs.gitlab.com/api/notes/#modify-existing-merge-request-note
    ///
    /// **Parameters**
    ///
    /// * project_id - The ID or URL-encoded path of the project
    /// * merge_request_iid
    /// * note_id
    /// * body
    pub async fn update_merge_request_note(
        &self,
        project_id: &str,
        merge_request_iid: u64,
        note_id: u64,
        body: String,
    ) -> ClientResult<Response<Vec<MergeRequestNote>>> {
        let url = self.client.url(
            &format!(
                "/projects/{}/merge_requests/{}/notes/{}",
                crate::client::support::encode_path(project_id),
                note_id,
                merge_request_iid,
            ),
            None,
        );

        self.client
            .put(
                &url,
                crate::client::Message {
                    body: Some(reqwest::Body::from(body)),
                    content_type: None,
                },
            )
            .await
    }
}
