use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub struct InitCommand {
    pub cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl crate::commands::Command for InitCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {}
