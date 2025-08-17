use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::{LinkReference, adr};

/// Creates a link between two ADRs, from SOURCE to TARGET
#[derive(Parser, Debug)]
#[command(name = "link")]
pub struct LinkADRs {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Reference number or (partial) filename of source ADR
    #[arg(long, short)]
    pub source: String,

    /// Description of the link created in the new ADR
    #[arg(long, short)]
    pub link: String,

    /// Reference number or (partial) filename of target ADR
    #[arg(long, short)]
    pub target: String,

    /// Description of the link created in the target ADR
    #[arg(long, short)]
    pub reverse_link: String,
}

#[async_trait::async_trait]
impl crate::commands::Command for LinkADRs {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        // let cwd = cmd.cwd.unwrap_or(std::env::current_dir()?);
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let _ = adr::link(
            &cwd,
            LinkReference::from_str(self.source.as_str())?,
            &self.link,
            LinkReference::from_str(self.target.as_str())?,
            &self.reverse_link,
        )?;
        Ok(None)
    }
}
