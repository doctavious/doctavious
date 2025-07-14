use std::path::PathBuf;
use std::sync::Arc;

use clap::Args;
use code_ownify::notify::CodeNotify;
use continuous_integration::ContinuousIntegrationProvider;
use github_client::webhook::PullRequestWebhookEventPayload;
use scm::commit::ScmCommitRange;
use scm::platforms::{ScmPlatform, gitlab};
use strum::VariantNames;

use crate::clap_enum_variants;

#[derive(Args, Debug)]
#[command()]
pub struct NotifyCommand {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    // TODO: should this be an enum? use constant from lib
    /// The format of the output (text or markdown)
    #[arg(long, short)]
    pub format: Option<String>,

    // TODO: use constant from lib
    /// The filename in which file subscribers are defined (Default: CODENOTIFY)
    #[arg(
        long,
        env = "DOCTAVIOUS_CODENOTIFY_FILENAME",
        default_value = "CODENOTIFY"
    )]
    pub file_name: String,

    /// The threshold for notifying subscribers (Default: 0)
    #[arg(
        long,
        env = "DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD",
        default_value_t = 0
    )]
    pub subscriber_threshold: i8,

    #[arg(long)]
    pub base_ref: Option<String>,

    #[arg(long)]
    pub head_ref: Option<String>,

    /// The author of the file diff
    #[arg(long)]
    pub author: Option<String>,

    // TODO: Doesnt work yet with lowercase values...
    #[arg(
        long,
        value_parser = clap_enum_variants!(ScmPlatform)
    )]
    pub scm_platform: Option<ScmPlatform>,
}

pub(crate) fn execute(cmd: NotifyCommand) -> anyhow::Result<Option<String>> {
    // TODO: Support writing comment to SCM platform
    let code_notify = get_options(cmd)?;

    code_notify.notify()?;
    Ok(Some(String::new()))
}

// TODO (sean): not sure whats the best way to handle, or how much we care to handle, related to
// CI discovery. For now we'll put here but we should reconsider this in the future
fn get_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    let subscriber_threshold = cmd.subscriber_threshold;
    let file_name = cmd.file_name;

    if let Some(ci_provider) = ContinuousIntegrationProvider::from_env() {
        let ci_context = ci_provider.context_from_env()?;
        if ci_context.draft {
            // TODO: better way to exit program
            std::process::exit(1)
        }

        let base_ref = cmd.base_ref.unwrap_or(ci_context.base);
        let head_ref = cmd.head_ref.unwrap_or(ci_context.head);

        let scm = ci_context.provider.associated_scm_platform();
        if scm.is_none() {
            // TODO: return error unable to detect SCM Platform
        }

        // let client = scm.unwrap().get_client_from_env();

        Ok(CodeNotify {
            cwd: ci_context.build_directory,
            format: "markdown".to_string(),
            file_name,
            subscriber_threshold,
            commit_range: ScmCommitRange(base_ref, Some(head_ref)),
            author: ci_context.author,
        })
    } else {
        // cli_options(cmd)

        if let Some(scm_platform) = cmd.scm_platform {
            // TODO: get client
        }

        Ok(CodeNotify {
            cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
            format: cmd.format.unwrap_or("text".to_string()),
            file_name,
            subscriber_threshold,
            commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
            author: None,
        })
    }

    // TODO: how do we want to support third-party CI providers?
    // Should maybe separate CI provider from SCM even if they are the same
    // if doctavious_std::env::as_boolean("GITHUB_ACTIONS") {
    //     return github_actions_options(cmd);
    // } else if doctavious_std::env::as_boolean("GITLAB_CI") {
    //     return gitlab_ci_options(cmd);
    // } else if doctavious_std::env::as_boolean("GITEA_ACTIONS") {
    //     return gitea_actions_options(cmd);
    // } else if doctavious_std::env::as_boolean("BITBUCKET_BUILD_NUMBER") {
    //     return bitbucket_pipelines_options();
    // }
    //
    // cli_options(cmd)
}

fn cli_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    Ok(CodeNotify {
        cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
        format: cmd.format.unwrap_or("text".to_string()),
        file_name: cmd.file_name,
        subscriber_threshold: cmd.subscriber_threshold,
        commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
        author: None,
    })
}

fn github_actions_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    // https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
    let cwd = match std::env::var("GITHUB_WORKSPACE") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            println!("Unable to get GitHub workspace cwd from env var 'GITHUB_WORKSPACE'");
            // return error or just exit
            std::process::exit(1)
        }
    };

    let event: PullRequestWebhookEventPayload = match std::env::var("GITHUB_EVENT_PATH") {
        Ok(path) => {
            // unable to read GitHub event json %s: %s", path, err
            let data = std::fs::read_to_string(path)?;
            serde_json::from_str(&data)?
        }
        Err(_) => {
            println!("env var 'GITHUB_EVENT_PATH' not set");
            // return error or just exit
            std::process::exit(1)
        }
    };

    if event.pull_request.draft.unwrap_or(false) {
        println!("Not sending notifications for draft pull request");
        // return error or just exit
        std::process::exit(1)
    }

    let subscriber_threshold: i8 =
        doctavious_std::env::parse("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
            .unwrap_or_default();

    // TODO: constant?
    let filename =
        std::env::var("DOCTAVIOUS_CODENOTIFY_FILENAME").unwrap_or("CODENOTIFY".to_string());

    // TODO: github client
    // TODO: GITHUB_API_URL
    // token := os.Getenv("GITHUB_TOKEN")
    // if token == "" {
    //     return fmt.Errorf("GITHUB_TOKEN is not set")
    // }
    // req.Header.Set("Authorization", "bearer "+token)

    Ok(CodeNotify {
        cwd,
        format: "markdown".to_string(),
        file_name: filename,
        subscriber_threshold,
        commit_range: ScmCommitRange(
            event.pull_request.base.sha,
            Some(event.pull_request.head.sha),
        ),
        author: event
            .pull_request
            .user
            .and_then(|u| Some(format!("@{}", u.login).to_string())),
    })
}

fn gitlab_ci_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    let draft = doctavious_std::env::as_boolean("CI_MERGE_REQUEST_DRAFT");
    if draft {
        println!("Not sending notifications for draft pull request");
        // return error or just exit
        std::process::exit(1)
    }

    // TODO: Confirm CI_BUILDS_DIR is correct
    let cwd = std::env::var("CI_BUILDS_DIR")
        .and_then(|s| Ok(PathBuf::from(s)))
        .unwrap_or(std::env::current_dir()?);
    let subscriber_threshold: i8 =
        doctavious_std::env::parse("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
            .unwrap_or_default();

    // TODO: constant?
    let filename =
        std::env::var("DOCTAVIOUS_CODENOTIFY_FILENAME").unwrap_or("CODENOTIFY".to_string());

    // TODO: should we return errors rather than defaults and using `ok()`?
    let base = std::env::var("CI_MERGE_REQUEST_DIFF_BASE_SHA").unwrap_or_default();
    let head = std::env::var("CI_COMMIT_SHORT_SHA").ok();
    let author = std::env::var("CI_COMMIT_AUTHOR")
        .ok()
        .and_then(|a| a.split_ascii_whitespace().next().map(|s| s.to_string()))
        .map(|s| format!("@{}", s));

    // TODO: fix this
    let provider = gitlab::provider::GitLabProvider {
        client: Arc::new(gitlab_client::client::Client::new("", None)?),
    };

    // provider.update_merge_request_note(GitlabRepositoryIdentifier {
    //     project_id: std::env::var("").unwrap()
    // }).await;

    Ok(CodeNotify {
        cwd,
        format: "markdown".to_string(),
        file_name: filename,
        subscriber_threshold,
        commit_range: ScmCommitRange(base, head),
        author,
    })
}

fn gitea_actions_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    // https://docs.gitea.com/usage/webhooks?_highlight=event#event-information
    // For pull_request events, in GitHub Actions, the ref is refs/pull/:prNumber/merge,
    // which is a reference to the merge commit preview. However, Gitea has no such reference.
    // Therefore, the ref in Gitea Actions is refs/pull/:prNumber/head, which points to the
    // head of pull request rather than the preview of the merge commit.

    // base ref - github.event.pull_request.base.sha
    // head ref - github.event.pull_request.head.sha
    // author - github.event.pull_request.user.login
    todo!()
}

fn bitbucket_pipelines_options() -> anyhow::Result<CodeNotify> {
    // BITBUCKET_COMMIT
    // BITBUCKET_PR_DESTINATION_COMMIT

    // Base Commit SHA ➡️ $BITBUCKET_PR_DESTINATION_COMMIT
    // Head Commit SHA ➡️ $BITBUCKET_COMMIT

    // base ref - custom webhook parsing / api
    // head ref - BITBUCKET_COMMIT
    // author - .author.display_name
    // curl -s -u $USERNAME:$APP_PASSWORD \
    // "https://api.bitbucket.org/2.0/repositories/$BITBUCKET_REPO_FULL_NAME/pullrequests/$PR_ID" \
    // | jq -r '.destination.commit.hash, .source.commit.hash, .author.display_name'
    todo!()
}
