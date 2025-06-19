pub mod orgs;
pub mod pulls;
pub mod teams;

use std::fmt;
use std::ops::{Deref, DerefMut};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, de};
use url::Url;

type BaseIdType = u64;

macro_rules! id_type {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($($name:ident),+) => {$(
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize)]
        pub struct $name(pub BaseIdType);
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }
        impl Deref for $name {
            type Target = BaseIdType;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        impl $name {
            pub fn into_inner(self) -> BaseIdType {
                self.0
            }
        }
        impl From<BaseIdType> for $name {
            fn from(value: BaseIdType) -> Self {
                Self(value)
            }
        }
        impl AsRef<BaseIdType> for $name {
            fn as_ref(&self) -> &BaseIdType {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where D: Deserializer<'de>
            {
                struct IdVisitor;
                impl<'de> de::Visitor<'de> for IdVisitor {
                    type Value = $name;
                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                        where E: de::Error {
                        Ok($name(value))
                    }
                    fn visit_str<E>(self, id: &str) -> Result<Self::Value, E>
                        where E: de::Error {
                        id.parse::<u64>().map($name).map_err(de::Error::custom)
                    }
                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "expected {} as number or string", stringify!($name)) // TODO: $name
                    }
                }

                deserializer.deserialize_any(IdVisitor)
            }
         }
    )+};
}

id_type!(
    AppId,
    EventInstallationId,
    InstallationId,
    IssueId,
    LabelId,
    MilestoneId,
    OrgId,
    PullRequestId,
    RepositoryId,
    TeamId,
    UserId,
    UserOrOrgId
);

macro_rules! convert_into {
    ($($from:ident -> $to:ident),+) => {$(
        impl From<$from> for $to {
            fn from(v: $from) -> $to {
                $to(v.0)
            }
        }
    )+};
}

convert_into!(OrgId -> UserOrOrgId,
              UserId -> UserOrOrgId,
              PullRequestId -> IssueId);

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Author {
    pub login: String,
    pub id: UserId,
    pub node_id: String,
    pub avatar_url: Url,
    pub gravatar_id: String,
    pub url: Url,
    pub html_url: Url,
    pub followers_url: Url,
    pub following_url: Url,
    pub gists_url: Url,
    pub starred_url: Url,
    pub subscriptions_url: Url,
    pub organizations_url: Url,
    pub repos_url: Url,
    pub events_url: Url,
    pub received_events_url: Url,
    pub r#type: String,
    pub site_admin: bool,
    pub name: Option<String>,
    pub patch_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AuthorAssociation {
    Collaborator,
    Contributor,
    FirstTimer,
    FirstTimeContributor,
    Mannequin,
    Member,
    None,
    Owner,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Label {
    pub id: LabelId,
    pub node_id: String,
    pub url: Url,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub color: String,
    pub default: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Milestone {
    pub url: Url,
    pub html_url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_url: Option<Url>,
    pub id: MilestoneId,
    pub node_id: String,
    pub number: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_issues: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_issues: Option<i64>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Repository {
    pub id: RepositoryId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Author>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub private: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork: Option<bool>,
    pub url: Url,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignees_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blobs_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branches_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collaborators_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compare_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contributors_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployments_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forks_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_commits_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_refs_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_tags_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_comment_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue_events_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keys_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merges_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestones_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notifications_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pulls_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub releases_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssh_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stargazers_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub teams_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trees_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clone_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mirror_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub svn_url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<::serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forks_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stargazers_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watchers_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_issues_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_template: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topics: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_issues: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_projects: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_wiki: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_pages: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_downloads: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "date_serde::deserialize_opt"
    )]
    pub pushed_at: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "date_serde::deserialize_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "date_serde::deserialize_opt"
    )]
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_rebase_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_repository: Option<Box<Repository>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_squash_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_merge_commit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_update_branch: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_forking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<License>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_auto_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_branch_on_merge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<Box<Repository>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Box<Repository>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct License {
    pub key: String,
    pub name: String,
    pub node_id: String,
    pub spdx_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    pub html_url: Option<Url>,
    pub description: Option<String>,
    pub implementation: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub conditions: Option<Vec<String>>,
    pub limitations: Option<Vec<String>>,
    pub body: Option<String>,
    pub featured: Option<bool>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Permissions {
    #[serde(default)]
    pub admin: bool,
    pub push: bool,
    pub pull: bool,
    #[serde(default)]
    pub triage: bool,
    #[serde(default)]
    pub maintain: bool,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct Installation {
    pub id: InstallationId,
    pub account: Author,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_tokens_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<AppId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<UserOrOrgId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<String>,
    pub permissions: InstallationPermissions,
    /// List of events in the installation.
    ///
    /// Note that for Webhook events, the list of events in the
    /// list is guaranteed to match variants from
    /// [WebhookEventType](webhook_events::WebhookEventType)
    pub events: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_file_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_selection: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "date_serde::deserialize_opt"
    )]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "date_serde::deserialize_opt"
    )]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct InstallationPermissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contents: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuses: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IssueState {
    Open,
    Closed,
}
