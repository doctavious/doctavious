use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til::list;
use doctavious_cli::errors::CliResult;

/// List TILs
#[derive(Parser, Debug)]
#[command()]
pub struct ListTils {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

pub fn execute(cmd: ListTils) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let output = list(&cwd)?;
    Ok(Some(
        output
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n"),
    ))
}
