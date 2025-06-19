use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::{
    Author, AuthorAssociation, IssueState, Label, Milestone, PullRequestId, Repository, teams,
};
use crate::{Client, ClientResult, OffsetBasedPagination, Response};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequest {
    pub url: String,
    pub id: PullRequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_comments_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_comment_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses_url: Option<Url>,
    /// The pull request number.  Note that GitHub's REST API
    /// considers every pull-request an issue with the same number.
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<IssueState>,
    #[serde(default)]
    pub locked: bool,
    #[serde(default)]
    pub maintainer_can_modify: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Box<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<Label>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<Box<Milestone>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_lock_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mergeable_state: Option<MergeableState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_at: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merged_by: Option<Box<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<Box<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_reviewers: Option<Vec<Author>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_teams: Option<Vec<teams::RequestedTeam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rebaseable: Option<bool>,
    pub head: Box<Head>,
    pub base: Box<Base>,
    #[serde(rename = "_links")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Box<Links>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_association: Option<AuthorAssociation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<Box<Repository>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deletions: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_files: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_comments: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Head {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<Repository>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Base {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(rename = "ref")]
    pub ref_field: String,
    pub sha: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<Repository>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Links {
    #[serde(rename = "self")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_link: Option<SelfLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_link: Option<HtmlLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_link: Option<IssueLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_link: Option<CommentsLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_comments_link: Option<ReviewCommentsLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_comment_link: Option<ReviewCommentLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_link: Option<CommitsLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses_link: Option<StatusesLink>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pull_request")]
    pub pull_request_link: Option<PullRequestLink>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SelfLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HtmlLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct IssueLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommentsLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReviewCommentsLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ReviewCommentLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CommitsLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StatusesLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequestLink {
    pub href: Url,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum MergeableState {
    /// The head ref is out of date.
    Behind,
    /// The merge is blocked, eg. the base branch is protected by a required
    /// status check that is pending
    Blocked,
    /// Mergeable and passing commit status.
    Clean,
    /// The merge commit cannot be cleanly created.
    Dirty,
    /// The merge is blocked due to the pull request being a draft.
    Draft,
    /// Mergeable with passing commit status and pre-receive hooks.
    HasHooks,
    /// The state cannot currently be determined.
    Unknown,
    /// Mergeable with non-passing commit status.
    Unstable,
}

pub struct PullRequests {
    pub client: Client,
}

impl PullRequests {
    pub fn new(client: Client) -> Self {
        PullRequests { client }
    }

    // Vec<PullRequestNote>>
    pub async fn list_all_pull_request_notes(
        &self,
        owner: &str,
        repository: &str,
        pull_request_id: u64,
        sort: Option<&str>,
        order_by: Option<&str>,
        pagination: Option<OffsetBasedPagination>,
    ) -> ClientResult<Response<()>> {
        todo!()
    }

    // MergeRequestNote
    pub async fn create_pull_request_note(
        &self,
        owner: &str,
        repository: &str,
        merge_request_iid: u64,
        body: String,
    ) -> ClientResult<Response<Vec<()>>> {
        todo!()
    }

    // MergeRequestNote
    pub async fn update_pull_request_note(
        &self,
        owner: &str,
        repository: &str,
        note_id: u64,
        body: String,
    ) -> ClientResult<Response<Vec<()>>> {
        todo!()
    }
}
