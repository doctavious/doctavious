use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd;
use doctavious_cli::errors::CliResult;
use strum::VariantNames;
use markup::MarkupFormat;
use crate::clap_enum_variants;

/// New RFD
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct NewRFD {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// RFD number
    #[arg(long, short)]
    pub number: Option<u32>,

    /// title of RFD
    #[arg(long, short, index = 0)]
    pub title: String,

    /// Format that should be used
    #[arg(
        long,
        short,
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,
}

pub(crate) fn execute(cmd: NewRFD) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let output = rfd::new(&cwd, cmd.number, cmd.title.as_str(), cmd.format)?;

    Ok(Some(output.to_string_lossy().to_string()))
}
