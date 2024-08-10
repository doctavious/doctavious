use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr::generate_toc;
use doctavious_cli::errors::CliResult;
use doctavious_cli::markup_format::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Gathers generate ADR commands
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct GenerateADRs {
    #[command(subcommand)]
    pub sub_command: GenerateAdrsCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

/// Generates ADR table of contents (Toc) to stdout
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AdrToc {
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

/// Create ADR Graph
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AdrGraph {
    /// Directory of ADRs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short)]
    pub link_extension: Option<String>,

    #[arg(long, short)]
    pub link_prefix: Option<String>,
}

pub(crate) fn execute(command: GenerateADRs) -> CliResult<Option<String>> {
    match command.sub_command {
        GenerateAdrsCommand::Toc(cmd) => execute_generate_toc(cmd),
        GenerateAdrsCommand::Graph(_) => unimplemented!(),
    }
}

fn execute_generate_toc(cmd: AdrToc) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let output = generate_toc(
        &cwd,
        cmd.format,
        cmd.intro.as_deref(),
        cmd.outro.as_deref(),
        cmd.link_prefix.as_deref(),
    )?;

    Ok(Some(output))
}
