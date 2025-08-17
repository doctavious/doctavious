use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::errors::CliResult;

#[derive(Parser, Debug)]
#[command()]
pub struct InitCommand {
    pub cwd: Option<PathBuf>,
}

pub fn execute(command: InitCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    Ok(None)
}

#[cfg(test)]
mod tests {}
