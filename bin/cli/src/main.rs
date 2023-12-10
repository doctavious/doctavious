use anyhow::Result;
use clap::Parser;
use doctavious_cli::cmd::{build, deploy};
use tracing::log::Level;

use crate::args::{BuildCommand, DeployCommand};
use crate::output::{parse_output, Output};

mod args;
mod config;
mod output;

#[derive(Debug, Parser)]
#[command(name = "Doctavious")]
pub struct Opt {
    #[arg(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
    debug: bool,

    #[arg(
        long,
        short,
        value_parser = parse_output,
        help = "How a command output should be rendered",
        global = true
    )]
    pub(crate) output: Option<Output>,

    #[command(subcommand)]
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Build(BuildCommand),
    Deploy(DeployCommand),
}

fn main() -> Result<()> {
    // TODO: get version and check for updates. print if cli is not latest

    let opt = Opt::parse();

    let tracing_level = if opt.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt::fmt()
        .with_max_level(tracing_level)
        .init();

    // TODO: should probably log diagnostics info/debug to stderr and result to stdout

    // TODO: get configuration: file + env

    let o = match opt.cmd {
        Command::Build(cmd) => build::invoke(cmd.cwd, cmd.dry, cmd.skip_install),
        Command::Deploy(cmd) => deploy::invoke(cmd.cwd, cmd.build),
    };

    Ok(())
}
