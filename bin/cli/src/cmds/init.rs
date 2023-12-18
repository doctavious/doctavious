use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Initialize Doctavious Projects locally")]
pub(crate) struct InitCommand {
    #[arg(help = "Name of the Project", index = 0)]
    pub name: Option<String>,
}
