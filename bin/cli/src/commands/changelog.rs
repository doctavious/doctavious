use clap::Parser;

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

#[async_trait::async_trait]
impl crate::commands::Command for ChangelogCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            ChangelogSubCommands::Init(cmd) => cmd.execute().await,
            ChangelogSubCommands::Release(cmd) => cmd.execute().await,
        }
    }
}
