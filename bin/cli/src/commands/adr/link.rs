use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::{adr, LinkReference};
use doctavious_cli::CliResult;

/// Creates a link between two ADRs, from SOURCE to TARGET
#[derive(Parser, Debug)]
#[command(name = "link")]
pub(crate) struct LinkADRs {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Reference number or (partial) filename of source ADR
    #[arg(long, short)]
    pub source: String,

    /// Description of the link created in the new ADR
    #[arg(long, short)]
    pub link: String,

    /// Reference number or (partial) filename of target ADR
    #[arg(long, short)]
    pub target: String,

    /// Description of the link created in the target ADR
    #[arg(long, short)]
    pub reverse_link: String,
}

pub(crate) fn execute(cmd: LinkADRs) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let _ = adr::link(
        &cwd,
        LinkReference::from_str(cmd.source.as_str())?,
        &cmd.link,
        LinkReference::from_str(cmd.target.as_str())?,
        &cmd.reverse_link,
    )?;
    Ok(Some(String::new()))
}
