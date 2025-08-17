use clap::Parser;

use crate::commands::scmhook::add::AddScmHook;
use crate::commands::scmhook::install::InstallScmHook;
use crate::commands::scmhook::run::RunScmHookCommand;
use crate::commands::scmhook::uninstall::UninstallScmHook;

mod add;
mod install;
mod run;
mod uninstall;

/// Manage SCM Hooks
#[derive(Parser, Debug)]
#[command()]
pub struct ScmHookCommand {
    #[command(subcommand)]
    pub sub_command: ScmHookSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub enum ScmHookSubCommand {
    Add(AddScmHook),
    Install(InstallScmHook),
    Run(RunScmHookCommand),
    Uninstall(UninstallScmHook),
}

#[async_trait::async_trait]
impl crate::commands::Command for ScmHookCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            ScmHookSubCommand::Add(cmd) => cmd.execute().await,
            ScmHookSubCommand::Install(cmd) => cmd.execute().await,
            ScmHookSubCommand::Run(cmd) => cmd.execute().await,
            ScmHookSubCommand::Uninstall(cmd) => cmd.execute().await,
        }
    }
}
