mod generate;
mod init;
mod list;
mod new;
mod open;

use clap::Parser;
use doctavious_cli::errors::CliResult;

use crate::commands::til::generate::GenerateTils;
use crate::commands::til::init::InitTil;
use crate::commands::til::list::ListTils;
use crate::commands::til::new::NewTil;
use crate::commands::til::open::OpenTil;

/// Manage Today I Learned (TIL) posts
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct TilCommand {
    #[command(subcommand)]
    pub sub_command: TilSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum TilSubCommand {
    Generate(GenerateTils),
    Init(InitTil),
    List(ListTils),
    New(NewTil),
    Open(OpenTil),
    // TODO: render
    // TODO: template
}

pub(crate) fn execute(command: TilCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        TilSubCommand::Generate(cmd) => generate::execute(cmd),
        TilSubCommand::Init(cmd) => init::execute(cmd),
        TilSubCommand::List(cmd) => list::execute(cmd),
        TilSubCommand::New(cmd) => new::execute(cmd),
        TilSubCommand::Open(cmd) => open::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
