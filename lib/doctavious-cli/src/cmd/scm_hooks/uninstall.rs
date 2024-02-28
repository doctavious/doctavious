use std::fs;
use std::path::Path;
use tracing::debug;
use tracing::log::error;
use scm::drivers::Scm;
use scm::hooks::OLD_HOOK_POSTFIX;
use scm::ScmRepository;

use crate::settings::{load_settings, SettingErrors};
use crate::{CliResult, DoctaviousCliError};
use crate::cmd::scm_hooks::is_doctavious_scm_hook_file;

pub fn uninstall(cwd: &Path) -> CliResult<()> {
    let Some(scm_settings) = &load_settings(cwd)?.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
                    .to_string(),
            ),
        ));
    };

    delete_hooks(cwd)?;

    // TODO: remove scm section from config

    Ok(())
}


fn delete_hooks(cwd: &Path) -> CliResult<()> {
    let scm = Scm::get(cwd)?;
    let hooks_path = scm.hooks_path()?;
    for entry in fs::read_dir(hooks_path)? {
        let entry = entry?;
        let path = entry.path();

        // do we want a force option?
        if !is_doctavious_scm_hook_file(&path) {
            continue;
        }

        match fs::remove_file(&path) {
            Ok(_) => debug!("{path:?} removed"),
            Err(e) => error!("Failed removing {path:?}: {e}")
        }

        let old_hook = path.join(OLD_HOOK_POSTFIX);
        if !old_hook.exists() {
            continue;
        }

        match fs::rename(&old_hook, &path) {
            Ok(_) => debug!("{old_hook:?} renamed to {path:?}"),
            Err(e) => error!("Failed renaming {old_hook:?}: {e}")
        }
    }

    Ok(())
}