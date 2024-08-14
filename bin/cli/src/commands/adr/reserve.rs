use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::errors::CliResult;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Reserve ADR
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub(crate) struct ReserveADR {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// ADR Number
    #[arg(long, short)]
    pub number: Option<u32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of ADR
    #[arg(long, short)]
    pub title: String,

    /// Format that should be used
    #[arg(
        long,
        short,
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,
}

pub(crate) fn execute(cmd: ReserveADR) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let _ = adr::reserve(&cwd, cmd.number, cmd.title, cmd.format)?;
    Ok(Some(String::new()))
}
