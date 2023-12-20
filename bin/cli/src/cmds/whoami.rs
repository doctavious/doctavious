use clap::{Parser, Subcommand};

/// Show the username of the user currently logged into Doctavious CLI.
#[derive(Parser, Debug)]
pub(crate) struct WhoAmICommand;
