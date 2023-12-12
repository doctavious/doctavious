use cifrs::frameworks;
use cifrs::frameworks::FrameworkInfo;

use crate::CliResult;

pub fn invoke() -> CliResult<Vec<FrameworkInfo>> {
    Ok(frameworks::get_all())
}
