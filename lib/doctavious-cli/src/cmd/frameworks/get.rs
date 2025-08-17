use cifrs::frameworks;

use crate::errors::{CliResult, DoctaviousCliError};

pub fn execute(name: String) -> CliResult<Option<String>> {
    let normalized_string = name.to_lowercase();
    let framework = frameworks::get_all().into_iter().find(|f| {
        f.name.to_lowercase() == normalized_string || f.id.to_lowercase() == normalized_string
    });
    match framework {
        None => Ok(None),
        Some(f) => serde_json::to_string(&f)
            .map_err(DoctaviousCliError::SerdeJson)
            .map(|f| Some(f)),
    }
}
