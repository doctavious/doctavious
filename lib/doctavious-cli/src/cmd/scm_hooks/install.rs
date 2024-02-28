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

/// Removes the hook from hooks path, saving non-doctavious hooks with .old suffix.
pub fn clean(hook: &str, path: &Path, force: bool) -> CliResult<()> {
    if !path.exists() {
        return Ok(());
    }

    if is_doctavious_scm_hook_file(path)? {
        return Ok(fs::remove_file(path)?);
    }

    let old_path = path.join(OLD_HOOK_POSTFIX);
    if old_path.exists() {
        if force {
            info!("File {old_path:?} already exists, overwriting");
        } else {
            // TODO: return error old path already exists
            // Can't rename x to x.old as file already exists
        }
    }

    fs::rename(&path, &old_path)?;

    info!("Renamed {path:?} to {old_path:?}");
    Ok(())
}

/// create a doctavious hook file using hook template
pub fn add_hook(hook: &str, path: &Path) -> CliResult<()> {
    // get hook path
    // write file with template. set file mode

    // let open_options = fs::OpenOptions::new()
    //     .create(true)
    //     .write(true);

    // TODO: what would windows be if anything?
    #[cfg(target_os = "windows")]
    open_options.mode(0o770);

    // #[cfg(not(target_os = "windows"))]
    // open_options = open_options.mode(0o770);

    Ok(fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o770)
        .open(path)?
        .write_all(HOOK_TEMPLATE)?)
}

/// Tests whether a hook file was created by doctavious.
fn is_doctavious_scm_hook_file(path: &Path) -> CliResult<bool> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    for line in reader.lines().flatten() {
        if DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX.is_match(&line) {
            return Ok(true);
        }
    }

    Ok(false)
}
