mod notify;

use clap::{Parser, Subcommand};

use crate::commands::codenotify::notify::NotifyCommand;

// TODO: determine why we need clap Subcommand import in this case
#[derive(Parser, Debug)]
#[command()]
pub struct CodeNotifyCommand {
    #[command(subcommand)]
    sub_commands: CodeNotifySubCommands,
}

#[remain::sorted]
#[derive(Debug, Subcommand)]
enum CodeNotifySubCommands {
    Notify(NotifyCommand),
}

#[async_trait::async_trait]
impl crate::commands::Command for CodeNotifyCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_commands {
            CodeNotifySubCommands::Notify(cmd) => cmd.execute().await,
        }
    }
}
