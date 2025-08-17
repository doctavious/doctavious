use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// New RFD
#[derive(Parser, Debug)]
#[command()]
pub struct NewRFD {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// RFD number
    #[arg(long, short)]
    pub number: Option<u32>,

    /// title of RFD
    #[arg(long, short, index = 0)]
    pub title: String,

    /// Format that should be used
    #[arg(
        long,
        short,
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,
}

#[async_trait::async_trait]
impl crate::commands::Command for NewRFD {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = rfd::new(&cwd, self.number, self.title.as_str(), self.format)?;

        Ok(Some(output.to_string_lossy().to_string()))
    }
}
