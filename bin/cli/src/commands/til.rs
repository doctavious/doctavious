mod generate;
mod init;
mod list;
mod new;
mod open;

use clap::Parser;

use crate::commands::til::generate::GenerateTils;
use crate::commands::til::init::InitTil;
use crate::commands::til::list::ListTils;
use crate::commands::til::new::NewTil;
use crate::commands::til::open::OpenTil;

/// Manage Today I Learned (TIL) posts
#[derive(Parser, Debug)]
#[command()]
pub struct TilCommand {
    #[command(subcommand)]
    pub sub_command: TilSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub enum TilSubCommand {
    Generate(GenerateTils),
    Init(InitTil),
    List(ListTils),
    New(NewTil),
    Open(OpenTil),
    // TODO: render
    // TODO: template
}

#[async_trait::async_trait]
impl crate::commands::Command for TilCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            TilSubCommand::Generate(cmd) => cmd.execute().await,
            TilSubCommand::Init(cmd) => cmd.execute().await,
            TilSubCommand::List(cmd) => cmd.execute().await,
            TilSubCommand::New(cmd) => cmd.execute().await,
            TilSubCommand::Open(cmd) => cmd.execute().await,
        }
    }
}
