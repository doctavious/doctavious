use clap::Parser;
use doctavious_cli::cmd::{build, deploy, frameworks};
use tracing::error;

use crate::commands::frameworks::FrameworkSubCommand;
use crate::commands::{adr, rfd, Command, Opt, til};

mod commands;
mod config;
mod output;

// #[async_trait]
pub trait RunnableCmd: Send + Sync {
    // async fn run(&self, ctx: &Context) -> Result<()>;
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
    // not sure how to configure that

    // TODO: get configuration: file + env

    // TODO: would like to return something other than a string so that we could handle multiple output formats
    let result = match opt.cmd {
        Command::Adr(cmd) => adr::execute(cmd),
        Command::Build(cmd) => build::invoke(cmd.cwd, cmd.dry, cmd.skip_install),
        Command::Deploy(cmd) => deploy::invoke(cmd.cwd, cmd.build),
        Command::Frameworks(cmd) => match cmd.framework_command {
            FrameworkSubCommand::Detect(cmd) => frameworks::detect::invoke(cmd.cwd),
            FrameworkSubCommand::Get(cmd) => frameworks::get::invoke(cmd.name),
            FrameworkSubCommand::List(_) => frameworks::list::invoke(),
        },
        Command::Init(..) => unimplemented!(),
        Command::Link(..) => unimplemented!(),
        Command::Rfd(cmd) => rfd::execute(cmd),
        Command::Til(cmd) => til::execute(cmd),
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
