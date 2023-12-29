use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::markup_format::MarkupFormat;
use doctavious_cli::CliResult;
use strum::VariantNames;

use crate::clap_enum_variants;

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
/// New ADR
#[derive(Parser, Debug)]
#[command(name = "new")]
pub(crate) struct NewADR {
    /// ADR Number
    #[arg(long, short)]
    pub number: Option<i32>,

    /// title of ADR
    #[arg(long, short)]
    pub title: String,

    /// Format that should be used
    #[arg(
        long,
        short,
        default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: MarkupFormat,

    /// A reference (number or partial filename) of a previous decision that the new decision supersedes.
    /// A Markdown link to the superseded ADR is inserted into the Status section.
    /// The status of the superseded ADR is changed to record that it has been superseded by the new ADR.
    #[arg(long, short)]
    pub supersede: Option<Vec<String>>,

    // Links the new ADR to a previous ADR.
    // TARGET is a reference (number or partial filename) of a
    // previous decision.
    // LINK is the description of the link created in the new ADR.
    // REVERSE-LINK is the description of the link created in the
    // existing ADR that will refer to the new ADR.
    #[arg(long, short)]
    pub link: Option<Vec<String>>,
}

pub(crate) fn execute(cmd: NewADR) -> CliResult<Option<String>> {
    // adr::new(cmd)

    Ok(Some(String::new()))
}
