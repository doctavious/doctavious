use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til::list;

/// List TILs
#[derive(Parser, Debug)]
#[command()]
pub struct ListTils {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl crate::commands::Command for ListTils {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = list(&cwd)?;
        Ok(Some(
            output
                .iter()
                .map(|p| p.to_string_lossy())
                .collect::<Vec<_>>()
                .join("\n"),
        ))
    }
}
