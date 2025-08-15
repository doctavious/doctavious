use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::Args;
use code_ownify::notify::CodeNotify;
use continuous_integration::ContinuousIntegrationProvider;
use scm::commit::ScmCommitRange;
use tracing::debug;

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

pub(crate) async fn execute(cmd: NotifyCommand) -> anyhow::Result<Option<String>> {
    let subscriber_threshold = cmd.subscriber_threshold;
    let file_name = cmd.file_name;

    if let Some(ci_provider) = ContinuousIntegrationProvider::from_env() {
        // running in CI
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
        let base_ref = cmd.base_ref.unwrap_or(ci_context.base);
        let head_ref = cmd.head_ref.unwrap_or(ci_context.head);

        let code_notify = CodeNotify {
            cwd: ci_context.build_directory,
            format: "markdown".to_string(),
            file_name,
            subscriber_threshold,
            commit_range: ScmCommitRange(base_ref, Some(head_ref)),
            author: ci_context.author,
        };

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
    } else {
        // running locally
        let code_notify = CodeNotify {
            cwd: cmd.cwd.unwrap_or(std::env::current_dir()?),
            format: cmd.format.unwrap_or("text".to_string()),
            file_name,
            subscriber_threshold,
            commit_range: ScmCommitRange(cmd.base_ref.unwrap_or_default(), cmd.head_ref),
            author: None,
        };

        let code_notify_result = code_notify.notify()?;
        write!(io::stdout(), "{}", code_notify_result.message)?;
    }

    Ok(Some(String::new()))
}
