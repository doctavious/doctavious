use std::path::PathBuf;

use clap::Parser;
use clap::builder::PossibleValuesParser;
use markup::MarkupFormat;

/// Gathers generate RFD commands
#[derive(Parser, Debug)]
#[command()]
pub struct GenerateRFDs {
    #[command(subcommand)]
    pub sub_command: GenerateRFDsCommand,
}

// TODO: flush this out more?
// keeping ToC is probably fine
// but also want to generate CSV
// Generate README / index file
// Update README with table (maybe even list)
#[derive(Parser, Debug)]
pub enum GenerateRFDsCommand {
    Toc(RFDToc), // template, csv file. what is the snippet?
    Csv(RFDCsv),
    File(RFDFile),
    // TODO: CSV - path, if exists just update. What about supporting it in another branch/remote. what about committing to that branch? flag for commit and commit message?
    // TODO: File - // template and path to where file should be created
    Graph(RFDGraph),
}

#[async_trait::async_trait]
impl crate::commands::Command for GenerateRFDs {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            GenerateRFDsCommand::Toc(cmd) => cmd.execute().await,
            GenerateRFDsCommand::Csv(cmd) => cmd.execute().await,
            GenerateRFDsCommand::File(cmd) => cmd.execute().await,
            GenerateRFDsCommand::Graph(cmd) => cmd.execute().await,
        }
    }
}

// optional file means to stdout
// add overwrite flag to not modify existing
// remote? commit message?
/// Generates RFD CSV
#[derive(Parser, Debug)]
#[command()]
pub struct RFDCsv {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    // #[clap(parse(from_os_str)] -> #[clap(value_parser)]
    // output_path
    #[arg(value_parser, long, short, help = "")]
    pub path: Option<PathBuf>, // where to write file to. stdout if not provided

    #[arg(long, short, help = "")]
    pub fields: Vec<String>, // which fields to include? default to all (*). should this just be a comma separate list?

    #[arg(long, short, help = "")]
    pub overwrite: bool,
}

#[async_trait::async_trait]
impl crate::commands::Command for RFDCsv {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}

/// Generates RFD File
#[derive(Parser, Debug)]
#[command()]
pub struct RFDFile {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// Template that will be used to generate file.
    /// If not present use value from config otherwise default template based on output_path extension
    /// will be used. See <location> for default template
    #[arg(long, short)]
    pub template: Option<String>, // optional. use config, use provided here. use default

    // output_path
    /// Path to file which to write table of contents to. File must contain snippet.
    /// If not present ToC will be written to stdout
    #[arg(long, short, value_parser)]
    pub path: PathBuf, // where to write file to. required
}

#[async_trait::async_trait]
impl crate::commands::Command for RFDFile {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}

/// Generates RFD table of contents (Toc) to stdout
#[derive(Parser, Debug)]
#[command()]
pub struct RFDToc {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// Template that will be used to generate file.
    /// If not present use value from config otherwise default template based on
    /// output_path extension will be used. See <location> for default template
    #[arg(long, short)]
    pub template: Option<String>, // optional. use config, use provided here. use default

    /// Path to file which to write table of contents to. File must contain snippet.
    /// If not present ToC will be written to stdout
    #[arg(long, short, value_parser)]
    pub output_path: PathBuf, // where to write file to. required

    #[arg(long, short)]
    pub intro: Option<String>,

    #[arg(long)]
    pub outro: Option<String>,

    #[arg(long, short)]
    pub link_prefix: Option<String>,

    /// Output format
    #[arg(
        long,
        short,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub format: Option<MarkupFormat>,
}

#[async_trait::async_trait]
impl crate::commands::Command for RFDToc {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}

/// Create RFD Graph
#[derive(Parser, Debug)]
#[command()]
pub struct RFDGraph {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short)]
    pub link_extension: Option<String>,

    #[arg(long, short)]
    pub link_prefix: Option<String>,
}

#[async_trait::async_trait]
impl crate::commands::Command for RFDGraph {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}
