use cifrs::frameworks;
use cifrs::frameworks::FrameworkInfo;

use crate::CliResult;

pub fn invoke(name: String) -> CliResult<Option<FrameworkInfo>> {
    Ok(frameworks::get_all().into_iter().find(|f| f.name == name))
}
