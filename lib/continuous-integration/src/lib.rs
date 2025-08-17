use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use github_client::webhook::PullRequestWebhookEventPayload;
use scm::platforms::github::provider::GithubRepositoryBoundedProvider;
use scm::platforms::{ScmPlatform, ScmPlatformRepositoryBoundedClient, github};
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
    GithubScmPlatformError(#[from] github::ClientError),

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
    /// https://docs.github.com/en/actions/reference/variables-reference?versionId=free-pro-team%40latest&productId=actions#default-environment-variables
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
                // TODO: do we have to handle if this isn't the payload we expect?
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

                    repository: std::env::var("GITHUB_REPOSITORY").ok(),
                    branch: std::env::var("GITHUB_REF_NAME").ok(),
                    commit: std::env::var("GITHUB_SHA").ok(),
                    is_pull_request: std::env::var("GITHUB_EVENT_NAME").ok()
                        == Some("pull_request".to_string()),
                    pull_request: Some(event.pull_request.id.to_string()),
                    scm_platform: Some(ScmPlatform::GitHub),
                    metadata: Default::default(),
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
                    repository: std::env::var("CI_PROJECT_PATH").ok(),
                    branch: std::env::var("CI_COMMIT_REF_NAME").ok(),
                    commit: std::env::var("CI_COMMIT_SHA").ok(),
                    is_pull_request: std::env::var("CI_MERGE_REQUEST_ID").is_ok(),
                    pull_request: std::env::var("CI_MERGE_REQUEST_ID").ok(),
                    scm_platform: Some(ScmPlatform::GitLab),
                    metadata: Default::default(),
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
                    // TODO: confirm
                    repository: std::env::var("GITHUB_REPOSITORY").ok(),
                    branch: std::env::var("GITHUB_REF_NAME").ok(),
                    commit: std::env::var("GITHUB_SHA").ok(),
                    is_pull_request: std::env::var("GITHUB_EVENT_NAME").ok()
                        == Some("pull_request".to_string()),
                    pull_request: Some(event.pull_request.id.to_string()),
                    scm_platform: Some(ScmPlatform::Gitea),
                    metadata: Default::default(),
                }
            }
            // TODO: finish...
            ContinuousIntegrationProvider::Buildkite => ContinuousIntegrationContext {
                provider: ContinuousIntegrationProvider::Buildkite,
                is_ci: self.in_ci(),
                build_directory: Default::default(),
                head: "".to_string(),
                base: "".to_string(),
                draft: false,
                author: None,
                repository: std::env::var("BUILDKITE_PIPELINE_SLUG").ok(),
                branch: std::env::var("BUILDKITE_BRANCH").ok(),
                commit: std::env::var("BUILDKITE_COMMIT").ok(),
                is_pull_request: std::env::var("BUILDKITE_PULL_REQUEST").is_ok(),
                pull_request: None,
                scm_platform: Self::detect_scm_from_repo_url(std::env::var("BUILDKIT_REPO").ok()),
                metadata: Default::default(),
            },
            // TODO: finish...
            ContinuousIntegrationProvider::CircleCI => ContinuousIntegrationContext {
                provider: ContinuousIntegrationProvider::CircleCI,
                is_ci: self.in_ci(),
                build_directory: Default::default(),
                head: "".to_string(),
                base: "".to_string(),
                draft: false,
                author: None,
                repository: std::env::var("CIRCLECPROJECT_REPONAME").ok(),
                branch: std::env::var("CIRCLECI_BRANCH").ok(),
                commit: std::env::var("CIRCLECI_SHA1").ok(),
                is_pull_request: std::env::var("CIRCLECI_PULL_REQUEST").is_ok(),
                pull_request: None,
                scm_platform: Self::detect_scm_from_repo_url(
                    std::env::var("CIRCLECI_REPOSITORY_URL").ok(),
                ),
                metadata: Default::default(),
            },
            _ => todo!(),
        })
    }

    fn detect_scm_from_repo_url(repo_url: Option<String>) -> Option<ScmPlatform> {
        repo_url.and_then(|url| {
            if url.contains("github.com") {
                Some(ScmPlatform::GitHub)
            } else if url.contains("gitlab.com") {
                Some(ScmPlatform::GitLab)
            } else if url.contains("bitbucket.org") {
                Some(ScmPlatform::BitBucket)
            } else if url.contains("dev.azure.com") {
                Some(ScmPlatform::Azure)
            } else if url.contains("gitea.com") {
                Some(ScmPlatform::Gitea)
            } else if url.contains("gogs.io") {
                Some(ScmPlatform::Gogs)
            } else {
                None
            }
        })
    }

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

    // TODO (sean): Not sure if this is the best way to handle this but trying to make progress on
    // something. Will revisit later in particular when we get to the point of how to integrate this
    // when handling customer webhooks.
    pub fn associated_bound_scm_client(
        &self,
        context: &ContinuousIntegrationContext,
    ) -> ContinuousIntegrationResult<Option<Box<dyn ScmPlatformRepositoryBoundedClient>>> {
        match self {
            ContinuousIntegrationProvider::GitHubActions => {
                if let Some((owner, repo_name)) =
                    context.repository.as_ref().unwrap().split_once('/')
                {
                    let token = std::env::var("GITHUB_TOKEN")?;
                    let p = GithubRepositoryBoundedProvider::new(
                        owner.to_string(),
                        repo_name.to_string(),
                        &token,
                        std::env::var("GITHUB_API_URL").ok().as_deref(),
                    )?;
                    Ok(Some(Box::new(p)))
                } else {
                    Ok(None)
                }
            }
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
    pub repository: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub is_pull_request: bool,
    pub pull_request: Option<String>,
    pub scm_platform: Option<ScmPlatform>,
    pub metadata: HashMap<String, String>,
}
