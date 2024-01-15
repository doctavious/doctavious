use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til::open;
use doctavious_cli::CliResult;

/// Open TIL
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct OpenTil {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// The post, in the format of <topic/title>, to open
    #[arg(index = 0)]
    pub post: String,
}

pub(crate) fn execute(cmd: OpenTil) -> CliResult<Option<String>> {
    open(cmd.cwd.as_deref(), cmd.post)?;
    Ok(Some(String::new()))
}
