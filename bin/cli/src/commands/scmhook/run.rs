use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::run::run;
use doctavious_cli::CliResult;

/// Execute commands/scripts associated to the specified hook.
///
/// This is called for every hook managed by doctavious.
/// You can also provide your own hooks that can only be called manually.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RunScmHookCommand {
    /// Name of the hook to run
    #[arg(index = 1)]
    pub hook: String,

    /// Path to execute run
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    // TODO: can use group = "files" to only allow one to be used?
    /// Run on specified file (repeat for multiple files). takes precedence over --all-files
    #[arg(long, short)]
    pub file: Option<Vec<PathBuf>>,

    /// Run hooks on all files
    #[arg(long, short, action)]
    pub all_files: bool,

    /// Run only specified executions (commands / scripts)
    #[arg(long = "executions", short = 'e')]
    pub run_only_executions: Option<Vec<String>>,

    /// Force execution of commands that can be skipped
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: RunScmHookCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    // TODO: turn all_files / files into an enum

    run(
        &path,
        &command.hook,
        command.all_files,
        command.file.unwrap_or_default(),
        command.run_only_executions.unwrap_or_default(),
        command.force,
    )?;

    Ok(None)
}
