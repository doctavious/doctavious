use clap::Parser;
use doctavious_cli::CliResult;

use crate::commands::scmhook::add::AddScmHook;
use crate::commands::scmhook::install::InstallScmHook;
use crate::commands::scmhook::run::RunScmHookCommand;
use crate::commands::scmhook::uninstall::UninstallScmHook;

mod add;
mod install;
mod run;
mod runner;
mod uninstall;

/// Manage SCM Hooks
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ScmHookCommand {
    #[command(subcommand)]
    pub sub_command: ScmHookSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum ScmHookSubCommand {
    Add(AddScmHook),
    Install(InstallScmHook),
    Run(RunScmHookCommand),
    Uninstall(UninstallScmHook),
}

pub(crate) fn execute(command: ScmHookCommand) -> CliResult<Option<String>> {
    match command.sub_command {
        ScmHookSubCommand::Add(cmd) => add::execute(cmd),
        ScmHookSubCommand::Install(cmd) => install::execute(cmd),
        ScmHookSubCommand::Run(cmd) => run::execute(cmd),
        ScmHookSubCommand::Uninstall(cmd) => uninstall::execute(cmd),
    }?;

    Ok(Some(String::new()))
}
