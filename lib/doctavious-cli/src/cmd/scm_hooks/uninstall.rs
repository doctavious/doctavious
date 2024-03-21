use std::borrow::Cow;
use std::fs;
use std::path::Path;

use scm::drivers::Scm;
use scm::hooks::OLD_HOOK_POSTFIX;
use scm::ScmRepository;
use tracing::{debug, error};

use crate::cmd::scm_hooks::is_doctavious_scm_hook_file;
use crate::settings::{load_settings, persist_settings, SettingErrors, Settings};
use crate::{CliResult, DoctaviousCliError};

pub fn uninstall(cwd: &Path, force: bool, remove_settings: bool) -> CliResult<()> {
    let mut settings: Settings = load_settings(cwd)?.into_owned();

    delete_hooks(cwd, force)?;

    if remove_settings && settings.scmhook_settings.is_some() {
        settings.scmhook_settings = None;
        persist_settings(cwd, &settings)?;
    }

    Ok(())
}

fn delete_hooks(cwd: &Path, force: bool) -> CliResult<()> {
    let scm = Scm::get(cwd)?;
    let hooks_path = scm.hooks_path()?;
    for entry in fs::read_dir(hooks_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            debug!("skipping {path:?}...not a file");
            continue;
        }

        // TODO: Do we want to keep sample hooks? Where should that logic live? In the scm interface?
        if !force && !is_doctavious_scm_hook_file(&path)? {
            debug!(
                "skipping {path:?}...not a doctavious hook file. To remove please use force option"
            );
            continue;
        }

        match fs::remove_file(&path) {
            Ok(_) => debug!("{path:?} removed"),
            Err(e) => error!("Failed removing {path:?}: {e}"),
        }

        let old_hook = path.join(OLD_HOOK_POSTFIX);
        if old_hook.exists() {
            match fs::rename(&old_hook, &path) {
                Ok(_) => debug!("{old_hook:?} renamed to {path:?}"),
                Err(e) => error!("Failed renaming {old_hook:?}: {e}"),
            }
        }
    }

    Ok(())
}
