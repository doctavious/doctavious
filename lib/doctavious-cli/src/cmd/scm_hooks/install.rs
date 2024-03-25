use std::fs;
use std::path::Path;

use scm::drivers::Scm;
use scm::ScmRepository;

use crate::cmd::scm_hooks::{add_hook, clean_hook};
use crate::settings::{load_settings, SettingErrors, Settings};
use crate::{CliResult, DoctaviousCliError};

pub fn install(cwd: &Path) -> CliResult<()> {
    let mut settings: Settings = load_settings(cwd)?.into_owned();
    let Some(scm_settings) = settings.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
                    .to_string(),
            ),
        ));
    };

    let scm = Scm::get(cwd)?;
    let hooks_path = scm.ensure_hooks_directory()?;

    for (key, value) in scm_settings.hooks.iter() {
        let hook_path = hooks_path.join(key);
        clean_hook(&key, &hook_path, false)?;
        add_hook(&key, &hook_path)?;
    }

    Ok(())
}
