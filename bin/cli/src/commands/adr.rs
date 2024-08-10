mod generate;
mod init;
mod link;
mod list;
mod new;
mod reserve;

use clap::Parser;
use doctavious_cli::errors::CliResult;

use crate::commands::adr::generate::GenerateADRs;
use crate::commands::adr::init::InitADR;
use crate::commands::adr::link::LinkADRs;
use crate::commands::adr::list::ListADRs;
use crate::commands::adr::new::NewADR;
use crate::commands::adr::reserve::ReserveADR;

/// Manage ADRs
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ADRCommand {
    #[command(subcommand)]
    pub sub_command: ADRSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum ADRSubCommand {
    Generate(GenerateADRs),
    Init(InitADR),
    Link(LinkADRs),
    List(ListADRs),
    New(NewADR),
    // TODO: Render
    Reserve(ReserveADR),
    // TODO: Templates (add/delete. global vs local)
}

pub(crate) fn execute(command: ADRCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        ADRSubCommand::Init(cmd) => init::execute(cmd),
        ADRSubCommand::Generate(cmd) => generate::execute(cmd),
        ADRSubCommand::List(cmd) => list::execute(cmd),
        ADRSubCommand::Link(cmd) => link::execute(cmd),
        ADRSubCommand::New(cmd) => new::execute(cmd),
        ADRSubCommand::Reserve(cmd) => reserve::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
