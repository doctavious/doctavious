mod generate;
mod init;
mod list;
mod new;
mod open;

use clap::Parser;
use doctavious_cli::CliResult;

use crate::commands::til::generate::BuildTilReadMe;
use crate::commands::til::init::InitTil;
use crate::commands::til::list::ListTils;
use crate::commands::til::new::NewTil;

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
    Init(InitTil),
    // #[clap(aliases = &["baz", "fizz"])]
    List(ListTils),
    New(NewTil),
    Readme(BuildTilReadMe),
    // TODO: render
    // TODO: open
    // TODO: template
}


pub(crate) fn execute(command: TilCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        TilSubCommand::Init(cmd) => init::execute(cmd),
        TilSubCommand::List(cmd) => list::execute(cmd),
        TilSubCommand::New(cmd) => new::execute(cmd),
        TilSubCommand::Readme(cmd) => generate::execute(cmd),
    }?;

    Ok(Some(String::new()))
}