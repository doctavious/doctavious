use serde::{Deserialize, Serialize};

use crate::models::orgs::Organization;
use crate::models::pulls::PullRequest;
use crate::models::teams::RequestedTeam;
use crate::models::{
    Author, EventInstallationId, Installation, Label, Milestone, Repository, RepositoryId,
};

/// The specific part of the payload in a webhook event
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum WebhookEventPayload {
    PullRequest(Box<PullRequestWebhookEventPayload>),
    Unknown(Box<serde_json::Value>),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[non_exhaustive]
pub struct WebhookEvent {
    pub sender: Option<Author>,
    pub repository: Option<Repository>,
    pub organization: Option<Organization>,
    pub installation: Option<EventInstallation>,
    #[serde(skip)]
    pub kind: WebhookEventType,
    #[serde(flatten)]
    pub specific: WebhookEventPayload,
}

/// Kind of webhook event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    PullRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct PullRequestWebhookEventPayload {
    pub action: PullRequestWebhookEventAction,
    pub assignee: Option<Author>,
    pub enterprise: Option<serde_json::Value>,
    pub number: u64,
    pub pull_request: PullRequest,
    pub reason: Option<String>,
    pub milestone: Option<Milestone>,
    pub label: Option<Label>,
    pub after: Option<String>,
    pub before: Option<String>,
    pub requested_reviewer: Option<Author>,
    pub requested_team: Option<RequestedTeam>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum PullRequestWebhookEventAction {
    Assigned,
    AutoMergeDisabled,
    AutoMergeEnabled,
    Closed,
    ConvertedToDraft,
    Demilestoned,
    Dequeued,
    Edited,
    Enqueued,
    Labeled,
    Locked,
    Milestoned,
    Opened,
    ReadyForReview,
    Reopened,
    ReviewRequestRemoved,
    ReviewRequested,
    Synchronize,
    Unassigned,
    Unlabeled,
    Unlocked,
}

// TODO: do we need this
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EventInstallation {
    /// A full installation object which is present for `Installation*` related webhook events.
    Full(Installation),
    /// The minimal installation object is present for all other event types.
    Minimal(EventInstallationId),
}
