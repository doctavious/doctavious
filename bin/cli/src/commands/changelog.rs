use clap::Parser;
use doctavious_cli::CliResult;

use crate::commands::changelog::init::InitCommand;
use crate::commands::changelog::release::ReleaseCommand;

pub mod init;
pub mod release;

/// Manage SCM Hooks
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ChangelogCommand {
    #[command(subcommand)]
    pub sub_command: ChangelogSubCommands,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum ChangelogSubCommands {
    Init(InitCommand),
    Release(ReleaseCommand),
}

pub(crate) fn execute(command: ChangelogCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        ChangelogSubCommands::Init(cmd) => init::execute(cmd),
        ChangelogSubCommands::Release(cmd) => release::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
