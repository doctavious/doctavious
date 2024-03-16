pub mod add;
pub mod install;
pub mod run;
mod runner;
pub mod uninstall;

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use minijinja::{AutoEscape, Environment};
use scm::hooks::OLD_HOOK_POSTFIX;
use scm::{DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX, HOOK_TEMPLATE};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use tracing::info;

use crate::templating::{TemplateContext, Templates};
use crate::{templating, CliResult};

// idea from rusty-hook and left-hook
// TODO: flush this out more

// add hook
// execute hook

/// Tests whether a hook file was created by doctavious.
pub(crate) fn is_doctavious_scm_hook_file(path: &Path) -> CliResult<bool> {
    info!("reading file {path:?}");
    let f = fs::File::open(path)?;
    let reader = BufReader::new(f);
    info!("checking file");
    for line in reader.lines().flatten() {
        if DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX.is_match(&line) {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Removes the hook from hooks path, saving non-doctavious hooks with .old suffix.
pub(crate) fn clean(hook: &str, path: &Path, force: bool) -> CliResult<()> {
    info!("check to see if path exists");
    if !path.exists() {
        return Ok(());
    }

    info!("check to see if path is doctavious hook");
    if is_doctavious_scm_hook_file(path)? {
        info!("removing file {path:?}");
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
