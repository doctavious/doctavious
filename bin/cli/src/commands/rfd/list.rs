use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd::list;
use doctavious_cli::errors::CliResult;
use markup::MarkupFormat;

/// List RFDs
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ListRFDs {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

pub(crate) fn execute(cmd: ListRFDs) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let output = list(&cwd, MarkupFormat::Markdown)?;
    Ok(Some(
        output
            .iter()
            .map(|p| p.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n"),
    ))
}
