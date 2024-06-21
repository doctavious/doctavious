pub mod add;
pub mod install;
pub mod run;
mod runner;
pub mod uninstall;

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
#[cfg(not(target_os = "windows"))]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use crc32c::crc32c;
use minijinja::{AutoEscape, Environment};
use scm::drivers::{Scm, ScmRepository};
use scm::hooks::OLD_HOOK_POSTFIX;
use scm::{ScmError, DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX, HOOK_TEMPLATE, HOOK_TEMPLATE_CHECKSUM};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use tracing::info;

use crate::settings::ScmHookSettings;
use crate::templating::{TemplateContext, Templates};
use crate::{templating, CliResult, DoctaviousCliError};

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

/// Tests whether a hook file was created by doctavious.
pub(crate) fn is_doctavious_scm_hook_file(path: &Path) -> CliResult<bool> {
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    for line in reader.lines().flatten() {
        if DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX.is_match(&line) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Removes the hook from hooks path, saving non-doctavious hooks with .old suffix.
pub(crate) fn clean_hook(hook: &str, path: &Path, force: bool) -> CliResult<()> {
    if !path.exists() {
        return Ok(());
    }

    if is_doctavious_scm_hook_file(path)? {
        info!("removing file {path:?}");
        return Ok(fs::remove_file(path)?);
    }

    let old_path = common::path::append_to_path(path, OLD_HOOK_POSTFIX);
    if old_path.exists() {
        if force {
            info!("File {old_path:?} already exists, overwriting");
        } else {
            return Err(DoctaviousCliError::ScmError(ScmError::OldHookExists(
                hook.to_string(),
            )));
        }
    }

    info!("rename file {path:?} to {old_path:?}");
    fs::rename(&path, &old_path)?;
    info!("Renamed {path:?} to {old_path:?}");

    Ok(())
}

/// create a doctavious hook file using hook template
pub(crate) fn add_hook(hook: &str, path: &Path) -> CliResult<()> {
    let mut binding = fs::OpenOptions::new();
    let mut options = binding.create(true).write(true);

    if !cfg!(windows) {
        options.mode(0o770);
    }

    let template = std::str::from_utf8(HOOK_TEMPLATE)?;
    let context = TemplateContext::from([("hook_name", hook)]);
    let file = Templates::one_off(template, context, false)?;

    Ok(options.open(path)?.write_all(file.as_bytes())?)
}

/// Ensures that SCM hook files are Doctavious hooks
///
/// Rather than always (re)writing hook files we decided to optionally update them based on two
/// main scenarios:
/// 1. Doctavious is updated and the latest version contains a change to the hook template
/// 2. SCM settings are updated, so we want to make sure all hooks are installed/updated
fn ensure_hooks(
    settings: &ScmHookSettings,
    scm: &Scm,
    check_synchronized: bool,
    force: bool,
) -> CliResult<()> {
    if check_synchronized && hooks_synchronized(settings, scm)? {
        return Ok(());
    }

    let hooks_path = scm.ensure_hooks_directory()?;
    let mut synced = Vec::with_capacity(settings.hooks.len());
    for (name, hook) in &settings.hooks {
        let hook_path = hooks_path.join(name);
        clean_hook(name, &hook_path, force)?;
        add_hook(name, &hook_path)?;
        synced.push(name.clone());
    }

    // hate having to do this here again
    let settings_checksum = config_checksum(settings)?;
    add_checksum(
        HOOK_TEMPLATE_CHECKSUM.as_str(),
        settings_checksum.as_str(),
        scm,
    )?;

    // TODO: log created hooks

    Ok(())
}

fn hooks_synchronized(settings: &ScmHookSettings, scm: &Scm) -> CliResult<bool> {
    let checksum_file = fs::File::open(checksum_file(scm)?)?;
    let reader = BufReader::new(checksum_file);
    if let Some(Ok(line)) = reader.lines().next() {
        if line.is_empty() {
            return Ok(false);
        }

        if let Some((stored_hook_checksum, stored_settings_checksum)) = line.split_once('|') {
            if HOOK_TEMPLATE_CHECKSUM.to_string() != stored_hook_checksum {
                return Ok(false);
            }

            let settings_checksum = config_checksum(settings)?;
            if settings_checksum == stored_settings_checksum {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

fn add_checksum(hook_checksum: &str, checksum: &str, scm: &Scm) -> CliResult<()> {
    fs::write(checksum_file(scm)?, format!("{hook_checksum}|{checksum}"))?;

    Ok(())
}

fn config_checksum(settings: &ScmHookSettings) -> CliResult<String> {
    Ok(format!(
        "{:x}",
        md5::compute(serde_json::to_string(settings)?.as_bytes())
    ))
}

fn checksum_file(scm: &Scm) -> CliResult<PathBuf> {
    Ok(scm.info_path()?.join("doctavious.synchronization"))
}
