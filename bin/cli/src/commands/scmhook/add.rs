use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::add::add;
use doctavious_cli::CliResult;

/// Adds a hook directory
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AddScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    #[arg(long, short)]
    pub name: Option<String>,

    /// Create a directory for scripts
    #[arg(long, short)]
    pub dir: Option<PathBuf>,

    /// Overwrite .old hooks
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: AddScmHook) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    add(&path, command.name.unwrap_or_default())?;

    Ok(None)
}
