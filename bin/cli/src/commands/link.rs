use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Links your local directory to a Doctavious Project.")]
pub(crate) struct LinkCommand {
    #[arg(
        long,
        short,
        help = "Directory of the local Doctavious project",
        index = 0
    )]
    pub cwd: Option<PathBuf>,
    // TODO: auto confirm
}
