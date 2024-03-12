use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::run::run;
use doctavious_cli::CliResult;

/// Adds a hook directory
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct Run {

    /// Name of the hook to run
    #[arg(index = 1)]
    pub name: String,

    /// Path to execute run
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
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    run(
        &path,
        &command.name,
        command.all_files,
        command.files.unwrap_or_default(),
        command.commands,
        command.force,
    )?;

    Ok(None)
}
