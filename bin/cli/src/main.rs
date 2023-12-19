use clap::Parser;
use doctavious_cli::cmd::{build, deploy, frameworks};
use tracing::error;

use crate::cmds::adr::ADRSubCommand;
use crate::cmds::frameworks::FrameworkSubCommand;
use crate::cmds::rfd::RFDSubCommand;
use crate::cmds::{Command, Opt};

mod cmds;
mod config;
mod output;

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
        Command::Adr(cmd) => match cmd.adr_command {
            ADRSubCommand::Init(_) => unimplemented!(),
            ADRSubCommand::Generate(_) => unimplemented!(),
            ADRSubCommand::List(_) => unimplemented!(),
            ADRSubCommand::Link(_) => unimplemented!(),
            ADRSubCommand::New(_) => unimplemented!(),
            ADRSubCommand::Reserve(_) => unimplemented!(),
        },
        Command::Build(cmd) => build::invoke(cmd.cwd, cmd.dry, cmd.skip_install),
        Command::Deploy(cmd) => deploy::invoke(cmd.cwd, cmd.build),
        Command::Frameworks(cmd) => match cmd.framework_command {
            FrameworkSubCommand::Detect(cmd) => frameworks::detect::invoke(cmd.cwd),
            FrameworkSubCommand::Get(cmd) => frameworks::get::invoke(cmd.name),
            FrameworkSubCommand::List(_) => frameworks::list::invoke(),
        },
        Command::Init(..) => unimplemented!(),
        Command::Link(..) => unimplemented!(),
        Command::Rfd(cmd) => match cmd.rfd_command {
            RFDSubCommand::Init(_) => unimplemented!(),
            RFDSubCommand::New(_) => unimplemented!(),
            RFDSubCommand::List(_) => unimplemented!(),
            RFDSubCommand::Generate(_) => unimplemented!(),
            RFDSubCommand::Reserve(_) => unimplemented!(),
        },
        Command::Til(..) => unimplemented!(),
        Command::WhoAmI(..) => unimplemented!(),
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
