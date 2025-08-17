use clap::Parser;
use doctavious_cli::errors::CliResult;

use crate::commands::changelog::init::InitCommand;
use crate::commands::changelog::release::ReleaseCommand;

pub mod init;
pub mod release;

/// Manage SCM Hooks
#[derive(Parser, Debug)]
#[command()]
pub struct ChangelogCommand {
    #[command(subcommand)]
    pub sub_command: ChangelogSubCommands,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub enum ChangelogSubCommands {
    Init(InitCommand),
    Release(ReleaseCommand),
}

pub fn execute(command: ChangelogCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        ChangelogSubCommands::Init(cmd) => init::execute(cmd),
        ChangelogSubCommands::Release(cmd) => release::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
