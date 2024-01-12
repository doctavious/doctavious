use clap::{Parser, Subcommand};

use crate::commands::adr::ADRCommand;
use crate::commands::build::BuildCommand;
use crate::commands::deploy::DeployCommand;
use crate::commands::frameworks::FrameworksCommand;
use crate::commands::init::InitCommand;
use crate::commands::link::LinkCommand;
use crate::commands::rfd::RFDCommand;
use crate::commands::til::TilCommand;
use crate::commands::whoami::WhoAmICommand;

pub mod adr;
pub mod build;
pub mod deploy;
pub mod frameworks;
pub mod init;
pub mod link;
mod macros;
pub mod rfd;
pub mod til;
pub mod whoami;

#[derive(Debug, Parser)]
#[command(name = "Doctavious")]
pub struct Opt {
    #[arg(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
    pub debug: bool,

    // TODO: Implement
    // #[arg(
    //     long,
    //     short,
    //     value_parser = parse_output,
    //     help = "How a command output should be rendered",
    //     global = true
    // )]
    // pub(crate) output: Option<Output>,
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Adr(ADRCommand),
    Build(BuildCommand),
    Deploy(DeployCommand),
    Frameworks(FrameworksCommand),
    Init(InitCommand),
    Link(LinkCommand),
    Rfd(RFDCommand),
    Til(TilCommand),
    #[command(name = "whoami")]
    WhoAmI(WhoAmICommand),
}
