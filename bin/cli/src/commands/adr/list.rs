use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr::list;
use markup::MarkupFormat;

/// List ADRs
#[derive(Parser, Debug)]
#[command(name = "list")]
pub struct ListADRs {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl crate::commands::Command for ListADRs {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = list(&cwd, MarkupFormat::Markdown)?;
        Ok(Some(
            output
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}
