use clap::builder::PossibleValuesParser;
use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::file_structure::FileStructure;
use doctavious_cli::markup_format::MarkupFormat;
use doctavious_cli::CliResult;

use crate::commands::adr::init::InitADR;

/// Link ADRs
#[derive(Parser, Debug)]
#[command(name = "link")]
pub(crate) struct LinkADRs {
    /// Reference number of source ADR
    #[arg(long, short)]
    pub source: i32,

    /// Description of the link created in the new ADR
    #[arg(long, short)]
    pub link: String,

    #[arg(long, short, help = "Reference number of target ADR")]
    pub target: i32,

    /// Description of the link created in the existing ADR that will refer to new ADR
    #[arg(long, short)]
    pub reverse_link: String,
}

pub(crate) fn execute(cmd: LinkADRs) -> CliResult<Option<String>> {
    // adr::link()
    Ok(Some(String::new()))
}
