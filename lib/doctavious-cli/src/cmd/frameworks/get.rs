use cifrs::frameworks;

use crate::{CliResult, DoctaviousCliError};

pub fn invoke(name: String) -> CliResult<Option<String>> {
    let framework = frameworks::get_all().into_iter().find(|f| f.name == name);
    serde_json::to_string(&framework)
        .map_err(DoctaviousCliError::SerdeJson)
        .map(|f| Some(f))
}
