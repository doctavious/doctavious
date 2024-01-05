mod generate;
mod init;
mod list;
mod new;
mod open;

use clap::Parser;

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
