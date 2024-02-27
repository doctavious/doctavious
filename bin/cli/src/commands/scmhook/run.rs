use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::CliResult;

use crate::commands::scmhook::add::Add;

/// Adds a hook directory
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Run {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Run hooks on specified files, comma-separated
    #[arg(long, short)]
    pub files: Option<Vec<String>>,

    /// Run on specified file (repeat for multiple files). takes precedence over --all-files
    #[arg(long, short)]
    pub file: Option<Vec<String>>,

    /// Run only specified commands
    #[arg(long, short)]
    pub commands: Option<Vec<String>>,

    /// Run hooks on all files
    #[arg(long, short, action)]
    pub all_files: bool,

    /// Force execution of commands that can be skipped
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: Run) -> CliResult<Option<String>> {
    Ok(None)
}
