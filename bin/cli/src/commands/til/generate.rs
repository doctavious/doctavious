use clap::builder::PossibleValuesParser;
use clap::Parser;
use doctavious_cli::markup_format::MarkupFormat;
use doctavious_cli::CliResult;

// TODO: flush this out more?
// keeping ToC is probably fine
// but also want to generate CSV
// Generate README / index file
// Update README with table (maybe even list)
#[derive(Parser, Debug)]
pub(crate) enum GenerateCommand {
    // Toc(crate::commands::rfd::generate::RFDToc), // template, csv file. what is the snippet?
    // Csv(crate::commands::rfd::generate::RFDCsv),
    // File(crate::commands::rfd::generate::RFDFile),
    // Atom Feed
}

/// Build TIL ReadMe
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct BuildTilReadMe {
    /// Directory where TILs are stored
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: optional path to template.
    /// Extension that should be used
    #[arg(
        long,
        short,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub format: Option<MarkupFormat>,
}

pub(crate) fn execute(command: BuildTilReadMe) -> CliResult<Option<String>> {
    Ok(Some(String::new()))
}
