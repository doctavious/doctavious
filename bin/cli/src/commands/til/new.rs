use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til;
use doctavious_cli::errors::CliResult;

/// New TIL
#[derive(Parser, Debug)]
#[command()]
pub struct NewTil {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Additional tags associated with the TIL entry
    #[arg(short, long)]
    pub tags: Option<Vec<String>>,

    // TODO: should this be an action?
    // TODO: should this also be a setting in TilSettings?
    /// Whether to build_mod a README after a new TIL is added
    #[arg(long)]
    pub toc: bool,

    /// The post, in the format of <category/title>, to create.
    ///
    /// The category will be the folder while title will be used for the filename. You can also include an extension
    #[arg(index = 1)]
    pub post: String,
}

pub fn execute(cmd: NewTil) -> CliResult<Option<String>> {
    let output = til::new(cmd.cwd.as_deref(), cmd.post, cmd.tags, cmd.toc)?;

    Ok(Some(output.to_string_lossy().to_string()))
}
