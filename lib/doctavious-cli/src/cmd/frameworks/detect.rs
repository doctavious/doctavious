use std::env;
use std::path::PathBuf;

use cifrs::Cifrs;

use crate::errors::{CliResult, DoctaviousCliError};

pub fn execute(dir: Option<PathBuf>) -> CliResult<Option<String>> {
    let cwd = dir.unwrap_or(env::current_dir()?);
    let framework = Cifrs::detect_frameworks(cwd)?;
    serde_json::to_string(&framework)
        .map_err(DoctaviousCliError::SerdeJson)
        .map(|f| Some(f))
}
