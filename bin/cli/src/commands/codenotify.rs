mod notify;

use clap::{Args, Parser, Subcommand};
use doctavious_cli::errors::{CliResult, DoctaviousCliError};

use crate::commands::codenotify::notify::NotifyCommand;

#[derive(Parser, Debug)]
#[command()]
pub(crate) struct CodeNotifyCli {
    #[command(subcommand)]
    command: Commands,
}

#[remain::sorted]
#[derive(Debug, Subcommand)]
enum Commands {
    Notify(NotifyCommand),
}

pub(crate) async fn execute(cli: CodeNotifyCli) -> CliResult<Option<String>> {
    // TODO: map_err is a hack. need to determine if I want to use anyhow through this stack
    match cli.command {
        Commands::Notify(cmd) => notify::execute(cmd).await,
    }
    .map_err(|e| DoctaviousCliError::GeneralError(e.to_string()))?;

    Ok(Some(String::new()))
}
