mod generate;
mod init;
mod list;
mod new;
mod reserve;

use clap::{Parser, Subcommand};

use crate::commands::rfd::generate::GenerateRFDs;
use crate::commands::rfd::init::InitRFD;
use crate::commands::rfd::list::ListRFDs;
use crate::commands::rfd::new::NewRFD;
use crate::commands::rfd::reserve::ReserveRFD;

/// Manage RFDs
#[derive(Parser, Debug)]
#[command()]
pub struct RFDCommand {
    #[command(subcommand)]
    pub sub_command: RFDSubCommand,
}

#[remain::sorted]
#[derive(Subcommand, Debug)]
pub enum RFDSubCommand {
    Generate(GenerateRFDs),
    Init(InitRFD),
    List(ListRFDs),
    New(NewRFD),
    // TODO: render
    Reserve(ReserveRFD),
    // TODO: Templates (add/delete. global vs local)
}

#[async_trait::async_trait]
impl crate::commands::Command for RFDCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            RFDSubCommand::Generate(cmd) => cmd.execute().await,
            RFDSubCommand::Init(cmd) => cmd.execute().await,
            RFDSubCommand::List(cmd) => cmd.execute().await,
            RFDSubCommand::New(cmd) => cmd.execute().await,
            RFDSubCommand::Reserve(cmd) => cmd.execute().await,
        }
    }
}
