use std::env;
use std::path::PathBuf;

use cifrs::Cifrs;

use crate::errors::CliResult;

pub fn execute(dir: Option<PathBuf>, dry: bool, skip_install: bool) -> CliResult<Option<String>> {
    let cwd = dir.unwrap_or(env::current_dir()?);
    Cifrs::build(&cwd, dry, skip_install)?;

    Ok(None)
}
