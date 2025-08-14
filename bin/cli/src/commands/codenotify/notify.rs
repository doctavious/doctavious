use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use code_ownify::notify::CodeNotify;
use continuous_integration::{ContinuousIntegrationContext, ContinuousIntegrationProvider};
use scm::commit::ScmCommitRange;
use scm::platforms::ScmPlatformRepositoryBoundedClient;
use tracing::{debug, error, info, span, warn};

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
}

enum RuntimeEnv {
    CI(
        (
            CodeNotify,
            ContinuousIntegrationContext,
            Box<dyn ScmPlatformRepositoryBoundedClient>,
        ),
    ),
    Local(CodeNotify),
}

pub(crate) async fn execute(cmd: NotifyCommand) -> anyhow::Result<Option<String>> {
    let runtime_env_setup = setup(cmd)?;
    match runtime_env_setup {
        RuntimeEnv::CI((code_notify, ci_context, scm)) => {
            let code_notify_result = code_notify.notify()?;

            // TODO: could likely just return errors here instead of the expects
            let pr_number = u64::from_str(
                &ci_context
                    .pull_request
                    .expect("Pull request cannot be none"),
            )
            .expect("unable to parse pull request");

            let notes = scm.list_all_merge_requests_notes(pr_number).await;
            let mut comment_id = None;
            for note in notes {
                if note.body.starts_with(&code_notify.markdown_comment_title()) {
                    comment_id = Some(note.id);
                    break;
                }
            }

            if let Some(comment_id) = comment_id {
                scm.update_merge_request_note(pr_number, comment_id, code_notify_result.message)
                    .await;
            } else {
                if code_notify_result.notify.is_empty() {
                    debug!("not adding a comment because there are no notifications to send");
                } else {
                    scm.create_merge_request_note(pr_number, code_notify_result.message)
                        .await;
                }
            }
        }
        RuntimeEnv::Local(code_notify) => {
            code_notify.notify()?;
        }
    }

    Ok(Some(String::new()))
}

fn setup(cmd: NotifyCommand) -> anyhow::Result<RuntimeEnv> {
    let subscriber_threshold = cmd.subscriber_threshold;
    let file_name = cmd.file_name;

    if let Some(ci_provider) = ContinuousIntegrationProvider::from_env() {
        let ci_context = ci_provider.context_from_env()?;
        if ci_context.draft {
            // TODO: better way to exit program
            debug!("Not sending notifications for draft pull request.");
            std::process::exit(1)
        }

        let scm = match ci_provider.associated_bound_scm_client(&ci_context)? {
            None => anyhow::bail!("unable to determine SCM platform client"),
            Some(scm) => scm,
        };

        // TODO: see if there is a way to remove these clones
        let base_ref = cmd.base_ref.unwrap_or(ci_context.base.clone());
        let head_ref = cmd.head_ref.unwrap_or(ci_context.head.clone());

        Ok(RuntimeEnv::CI((
            CodeNotify {
                cwd: ci_context.build_directory.clone(),
                format: "markdown".to_string(),
                file_name: file_name.clone(),
                subscriber_threshold,
                commit_range: ScmCommitRange(base_ref.clone(), Some(head_ref.clone())),
                author: ci_context.author.clone(),
            },
            ci_context,
            scm,
        )))

        // Ok(CodeNotify {
        //     cwd: ci_context.build_directory,
        //     format: "markdown".to_string(),
        //     file_name,
        //     subscriber_threshold,
        //     commit_range: ScmCommitRange(base_ref, Some(head_ref)),
        //     author: ci_context.author,
        // })
    } else {
        // Ok(CodeNotify {
        //     cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
        //     format: cmd.format.unwrap_or("text".to_string()),
        //     file_name,
        //     subscriber_threshold,
        //     commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
        //     author: None,
        // })
        Ok(RuntimeEnv::Local(CodeNotify {
            cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
            format: cmd.format.unwrap_or("text".to_string()),
            file_name,
            subscriber_threshold,
            commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
            author: None,
        }))
    }
}

async fn comment_on_scm_merge_request(
    scm: Box<dyn ScmPlatformRepositoryBoundedClient>,
    mr: u64,
    comment: String,
) {
    let comments = scm.list_all_merge_requests_notes(mr).await;
}

// fn cli_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
//     Ok(CodeNotify {
//         cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
//         format: cmd.format.unwrap_or("text".to_string()),
//         file_name: cmd.file_name,
//         subscriber_threshold: cmd.subscriber_threshold,
//         commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
//         author: None,
//     })
// }

// fn github_actions_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
//     // https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/store-information-in-variables#default-environment-variables
//     let cwd = match std::env::var("GITHUB_WORKSPACE") {
//         Ok(path) => PathBuf::from(path),
//         Err(_) => {
//             println!("Unable to get GitHub workspace cwd from env var 'GITHUB_WORKSPACE'");
//             // return error or just exit
//             std::process::exit(1)
//         }
//     };
//
//     let event: PullRequestWebhookEventPayload = match std::env::var("GITHUB_EVENT_PATH") {
//         Ok(path) => {
//             // unable to read GitHub event json %s: %s", path, err
//             let data = std::fs::read_to_string(path)?;
//             serde_json::from_str(&data)?
//         }
//         Err(_) => {
//             println!("env var 'GITHUB_EVENT_PATH' not set");
//             // return error or just exit
//             std::process::exit(1)
//         }
//     };
//
//     if event.pull_request.draft.unwrap_or(false) {
//         println!("Not sending notifications for draft pull request");
//         // return error or just exit
//         std::process::exit(1)
//     }
//
//     let subscriber_threshold: i8 =
//         doctavious_std::env::parse("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
//             .unwrap_or_default();
//
//     // TODO: constant?
//     let filename =
//         std::env::var("DOCTAVIOUS_CODENOTIFY_FILENAME").unwrap_or("CODENOTIFY".to_string());
//
//     // TODO: github client
//     // TODO: GITHUB_API_URL
//     // token := os.Getenv("GITHUB_TOKEN")
//     // if token == "" {
//     //     return fmt.Errorf("GITHUB_TOKEN is not set")
//     // }
//     // req.Header.Set("Authorization", "bearer "+token)
//
//     Ok(CodeNotify {
//         cwd,
//         format: "markdown".to_string(),
//         file_name: filename,
//         subscriber_threshold,
//         commit_range: ScmCommitRange(
//             event.pull_request.base.sha,
//             Some(event.pull_request.head.sha),
//         ),
//         author: event
//             .pull_request
//             .user
//             .and_then(|u| Some(format!("@{}", u.login).to_string())),
//     })
// }
//
// fn gitlab_ci_options(cmd: NotifyCommand) -> anyhow::Result<CodeNotify> {
//     let draft = doctavious_std::env::as_boolean("CI_MERGE_REQUEST_DRAFT");
//     if draft {
//         println!("Not sending notifications for draft pull request");
//         // return error or just exit
//         std::process::exit(1)
//     }
//
//     // TODO: Confirm CI_BUILDS_DIR is correct
//     let cwd = std::env::var("CI_BUILDS_DIR")
//         .and_then(|s| Ok(PathBuf::from(s)))
//         .unwrap_or(std::env::current_dir()?);
//     let subscriber_threshold: i8 =
//         doctavious_std::env::parse("DOCTAVIOUS_CODENOTIFY_SUBSCRIBER_THRESHOLD")
//             .unwrap_or_default();
//
//     // TODO: constant?
//     let filename =
//         std::env::var("DOCTAVIOUS_CODENOTIFY_FILENAME").unwrap_or("CODENOTIFY".to_string());
//
//     // TODO: should we return errors rather than defaults and using `ok()`?
//     let base = std::env::var("CI_MERGE_REQUEST_DIFF_BASE_SHA").unwrap_or_default();
//     let head = std::env::var("CI_COMMIT_SHORT_SHA").ok();
//     let author = std::env::var("CI_COMMIT_AUTHOR")
//         .ok()
//         .and_then(|a| a.split_ascii_whitespace().next().map(|s| s.to_string()))
//         .map(|s| format!("@{}", s));
//
//     // TODO: fix this
//     let provider = gitlab::provider::GitLabProvider {
//         client: Arc::new(gitlab_client::client::Client::new("", None)?),
//     };
//
//     // provider.update_merge_request_note(GitlabRepositoryIdentifier {
//     //     project_id: std::env::var("").unwrap()
//     // }).await;
//
//     Ok(CodeNotify {
//         cwd,
//         format: "markdown".to_string(),
//         file_name: filename,
//         subscriber_threshold,
//         commit_range: ScmCommitRange(base, head),
//         author,
//     })
// }
