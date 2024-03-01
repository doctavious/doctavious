use std::fs;
use std::path::Path;

use scm::drivers::Scm;
use scm::{ScmError, ScmRepository};
use tracing::log::error;

use crate::cmd::scm_hooks::{add_hook, clean};
use crate::settings::{load_settings, SettingErrors};
use crate::{CliResult, DoctaviousCliError};

// TODO: probably detail out more info.
// For bash scripts?
// Creates a directory - provide file structure
// Should this include more options like type, script file name (for bash scripts)? should we open editor for it?

/// Adds a hook directory to a repository
pub fn add(cwd: &Path, hook: String) -> CliResult<()> {
    // let Some(scm_settings) = &load_settings(cwd)?.scmhook_settings else {
    //     return Err(DoctaviousCliError::SettingError(
    //         SettingErrors::SectionNotFound(
    //             "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
    //                 .to_string(),
    //         ),
    //     ));
    // };

    let scm = Scm::get(cwd)?;
    if !scm.supports_hook(&hook) {
        error!("Skip adding, hook is unavailable: {hook}");
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook.to_string(),
        )));
    }

    let hooks_path = scm.hooks_path()?;
    if !hooks_path.exists() {
        fs::create_dir_all(&hooks_path)?;
    }

    clean(&hook, &hooks_path, false)?;
    add_hook(&hook, &hooks_path)?;

    // TODO: create global and local directories for hook scripts
    // global exists ?
    // local exists ?

    Ok(())
}
