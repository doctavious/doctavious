use std::env;
use std::path::PathBuf;
use cifrs::Cifrs;
use crate::CliResult;

pub fn invoke(dir: Option<PathBuf>, dry: bool, skip_install: bool) -> CliResult<()> {
    let cwd = dir.unwrap_or(env::current_dir()?);
    if dry {
        let _frameworks = Cifrs::detect_frameworks(cwd)?;
    } else {
        Cifrs::build(&cwd, skip_install)?;
    }

    Ok(())
}