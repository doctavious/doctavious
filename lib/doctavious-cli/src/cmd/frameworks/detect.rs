use std::env;
use std::path::PathBuf;

use cifrs::frameworks::FrameworkInfo;
use cifrs::Cifrs;

use crate::CliResult;

pub fn invoke(dir: Option<PathBuf>) -> CliResult<FrameworkInfo> {
    let cwd = dir.unwrap_or(env::current_dir()?);
    Ok(Cifrs::detect_frameworks(cwd)?)
}
