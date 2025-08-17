mod commands;
mod config;
mod context;
mod output;

use clap::{Parser, Subcommand};
use doctavious_cli::cmd::{build, deploy, frameworks};
use tracing::error;

use crate::commands::frameworks::FrameworkSubCommand;

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
    Adr(commands::adr::ADRCommand),
    Build(commands::build::BuildCommand),
    Changelog(commands::changelog::ChangelogCommand),
    #[command(name = "codenotify")]
    CodeNotify(commands::codenotify::CodeNotifyCli),
    #[command(name = "codeowners")]
    CodeOwners(commands::codeowners::CodeOwnersCli),
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
    // Example: https://github.com/KittyCAD/cli/blob/main/src/main.rs

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
        Command::Adr(cmd) => commands::adr::execute(cmd),
        Command::Build(cmd) => build::execute(cmd.cwd, cmd.dry, cmd.skip_install),
        Command::Changelog(cmd) => commands::changelog::execute(cmd),
        Command::CodeNotify(cmd) => commands::codenotify::execute(cmd).await,
        Command::CodeOwners(cmd) => commands::codeowners::execute(cmd),
        Command::Deploy(cmd) => deploy::execute(cmd.cwd, cmd.build),
        Command::Frameworks(cmd) => match cmd.framework_command {
            FrameworkSubCommand::Detect(cmd) => frameworks::detect::execute(cmd.cwd),
            FrameworkSubCommand::Get(cmd) => frameworks::get::execute(cmd.name),
            FrameworkSubCommand::List(_) => frameworks::list::execute(),
        },
        Command::Init(..) => unimplemented!(),
        Command::Link(..) => unimplemented!(),
        Command::Rfd(cmd) => commands::rfd::execute(cmd),
        Command::ScmHook(cmd) => commands::scmhook::execute(cmd),
        Command::Til(cmd) => commands::til::execute(cmd),
        Command::Version(cmd) => commands::version::execute(cmd),
        Command::WhoAmI(..) => unimplemented!(),
    };

    // TODO: support more process exit codes
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
