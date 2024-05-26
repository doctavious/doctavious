use clap::Parser;
use doctavious_cli::CliResult;
use crate::built_info;

/// Version Information
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct VersionCommand;

pub(crate) fn execute(_: VersionCommand) -> CliResult<()> {
    println!("Doctavious version: {}", built_info::PKG_VERSION);
    Ok(())
}