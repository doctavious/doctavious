use std::path::Path;

use scm::drivers::Scm;

use crate::cmd::scm_hooks::ensure_hooks;
use crate::errors::{CliResult, DoctaviousCliError};
use crate::settings::{load_settings, SettingErrors, Settings};

pub fn install(cwd: &Path, force: bool) -> CliResult<()> {
    let settings: Settings = load_settings(cwd)?.into_owned();
    let Some(scm_settings) = settings.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scmhook add`"
                    .to_string(),
            ),
        ));
    };

    let scm = Scm::get(cwd)?;
    ensure_hooks(&scm_settings, &scm, false, force)
}
