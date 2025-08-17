use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::frameworks;

#[derive(Parser, Debug)]
#[command(about = "")]
pub struct FrameworksCommand {
    #[command(subcommand)]
    pub sub_command: FrameworkSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub enum FrameworkSubCommand {
    Detect(DetectFrameworks),
    Get(GetFramework),
    List(ListFrameworks),
}

#[async_trait::async_trait]
impl crate::commands::Command for FrameworksCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            FrameworkSubCommand::Detect(cmd) => cmd.execute().await,
            FrameworkSubCommand::Get(cmd) => cmd.execute().await,
            FrameworkSubCommand::List(cmd) => cmd.execute().await,
        }
    }
}

#[derive(Parser, Debug)]
#[command(about = "Detect Frameworks")]
pub struct DetectFrameworks {
    #[arg(long, short, help = "Directory to detect framewoks in")]
    pub cwd: Option<PathBuf>,
}

#[async_trait::async_trait]
impl crate::commands::Command for DetectFrameworks {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        frameworks::detect::execute(self.cwd.clone())?;
        Ok(None)
    }
}

#[derive(Parser, Debug)]
#[command(about = "List Frameworks")]
pub struct ListFrameworks {}

#[async_trait::async_trait]
impl crate::commands::Command for ListFrameworks {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        frameworks::list::execute()?;
        Ok(None)
    }
}

#[derive(Parser, Debug)]
#[command(about = "Get Framework Details")]
pub struct GetFramework {
    #[arg(long, short, help = "Name of the framework")]
    pub name: String,
}

#[async_trait::async_trait]
impl crate::commands::Command for GetFramework {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        frameworks::get::execute(self.name.clone())?;
        Ok(None)
    }
}
