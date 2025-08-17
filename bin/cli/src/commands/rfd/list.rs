use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::rfd::list;
use markup::MarkupFormat;

/// List RFDs
#[derive(Parser, Debug)]
#[command()]
pub struct ListRFDs {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl crate::commands::Command for ListRFDs {
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
