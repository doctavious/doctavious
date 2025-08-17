mod commands;
mod config;
mod context;
mod output;

use clap::{Parser, Subcommand};
use tracing::error;

use crate::commands::Command;

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
    pub cmd: SubCommand,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    Adr(commands::adr::ADRCommand),
    Build(commands::build::BuildCommand),
    Changelog(commands::changelog::ChangelogCommand),
    #[command(name = "codenotify")]
    CodeNotify(commands::codenotify::CodeNotifyCommand),
    #[command(name = "codeowners")]
    CodeOwners(commands::codeowners::CodeOwnersCommand),
    Deploy(commands::deploy::DeployCommand),
    Frameworks(commands::frameworks::FrameworksCommand),
    Init(commands::init::InitCommand),
    Link(commands::link::LinkCommand),
    Rfd(commands::rfd::RFDCommand),
    #[command(name = "scmhook")]
    ScmHook(commands::scmhook::ScmHookCommand),
    Til(commands::til::TilCommand),
    Version(commands::version::VersionCommand),
    #[command(name = "whoami")]
    WhoAmI(commands::whoami::WhoAmICommand),
}

// TODO: do we need/want custom error codes?
// TODO: this should probably have
#[tokio::main]
async fn main() {
    // TODO: get version and check for updates. print if cli is not latest
    // Examples:
    // https://github.com/KittyCAD/cli/blob/main/src/main.rs
    // https://github.com/oxidecomputer/oxide.rs/blob/main/cli/src/main.rs

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
        SubCommand::Adr(cmd) => cmd.execute().await,
        SubCommand::Build(cmd) => cmd.execute().await,
        SubCommand::Changelog(cmd) => cmd.execute().await,
        SubCommand::CodeNotify(cmd) => cmd.execute().await,
        SubCommand::CodeOwners(cmd) => cmd.execute().await,
        SubCommand::Deploy(cmd) => cmd.execute().await,
        SubCommand::Frameworks(cmd) => cmd.execute().await,
        SubCommand::Init(..) => unimplemented!(),
        SubCommand::Link(..) => unimplemented!(),
        SubCommand::Rfd(cmd) => cmd.execute().await,
        SubCommand::ScmHook(cmd) => cmd.execute().await,
        SubCommand::Til(cmd) => cmd.execute().await,
        SubCommand::Version(cmd) => cmd.execute().await,
        SubCommand::WhoAmI(..) => unimplemented!(),
    };

    // TODO: support more process exit codes
    match result {
        Ok(output) => {
            if let Some(output) = output {
                println!("{output}");
            }
            std::process::exit(0);
        }
        Err(e) => {
            error!("{e}");
            std::process::exit(1);
        }
    };
}
