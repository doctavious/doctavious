use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::install::install;
use doctavious_cli::CliResult;

/// Synchronize SCM hooks with your configuration.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct InstallScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

pub(crate) fn execute(command: InstallScmHook) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    install(&path)?;

    Ok(None)
}
