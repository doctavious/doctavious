use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr::generate_toc;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Gathers generate ADR commands
#[derive(Parser, Debug)]
#[command()]
pub struct GenerateADRs {
    #[command(subcommand)]
    pub sub_command: GenerateAdrsCommand,
}

#[derive(Parser, Debug)]
pub enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

#[async_trait::async_trait]
impl crate::commands::Command for GenerateADRs {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            GenerateAdrsCommand::Toc(cmd) => cmd.execute().await,
            GenerateAdrsCommand::Graph(_) => unimplemented!(),
        }?;

        Ok(Some(String::new()))
    }
}

/// Generates ADR table of contents (Toc) to stdout
#[derive(Parser, Debug)]
#[command()]
pub struct AdrToc {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Precede the table of contents with the given text.
    #[arg(long, short)]
    pub intro: Option<String>,

    /// Follow the table of contents with the given text.
    #[arg(long)]
    pub outro: Option<String>,

    /// Prefix each decision file link with given text.
    #[arg(long, short)]
    pub link_prefix: Option<String>,

    /// Output format
    #[arg(
        long,
        short,
        default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: MarkupFormat,
}

#[async_trait::async_trait]
impl crate::commands::Command for AdrToc {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = generate_toc(
            &cwd,
            self.format,
            self.intro.as_deref(),
            self.outro.as_deref(),
            self.link_prefix.as_deref(),
        )?;

        Ok(Some(output))
    }
}

/// Create ADR Graph
#[derive(Parser, Debug)]
#[command()]
pub struct AdrGraph {
    /// Directory of ADRs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short)]
    pub link_extension: Option<String>,

    #[arg(long, short)]
    pub link_prefix: Option<String>,
}
