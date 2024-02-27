use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::CliResult;

/// Adds a hook directory
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Add {
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

pub(crate) fn execute(command: Add) -> CliResult<Option<String>> {
    Ok(None)
}
