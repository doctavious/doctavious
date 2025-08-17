use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd;
use doctavious_cli::file_structure::FileStructure;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Init RFD
#[derive(Parser, Debug)]
#[command()]
pub struct InitRFD {
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

#[async_trait::async_trait]
impl crate::commands::Command for InitRFD {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = rfd::init(
            cwd.as_path(),
            Some(self.directory.clone()),
            self.structure,
            self.format,
        )?;

        Ok(Some(output.to_string_lossy().to_string()))
    }
}
