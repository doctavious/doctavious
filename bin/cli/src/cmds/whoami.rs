use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Show the username of the user currently logged into Doctavious CLI.")]
pub(crate) struct WhoAmICommand;
