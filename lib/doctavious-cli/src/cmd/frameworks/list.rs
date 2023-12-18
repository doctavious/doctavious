use cifrs::frameworks;

use crate::{CliResult, DoctaviousCliError};

pub fn invoke() -> CliResult<Option<String>> {
    let frameworks = frameworks::get_all();
    serde_json::to_string(&frameworks)
        .map_err(DoctaviousCliError::SerdeJson)
        .map(|f| Some(f))
}
