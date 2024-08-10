// this is used to setup the project in Doctavious
// can be auto setup as part of deploy

// this is really just linking
// reference - https://github.com/vercel/vercel/blob/cfc1c9e818ebb55d440479cf0edf18536b772b28/packages/cli/src/commands/deploy/index.ts#L274

use std::env;
use std::path::PathBuf;

use crate::errors::CliResult;

pub fn invoke(dir: Option<PathBuf>, name: Option<String>) -> CliResult<Option<String>> {
    let cwd = dir.unwrap_or(env::current_dir()?);

    // TODO: default to normalized directory name
    let project_name = name.unwrap_or("".to_string());

    Ok(None)
}
