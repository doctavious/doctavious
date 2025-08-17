use std::env;
use std::path::PathBuf;

use cifrs::Cifrs;

use crate::cmd::design_decisions::adr;
use crate::errors::CliResult;

// TODO: I think we can remove this. Seems unnecessary
pub fn execute(cwd: PathBuf, dry: bool, skip_install: bool) -> CliResult<Option<String>> {
    Cifrs::build(&cwd, dry, skip_install)?;

    Ok(None)
}
