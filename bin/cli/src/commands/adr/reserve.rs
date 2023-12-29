use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::markup_format::MarkupFormat;
use doctavious_cli::CliResult;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Reserve ADR
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub(crate) struct ReserveADR {
    /// ADR Number
    #[arg(long, short)]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
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
}

pub(crate) fn execute(cmd: ReserveADR) -> CliResult<Option<String>> {
    // adr::reserve()
    Ok(Some(String::new()))
}
