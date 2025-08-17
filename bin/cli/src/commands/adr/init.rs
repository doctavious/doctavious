use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::file_structure::FileStructure;
use doctavious_cli::settings::DEFAULT_ADR_DIR;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Initialises the directory of architecture decision records:
/// * creates a subdirectory of the current working directory
/// * creates the first ADR in that subdirectory, recording the decision to record architectural decisions with ADRs.
#[derive(Parser, Debug)]
#[command(name = "init")]
pub struct InitADR {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Directory to store ADRs
    #[arg(
        long,
        short,
        default_value=PathBuf::from(DEFAULT_ADR_DIR).into_os_string()
    )]
    pub directory: PathBuf,

    /// How ADRs should be structured
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

#[async_trait::async_trait]
impl crate::commands::Command for InitADR {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = adr::init(
            cwd.as_path(),
            Some(self.directory.clone()),
            self.structure,
            self.format,
        )?;

        Ok(Some(output.to_string_lossy().to_string()))
    }
}
