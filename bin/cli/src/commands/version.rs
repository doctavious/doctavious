use doctavious_cli::CliResult;
use crate::built_info;

pub(crate) fn execute() -> CliResult<Option<String>> {
    println!("Doctavious version: {}", built_info::PKG_VERSION);
    Ok(Some(String::new()))
}