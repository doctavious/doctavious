use std::path::PathBuf;

use clap::Parser;
use clap::builder::PossibleValuesParser;
use doctavious_cli::cmd::til;
use doctavious_cli::errors::CliResult;
use markup::MarkupFormat;

// TODO: flush this out more?
// keeping ToC is probably fine
// but also want to generate CSV
// Generate README / index file
// Update README with table (maybe even list)
#[derive(Parser, Debug)]
#[command()]
pub struct GenerateTils {
    // Toc(crate::commands::rfd::generate::RFDToc), // template, csv file. what is the snippet?
    // Csv(crate::commands::rfd::generate::RFDCsv),
    // File(crate::commands::rfd::generate::RFDFile),
    // Atom Feed
    #[command(subcommand)]
    pub sub_command: GenerateTilsCommand,
}

#[derive(Parser, Debug)]
pub enum GenerateTilsCommand {
    Toc(TilToc),
}

/// Build TIL ReadMe
#[derive(Parser, Debug)]
#[command()]
pub struct TilToc {
    /// Directory where TILs are stored
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    pub destination: Option<PathBuf>,

    // TODO: optional path to template.
    /// Extension that should be used
    #[arg(
        long,
        short,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub format: Option<MarkupFormat>,
}

pub fn execute(command: GenerateTils) -> CliResult<Option<String>> {
    match command.sub_command {
        GenerateTilsCommand::Toc(cmd) => execute_generate_toc(cmd),
    }
}

pub fn execute_generate_toc(cmd: TilToc) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);

    til::generate_toc(cwd.as_path(), cmd.format.unwrap_or_default())?;

    Ok(Some(String::new()))
}
