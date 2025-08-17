use clap::Parser;
use doctavious_cli::errors::CliResult;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser, Debug, Clone)]
pub struct VersionCommand;

pub fn execute(_cmd: VersionCommand) -> CliResult<Option<String>> {
    println!("Doctavious version: {}", built_info::PKG_VERSION);
    // TODO: include git commit hash
    // TODO: include Doctavious API version
    Ok(Some(String::new()))
}
