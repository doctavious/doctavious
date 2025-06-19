use std::path::PathBuf;
use std::sync::Arc;

use clap::Args;
use code_ownify::notify::CodeNotify;
use github_client::webhook::PullRequestWebhookEventPayload;
use scm::commit::ScmCommitRange;
use scm::platforms::gitlab::provider::GitlabRepositoryIdentifier;
use scm::platforms::{ScmPlatformClient, github, gitlab};
use strum::{Display, EnumIter, EnumString, VariantNames};

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
    #[arg(long, default_value = "CODENOTIFY")]
    pub file_name: String,

    /// The threshold for notifying subscribers (Default: 0)
    #[arg(long, default_value_t = 0)]
    pub subscriber_threshold: i8,

    #[arg(long, default_value = "")]
    pub base_ref: String,

    #[arg(long)]
    pub head_ref: Option<String>,

    /// The author of the file diff
    #[arg(long)]
    pub author: Option<String>,
    // // TODO: this should be an enum
    // pub scm_platform: Option<ScmPlatform>,
}

// #[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, VariantNames, PartialEq)]
// #[strum(serialize_all = "lowercase")]
// #[non_exhaustive]
// pub enum ScmPlatform {
//     // #[strum(serialize = "github")]
//     Github,
//     // #[strum(serialize = "gitlab")]
//     Gitlab,
// }

pub(crate) fn execute(cmd: NotifyCommand) -> anyhow::Result<Option<String>> {
    // TODO: pass in scm to support writing comments
    let code_notify = get_options(cmd)?;

    code_notify.notify()?;
    Ok(Some(String::new()))
}

// TODO (sean): not sure whats the best way to handle, or how much we care to handle, related to
// CI discovery. For now we'll put here but we should reconsider this in the future
fn get_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    if std::env::var("GITHUB_ACTIONS")
        .ok()
        .is_some_and(|a| a == "true")
    {
        return github_actions_options(cmd);
    } else if std::env::var("GITLAB_CI").ok().is_some_and(|a| a == "true") {
        return gitlab_ci_options(cmd);
    } else if std::env::var("GITEA_ACTIONS")
        .ok()
        .is_some_and(|a| a == "true")
    {
        return gitea_actions_options(cmd);
    } else if std::env::var("BITBUCKET_BUILD_NUMBER")
        .ok()
        .is_some_and(|a| a == "true")
    {
        return bitbucket_pipelines_options();
    }

    cli_options(cmd)
}

fn cli_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
    Ok(CodeNotify {
        cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
        format: cmd.format.unwrap_or("text".to_string()),
        file_name: cmd.file_name,
        subscriber_threshold: cmd.subscriber_threshold,
        commit_range: ScmCommitRange(cmd.base_ref, cmd.head_ref),
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

    let subscriber_threshold = std::env::var("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
        .unwrap_or_default()
        .parse::<i8>()
        .unwrap_or_default();

    // TODO: constant?
    let filename =
        std::env::var("DOCTAVIOUS_CODENOTIFY_FILENAME").unwrap_or("CODENOTIFY".to_string());

    // TODO: github client
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
    let draft = std::env::var("CI_MERGE_REQUEST_DRAFT")
        .ok()
        .is_some_and(|e| e == "true");

    if draft {
        println!("Not sending notifications for draft pull request");
        // return error or just exit
        std::process::exit(1)
    }

    // TODO: Confirm CI_BUILDS_DIR is correct
    let cwd = std::env::var("CI_BUILDS_DIR")
        .and_then(|s| Ok(PathBuf::from(s)))
        .unwrap_or(std::env::current_dir()?);

    let subscriber_threshold = std::env::var("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
        .unwrap_or_default()
        .parse::<i8>()
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
        client: Arc::new(gitlab_client::Client::new("", None)?),
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
    // base ref - custom webhook parsing / api
    // head ref - BITBUCKET_COMMIT
    // author - .author.display_name
    // curl -s -u $USERNAME:$APP_PASSWORD \
    // "https://api.bitbucket.org/2.0/repositories/$BITBUCKET_REPO_FULL_NAME/pullrequests/$PR_ID" \
    // | jq -r '.destination.commit.hash, .source.commit.hash, .author.display_name'
    todo!()
}
