use std::path::PathBuf;

use clap::Parser;
use clap::builder::PossibleValuesParser;
use doctavious_cli::cmd::til;
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

#[async_trait::async_trait]
impl crate::commands::Command for GenerateTils {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            GenerateTilsCommand::Toc(cmd) => cmd.execute().await,
        }
    }
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

#[async_trait::async_trait]
impl crate::commands::Command for TilToc {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        til::generate_toc(cwd.as_path(), self.format.unwrap_or_default())?;
        Ok(None)
    }
}
