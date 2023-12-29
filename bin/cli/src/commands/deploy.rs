use std::path::PathBuf;

use clap::Parser;

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
