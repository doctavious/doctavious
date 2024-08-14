use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd;
use doctavious_cli::errors::CliResult;
use strum::VariantNames;
use markup::MarkupFormat;
use crate::clap_enum_variants;

/// Reserve RFD
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub(crate) struct ReserveRFD {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// RFD Number
    #[arg(long, short)]
    pub number: Option<u32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of RFD
    #[arg(long, short)]
    pub title: String,

    /// Extension that should be used
    #[arg(
        long,
        short,
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,
}

pub(crate) fn execute(cmd: ReserveRFD) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let _ = rfd::reserve(&cwd, cmd.number, cmd.title, cmd.format)?;
    Ok(Some(String::new()))
}
