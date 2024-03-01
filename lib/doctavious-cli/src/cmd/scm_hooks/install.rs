use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
#[cfg(target_os = "windows")]
use std::os::windows::fs::OpenOptionsExt;
use std::path::Path;

use scm::drivers::Scm;
use scm::hooks::OLD_HOOK_POSTFIX;
use scm::{ScmRepository, DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX, HOOK_TEMPLATE};
use tracing::log::info;

use crate::cmd::scm_hooks::{add_hook, clean, is_doctavious_scm_hook_file};
use crate::settings::{load_settings, ScmHookSettings, SettingErrors};
use crate::{CliResult, DoctaviousCliError};

pub fn install(cwd: &Path) -> CliResult<()> {
    let Some(scm_settings) = &load_settings(cwd)?.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
                    .to_string(),
            ),
        ));
    };

    let scm = Scm::get(cwd)?;
    let hooks_path = scm.hooks_path()?;
    if !hooks_path.exists() {
        fs::create_dir_all(&hooks_path)?;
    }

    for (key, value) in scm_settings.hooks.iter() {
        let hook_path = hooks_path.join(key);
        clean(&key, &hook_path, false)?;
        add_hook(&key, &hook_path)?;
    }

    Ok(())
}
