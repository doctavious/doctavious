use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "")]
pub struct FrameworksCommand {
    #[command(subcommand)]
    pub framework_command: FrameworkSubCommand,
}

#[derive(Parser, Debug)]
pub enum FrameworkSubCommand {
    Detect(DetectFrameworks),
    Get(GetFramework),
    List(ListFrameworks),
}

#[derive(Parser, Debug)]
#[command(about = "Detect Frameworks")]
pub struct DetectFrameworks {
    #[arg(long, short, help = "Directory to detect framewoks in")]
    pub cwd: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(about = "List Frameworks")]
pub struct ListFrameworks {}

#[derive(Parser, Debug)]
#[command(about = "Get Framework Details")]
pub struct GetFramework {
    #[arg(long, short, help = "Name of the framework")]
    pub name: String,
}
