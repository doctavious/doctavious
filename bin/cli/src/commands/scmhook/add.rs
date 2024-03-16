use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::add::add;
use doctavious_cli::CliResult;

/// Create a SCM Hook.
///
/// Similar to what `scmhook install` command does but doesn't creating a configuration first.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AddScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// SCM Hook name
    #[arg(index = 1)]
    pub name: String,

    /// Whether to create a directory for scripts
    #[arg(long, short, action)]
    pub dir: bool,

    /// Overwrite .old hooks
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: AddScmHook) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    add(&path, command.name, command.dir, command.force)?;

    Ok(None)
}
