use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::path::PathBuf;

use github_client::webhook::PullRequestWebhookEventPayload;
use scm::platforms::ScmPlatform;
use serde_derive::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};
use thiserror::Error;

// TODO: was hoping to not have to define an error or result here...
#[remain::sorted]
#[derive(Debug, Error)]
pub enum ContinuousIntegrationError {
    #[error(transparent)]
    EnvVarError(#[from] doctavious_std::env::EnvVarError),

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    VarError(#[from] std::env::VarError),
}

pub type ContinuousIntegrationResult<T> = Result<T, ContinuousIntegrationError>;

#[derive(Clone, Debug, Deserialize, EnumIter, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[remain::sorted]
pub enum ContinuousIntegrationProvider {
    /// https://learn.microsoft.com/en-us/azure/devops/pipelines/build/variables?view=azure-devops&tabs=yaml
    AzureDevOpsPipelines,
    /// https://support.atlassian.com/bitbucket-cloud/docs/variables-and-secrets/#Default-variables
    BitBucket,
    /// https://buildkite.com/docs/pipelines/configure/environment-variables
    Buildkite,
    /// https://circleci.com/docs/variables/#built-in-environment-variables
    CircleCI,
    /// https://docs.gitea.com/usage/webhooks?_highlight=event#event-information
    Gitea,
    /// https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
    GitHubActions,
    /// https://docs.gitlab.com/ci/variables/predefined_variables/
    GitLab,
    Jenkins,
    TeamCity,
    Travis,
}

impl ContinuousIntegrationProvider {
    pub fn from_env() -> Option<Self> {
        for provider in Self::iter() {
            if provider.in_ci() {
                return Some(provider);
            }
        }

        None
    }

    pub fn in_ci(&self) -> bool {
        match self {
            ContinuousIntegrationProvider::AzureDevOpsPipelines => {
                doctavious_std::env::as_boolean("TF_BUILD")
            }
            ContinuousIntegrationProvider::BitBucket => {
                std::env::var("BITBUCKET_BUILD_NUMBER").is_ok()
            }
            ContinuousIntegrationProvider::Buildkite => {
                doctavious_std::env::as_boolean("BUILDKITE")
            }
            ContinuousIntegrationProvider::CircleCI => doctavious_std::env::as_boolean("CIRCLECI"),
            ContinuousIntegrationProvider::Gitea => {
                doctavious_std::env::as_boolean("GITEA_ACTIONS")
            }
            ContinuousIntegrationProvider::GitHubActions => {
                doctavious_std::env::as_boolean("GITHUB_ACTIONS")
            }
            ContinuousIntegrationProvider::GitLab => doctavious_std::env::as_boolean("GITLAB_CI"),
            ContinuousIntegrationProvider::Jenkins => std::env::var("JENKINS_URL").is_ok(),
            ContinuousIntegrationProvider::TeamCity => std::env::var("TEAMCITY_VERSION").is_ok(),
            ContinuousIntegrationProvider::Travis => doctavious_std::env::as_boolean("TRAVIS"),
        }
    }

    pub fn context_from_env(&self) -> ContinuousIntegrationResult<ContinuousIntegrationContext> {
        Ok(match self {
            ContinuousIntegrationProvider::GitHubActions => {
                let build_directory = doctavious_std::env::parse("GITHUB_WORKSPACE")?;
                let event_path = std::env::var("GITHUB_EVENT_PATH")?;
                let data = std::fs::read_to_string(event_path)?;
                let event: PullRequestWebhookEventPayload = serde_json::from_str(&data)?;
                ContinuousIntegrationContext {
                    provider: ContinuousIntegrationProvider::GitHubActions,
                    is_ci: self.in_ci(),
                    build_directory,
                    base: event.pull_request.base.sha,
                    head: event.pull_request.head.sha,
                    draft: event.pull_request.draft.unwrap_or(false),
                    author: event
                        .pull_request
                        .user
                        .and_then(|u| Some(format!("@{}", u.login).to_string())),
                }
            }
            ContinuousIntegrationProvider::GitLab => {
                let build_directory = doctavious_std::env::parse("CI_BUILDS_DIR")?;
                let draft = doctavious_std::env::as_boolean("CI_MERGE_REQUEST_DRAFT");
                let base = std::env::var("CI_MERGE_REQUEST_DIFF_BASE_SHA").unwrap_or_default();
                let head = std::env::var("CI_COMMIT_SHORT_SHA").unwrap_or_default();
                let author = std::env::var("CI_COMMIT_AUTHOR")
                    .ok()
                    .and_then(|a| a.split_ascii_whitespace().next().map(|s| s.to_string()))
                    .map(|s| format!("@{}", s));

                ContinuousIntegrationContext {
                    provider: ContinuousIntegrationProvider::GitLab,
                    is_ci: self.in_ci(),
                    build_directory,
                    base,
                    head,
                    draft,
                    author,
                }
            }
            ContinuousIntegrationProvider::Gitea => {
                // Gitea Actions is heavily inspired by GitHub Actions and they use the same
                // variables for the sake of compatability.
                let build_directory = doctavious_std::env::parse("GITHUB_WORKSPACE")?;
                let event_path = std::env::var("GITHUB_EVENT_PATH")?;
                let data = std::fs::read_to_string(event_path)?;
                let event: PullRequestWebhookEventPayload = serde_json::from_str(&data)?;
                ContinuousIntegrationContext {
                    provider: ContinuousIntegrationProvider::Gitea,
                    is_ci: self.in_ci(),
                    build_directory,
                    base: event.pull_request.base.sha,
                    head: event.pull_request.head.sha,
                    draft: event.pull_request.draft.unwrap_or(false),
                    author: event
                        .pull_request
                        .user
                        .and_then(|u| Some(format!("@{}", u.login).to_string())),
                }
            }
            _ => todo!(),
        })
    }

    // fn bitbucket_pipelines_options() -> anyhow::Result<CodeNotify> {
    //     // BITBUCKET_COMMIT
    //     // BITBUCKET_PR_DESTINATION_COMMIT
    //
    //     // Base Commit SHA ➡️ $BITBUCKET_PR_DESTINATION_COMMIT
    //     // Head Commit SHA ➡️ $BITBUCKET_COMMIT
    //
    //     // base ref - custom webhook parsing / api
    //     // head ref - BITBUCKET_COMMIT
    //     // author - .author.display_name
    //     // curl -s -u $USERNAME:$APP_PASSWORD \
    //     // "https://api.bitbucket.org/2.0/repositories/$BITBUCKET_REPO_FULL_NAME/pullrequests/$PR_ID" \
    //     // | jq -r '.destination.commit.hash, .source.commit.hash, .author.display_name'
    //     todo!()
    // }

    // TODO: is this useful here or is it mainly an SCMProvider concern?
    // GitHub Actions does reference a webhook payload path in their CI env which does contain
    // details
    pub fn context_from_webhook(&self, _data: &str) -> ContinuousIntegrationContext {
        todo!()
    }

    // TODO: does this make sense?
    // if we can determine which scm provider we can do that here or context?
    // even if we did determine we wouldnt know the authentication key to use and
    // would need users to pass details into the CLI. we can check for a well known env var
    pub fn associated_scm_platform(&self) -> Option<ScmPlatform> {
        match self {
            ContinuousIntegrationProvider::Gitea => Some(ScmPlatform::Gitea),
            ContinuousIntegrationProvider::GitHubActions => Some(ScmPlatform::GitHub),
            ContinuousIntegrationProvider::GitLab => Some(ScmPlatform::GitLab),
            _ => todo!(),
        }
    }
}

pub trait ContinuousIntegrationOperations: Send + Sync {
    // TODO: maybe something like a mode? or source? to support info from webhook?
    /// Check if currently running in CI environment
    fn in_ci(&self) -> bool;

    /// Get the provider type
    fn provider_type(&self) -> ContinuousIntegrationProvider;

    /// Get environment variables with provider-specific prefixes
    fn env_vars(&self) -> HashMap<String, String>;

    /// Get CI information
    fn get_context(&self) -> ContinuousIntegrationContext;
}

pub struct GitHubActionsProvider;
impl ContinuousIntegrationOperations for GitHubActionsProvider {
    fn in_ci(&self) -> bool {
        doctavious_std::env::as_boolean("GITHUB_ACTIONS")
    }

    fn provider_type(&self) -> ContinuousIntegrationProvider {
        ContinuousIntegrationProvider::GitHubActions
    }

    fn env_vars(&self) -> HashMap<String, String> {
        std::env::vars()
            .filter(|(key, _)| key.starts_with("GITHUB_"))
            .collect()
    }

    fn get_context(&self) -> ContinuousIntegrationContext {
        todo!()
    }
}

pub struct GitLabCiProvider;
impl ContinuousIntegrationOperations for GitLabCiProvider {
    fn in_ci(&self) -> bool {
        doctavious_std::env::as_boolean("GITLAB_CI")
    }

    fn provider_type(&self) -> ContinuousIntegrationProvider {
        ContinuousIntegrationProvider::GitLab
    }

    fn env_vars(&self) -> HashMap<String, String> {
        std::env::vars()
            .filter(|(key, _)| key.starts_with("CI_") || key.starts_with("GITLAB_"))
            .collect()
    }

    fn get_context(&self) -> ContinuousIntegrationContext {
        todo!()
    }
}

pub struct CircleCiProvider;
impl ContinuousIntegrationOperations for CircleCiProvider {
    fn in_ci(&self) -> bool {
        doctavious_std::env::as_boolean("CIRCLECI")
    }

    fn provider_type(&self) -> ContinuousIntegrationProvider {
        ContinuousIntegrationProvider::CircleCI
    }

    fn env_vars(&self) -> HashMap<String, String> {
        std::env::vars()
            .filter(|(key, _)| key.starts_with("CIRCLECI_"))
            .collect()
    }

    fn get_context(&self) -> ContinuousIntegrationContext {
        todo!()
    }
}

/// Common struct to hold continuous integration context details from
/// - environment variables
/// - webhook event payload
///
/// This is not a robust solution and contains only enough information to satisfy Doctavious use
/// cases.
///
/// TODO: Think through a more robust solution that would potentially allow us to get at any CI
/// information that might be exposed via environment variables, webhook payloads, etc across the
/// different CI providers.
pub struct ContinuousIntegrationContext {
    pub provider: ContinuousIntegrationProvider,
    pub is_ci: bool,
    pub build_directory: PathBuf,
    pub head: String,
    pub base: String,
    pub draft: bool,
    pub author: Option<String>,
}
