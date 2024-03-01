mod add;
pub mod init;
pub mod install;
pub mod run;
pub mod uninstall;

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use scm::hooks::OLD_HOOK_POSTFIX;
use scm::{DOCTAVIOUS_SCM_HOOK_CONTENT_REGEX, HOOK_TEMPLATE};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::log::info;

use crate::CliResult;

// idea from rusty-hook and left-hook
// TODO: flush this out more

// add hook
// execute hook

fn init() {}

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
pub(crate) fn clean(hook: &str, path: &Path, force: bool) -> CliResult<()> {
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
pub(crate) fn add_hook(hook: &str, path: &Path) -> CliResult<()> {
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
