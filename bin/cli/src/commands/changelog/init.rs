use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::CliResult;

#[derive(Parser, Debug)]
#[command()]
pub(crate) struct InitCommand {
    pub cwd: Option<PathBuf>,
}

pub(crate) fn execute(command: InitCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    Ok(None)
}

#[cfg(test)]
mod tests {}
