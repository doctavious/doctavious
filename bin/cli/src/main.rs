use clap::Parser;
use doctavious_cli::cmd::{build, deploy, frameworks};
use tracing::error;

use crate::args::{
    BuildCommand, DeployCommand, FrameworkSubCommand, FrameworksCommand, InitCommand, LinkCommand,
    WhoAmICommand,
};

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
    cmd: Command,
}

#[derive(Debug, Parser)]
enum Command {
    Build(BuildCommand),
    Deploy(DeployCommand),
    Frameworks(FrameworksCommand),
    WhoAmI(WhoAmICommand),
    Init(InitCommand),
    Link(LinkCommand),
}

// TODO: do we need/want custom error codes?
// TODO: this should probably have
// #[tokio::main]
fn main() {
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

    let result = match opt.cmd {
        Command::Build(cmd) => build::invoke(cmd.cwd, cmd.dry, cmd.skip_install),
        Command::Deploy(cmd) => deploy::invoke(cmd.cwd, cmd.build),
        Command::Frameworks(cmd) => match cmd.framework_command {
            FrameworkSubCommand::Detect(cmd) => frameworks::detect::invoke(cmd.cwd),
            FrameworkSubCommand::Get(cmd) => frameworks::get::invoke(cmd.name),
            FrameworkSubCommand::List(_) => frameworks::list::invoke(),
        },
        Command::WhoAmI(..) => unimplemented!(),
        Command::Init(..) => unimplemented!(),
        Command::Link(..) => unimplemented!(),
    };

    match result {
        Ok(output) => {
            if let Some(output) = output {
                println!("{output}");
                std::process::exit(0);
            }
        }
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };
}
