use std::path::PathBuf;

use cifrs::Cifrs;

use crate::errors::CliResult;

// TODO: I think we can remove this. Seems unnecessary
pub fn execute(cwd: PathBuf, dry: bool, skip_install: bool) -> CliResult<Option<String>> {
    Cifrs::build(&cwd, dry, skip_install)?;

    Ok(None)
}
