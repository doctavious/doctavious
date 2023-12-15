use std::env;
use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Build on your local machine")]
pub(crate) struct BuildCommand {
    #[arg(
        long,
        short,
        help = "The directory to build. Defaults to current directory."
    )]
    pub cwd: Option<PathBuf>,

    // Dry run: show instructions without running them (default: false)
    // should this just find framework and show command it will run?
    #[arg(long, short, help = "Dry run: show instructions without running them")]
    pub dry: bool,

    // not sure if this is needed
    // context Specify a build_mod context or branch (contexts: "production", "deploy-preview", "branch-deploy", "dev") (default: "production")

    // yes want this
    // option can be used to provide a working directory (that can be different from the current directory) when running CLI commands.
    // --cwd
    // pub cwd: String

    // this is global
    // The --debug option, shorthand -d, can be used to provide a more verbose output when running Vercel CLI commands.
    #[arg(long, short, help = "Skip installing dependencies")]
    pub skip_install: bool,
}

impl Default for BuildCommand {
    fn default() -> Self {
        Self {
            cwd: Some(env::current_dir().expect("Should be able to get current working directory")),
            dry: false,
            skip_install: false,
        }
    }
}

#[derive(Parser, Debug)]
#[command(about = "Create a new deploy from the contents of a folder")]
pub(crate) struct DeployCommand {
    #[arg(
        long,
        short,
        help = "The directory to build. Defaults to current directory."
    )]
    pub cwd: Option<PathBuf>,

    #[arg(
        long,
        short,
        help = "Specifies the alias for deployment, the string at the beginning of the deploy subdomain. Useful for creating predictable deployment URLs."
    )]
    pub alias: Option<String>,

    #[arg(long, short, help = "Deploy to Production")]
    pub prod: bool,

    // build / --prebuilt': Boolean,
    // TODO: include details that this will fail if build is false and output dir is not found/empty
    #[arg(long, short, help = "Whether to build prior to deploy.")]
    pub build: bool,
    // -a --auth <token>
    // --build_mod Run build_mod command before deploying

    // -m, --message <message> A short message to include in the deploy log

    // -o, --open Open site after deploy (default: false)

    // vercel had something similar called name which is deprecated in favor of linking...i prefer
    // the link as well
    // -s, --site <name-or-id> A site name or ID to deploy to

    // --timeout <number>  Timeout to wait for deployment to finish

    // '-y': '--yes', is autoConfirm

    // allow for build-env
}

#[derive(Parser, Debug)]
#[command(about = "")]
pub(crate) struct FrameworksCommand {
    #[command(subcommand)]
    pub framework_command: FrameworkSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum FrameworkSubCommand {
    Detect(DetectFrameworks),
    Get(GetFramework),
    List(ListFrameworks),
}

#[derive(Parser, Debug)]
#[command(about = "Detect Frameworks")]
pub(crate) struct DetectFrameworks {
    #[arg(long, short, help = "Directory to detect framewoks in")]
    pub cwd: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(about = "List Frameworks")]
pub(crate) struct ListFrameworks {}

#[derive(Parser, Debug)]
#[command(about = "Get Framework Details")]
pub(crate) struct GetFramework {
    #[arg(long, short, help = "Name of the framework")]
    pub name: String,
}


#[derive(Parser, Debug)]
#[command(about = "Show the username of the user currently logged into Doctavious CLI.")]
pub(crate) struct WhoAmICommand;

#[derive(Parser, Debug)]
#[command(about = "Initialize Doctavious Projects locally")]
pub(crate) struct InitCommand {
    #[arg(help = "Name of the Project", index = 0)]
    pub name: Option<String>,
}

#[derive(Parser, Debug)]
#[command(about = "Links your local directory to a Doctavious Project.")]
pub(crate) struct LinkCommand {
    #[arg(long, short, help = "Directory of the local Doctavious project", index=0)]
    pub cwd: Option<PathBuf>,

    // TODO: auto confirm
}