use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Reserve ADR
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub struct ReserveADR {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// ADR Number
    #[arg(long, short)]
    pub number: Option<u32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of ADR
    #[arg(long, short)]
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
impl crate::commands::Command for ReserveADR {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let _ = adr::reserve(&cwd, self.number, self.title.clone(), self.format)?;
        Ok(None)
    }
}
