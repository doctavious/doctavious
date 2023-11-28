use anyhow::Result;
use clap::Parser;

use crate::args::BuildCommand;
use crate::output::{Output, parse_output};

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
}


fn main() -> Result<()> {

    // TODO: telemetry?


    // let opt = Opt::parse();
    // if opt.debug {
    //     env::set_var("RUST_LOG", "debug");
    //     env_logger::init();
    // }

    Ok(())
}