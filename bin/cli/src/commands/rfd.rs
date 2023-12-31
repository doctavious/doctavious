mod generate;
mod init;
mod list;
mod new;
mod reserve;

use clap::{Parser, Subcommand};
use doctavious_cli::CliResult;

use crate::commands::rfd::generate::GenerateRFDs;
use crate::commands::rfd::init::InitRFD;
use crate::commands::rfd::list::ListRFDs;
use crate::commands::rfd::new::NewRFD;
use crate::commands::rfd::reserve::ReserveRFD;

#[derive(Parser, Debug)]
#[command(about = "Gathers RFD management commands")]
pub(crate) struct RFDCommand {
    #[command(subcommand)]
    pub sub_command: RFDSubCommand,
}

#[remain::sorted]
#[derive(Subcommand, Debug)]
pub(crate) enum RFDSubCommand {
    Generate(GenerateRFDs),
    Init(InitRFD),
    List(ListRFDs),
    New(NewRFD),
    Reserve(ReserveRFD),
    // Templates (add/delete. global vs local)
}

pub(crate) fn execute(command: RFDCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        RFDSubCommand::Init(cmd) => init::execute(cmd),
        RFDSubCommand::Generate(cmd) => generate::execute(cmd),
        RFDSubCommand::List(cmd) => list::execute(cmd),
        RFDSubCommand::New(cmd) => new::execute(cmd),
        RFDSubCommand::Reserve(cmd) => reserve::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
