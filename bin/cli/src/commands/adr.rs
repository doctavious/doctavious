mod generate;
mod init;
mod link;
mod list;
mod new;
mod reserve;

use clap::Parser;

use crate::commands::adr::generate::GenerateADRs;
use crate::commands::adr::init::InitADR;
use crate::commands::adr::link::LinkADRs;
use crate::commands::adr::list::ListADRs;
use crate::commands::adr::new::NewADR;
use crate::commands::adr::reserve::ReserveADR;

/// Manage ADRs
#[derive(Parser, Debug)]
#[command()]
pub struct ADRCommand {
    #[command(subcommand)]
    pub sub_command: ADRSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub enum ADRSubCommand {
    Generate(GenerateADRs),
    Init(InitADR),
    Link(LinkADRs),
    List(ListADRs),
    New(NewADR),
    // TODO: Render
    Reserve(ReserveADR),
    // TODO: Templates (add/delete. global vs local)
}

#[async_trait::async_trait]
impl crate::commands::Command for ADRCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            ADRSubCommand::Init(cmd) => cmd.execute().await,
            ADRSubCommand::Generate(cmd) => cmd.execute().await,
            ADRSubCommand::List(cmd) => cmd.execute().await,
            ADRSubCommand::Link(cmd) => cmd.execute().await,
            ADRSubCommand::New(cmd) => cmd.execute().await,
            ADRSubCommand::Reserve(cmd) => cmd.execute().await,
        }
    }
}
