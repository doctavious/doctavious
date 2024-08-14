use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd;
use doctavious_cli::errors::CliResult;
use doctavious_cli::file_structure::FileStructure;
use strum::VariantNames;
use markup::MarkupFormat;
use crate::clap_enum_variants;

/// Init RFD
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct InitRFD {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Directory to store RFDs
    #[arg(long, short)]
    pub directory: PathBuf,

    /// How RFDs should be structured
    #[arg(
        long,
        short,
        default_value_t = FileStructure::default(),
        value_parser = clap_enum_variants!(FileStructure)
    )]
    pub structure: FileStructure,

    /// Format that should be used
    #[arg(
        long,
        short,
        default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: MarkupFormat,
}

pub(crate) fn execute(cmd: InitRFD) -> CliResult<Option<String>> {
    let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
    let output = rfd::init(
        cwd.as_path(),
        Some(cmd.directory),
        cmd.structure,
        cmd.format,
    )?;

    Ok(Some(output.to_string_lossy().to_string()))
}
