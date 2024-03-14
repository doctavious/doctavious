use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::uninstall::uninstall;
use doctavious_cli::CliResult;

/// Clear SCM hooks related to doctavious configuration
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct UninstallScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Flag to remove all SCM hooks even those not related to doctavious
    #[arg(long, short, action)]
    pub force: bool,

    /// Flag to remove SCM hook configuration from doctavious configuration
    #[arg(long, short, action)]
    pub remove_config: bool,
}

pub(crate) fn execute(command: UninstallScmHook) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    uninstall(&path)?;

    Ok(None)
}
