use std::fs;
use std::path::{Path, PathBuf};

use scm::drivers::Scm;
use scm::{ScmError, ScmRepository};
use tracing::{error, info};

use crate::cmd::scm_hooks::{add_hook, clean};
use crate::settings::{load_settings, SettingErrors, DEFAULT_CONFIG_DIR};
use crate::{CliResult, DoctaviousCliError};

// TODO: probably detail out more info.
// For bash scripts?
// Creates a directory - provide file structure
// Should this include more options like type, script file name (for bash scripts)? should we open editor for it?

/// Adds a hook directory to a repository
pub fn add(cwd: &Path, hook_name: String, dirs: bool, force: bool) -> CliResult<()> {
    // let Some(scm_settings) = &load_settings(cwd)?.scmhook_settings else {
    //     return Err(DoctaviousCliError::SettingError(
    //         SettingErrors::SectionNotFound(
    //             "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
    //                 .to_string(),
    //         ),
    //     ));
    // };

    let scm = Scm::get(cwd)?;
    if !scm.supports_hook(&hook_name) {
        error!("Skip adding, hook is unavailable: {hook_name}");
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook_name.to_string(),
        )));
    }

    info!("checking hook path");
    let hooks_path = scm.hooks_path()?;
    if !hooks_path.exists() {
        fs::create_dir_all(&hooks_path)?;
    }

    let hook_path = hooks_path.join(&hook_name);

    info!("clean");
    clean(&hook_name, &hook_path, force)?;

    info!("add hook");
    add_hook(&hook_name, &hook_path)?;

    if dirs {
        fs::create_dir_all(PathBuf::from(DEFAULT_CONFIG_DIR).join(hook_name))?;
    }

    // TODO: create global and local directories for hook scripts
    // global exists ?
    // local exists ?

    Ok(())
}
