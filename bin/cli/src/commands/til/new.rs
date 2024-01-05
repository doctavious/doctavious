use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til;
use doctavious_cli::markup_format::MarkupFormat;
use doctavious_cli::CliResult;
use strum::VariantNames;

use crate::clap_enum_variants;

/// New TIL
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct NewTil {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// TIL category. Represents the directory to place TIL entry under
    #[arg(short, long)]
    pub category: String,

    /// title of the TIL entry
    #[arg(long, short)]
    pub title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    /// Additional tags associated with the TIL entry
    #[arg(short = 'T', long)]
    pub tags: Option<Vec<String>>,

    /// File name that should be used.
    /// If extension is included will take precedence over extension argument and configuration file.
    #[arg(long, short)]
    pub file_name: Option<String>,

    /// Format that should be used. This overrides value from configuration file.
    #[arg(
        long,
        short,
        // default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,

    // TODO: should this also be a setting in TilSettings?
    /// Whether to build_mod a README after a new TIL is added
    #[arg(short, long)]
    pub readme: bool,
}

pub(crate) fn execute(cmd: NewTil) -> CliResult<Option<String>> {
    let output = til::new(
        cmd.cwd.as_deref(),
        cmd.title,
        cmd.category,
        cmd.tags,
        cmd.file_name,
        cmd.format,
        cmd.readme,
    )?;

    Ok(Some(output.to_string_lossy().to_string()))
}
