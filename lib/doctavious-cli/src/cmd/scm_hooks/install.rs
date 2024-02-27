use std::path::Path;

use scm::Scm;

use crate::settings::{load_settings, Settings};
use crate::CliResult;

pub fn install(cwd: &Path) -> CliResult<()> {
    if let Some(settings) = &load_settings(cwd)?.scmhook_settings {
        // return error - scm settings not found. Either edit scm settings in doctavious.yaml or
        // use `scm_hook add`
    }

    let scm = Scm::get(cwd)?;
    // get scm
    // get hook path
    // ensure hooks dir exists

    // for each hook in settings
    // clean
    // add hook

    // see what a ghost hook is

    Ok(())
}

pub fn clean(hook: String, force: bool) -> CliResult<()> {
    // get hook path
    // confirm exists

    // if doctavious hook then remove

    // is if .old  file already exists before renaming
    // if it exists and force is true log overwriting
    // else return error that path.old already exists

    // rename hook path with .old postfix

    Ok(())
}

pub fn add_hook(hook: String) -> CliResult<()> {
    // get hook path
    // write file with template. set file mode
    Ok(())
}
