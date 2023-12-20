use clap::{Parser, Subcommand};

use crate::cmds::adr::ADRCommand;
use crate::cmds::build::BuildCommand;
use crate::cmds::deploy::DeployCommand;
use crate::cmds::frameworks::FrameworksCommand;
use crate::cmds::init::InitCommand;
use crate::cmds::link::LinkCommand;
use crate::cmds::rfd::RFDCommand;
use crate::cmds::til::TilCommand;
use crate::cmds::whoami::WhoAmICommand;

pub mod adr;
pub mod build;
pub mod deploy;
pub mod frameworks;
pub mod init;
pub mod link;
pub mod rfd;
mod til;
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
