use std::fs;
use std::path::Path;

use scm::drivers::Scm;
use scm::{ScmError, ScmRepository};

use crate::cmd::scm_hooks::{add_hook, clean_hook};
use crate::settings::DEFAULT_CONFIG_DIR;
use crate::{CliResult, DoctaviousCliError};

// TODO: probably detail out more info.
// For bash scripts?
// Creates a directory - provide file structure
// Should this include more options like type, script file name (for bash scripts)? should we open editor for it?

/// Adds a Doctavious hook.
///
/// If `create_hook_script_dir` is `true` a directory [DEFAULT_CONFIG_DIR]/scmhooks/<hook_name>
/// will be created.
///
/// If `force` is `true` existing hooks will be
pub fn add(
    cwd: &Path,
    hook_name: String,
    create_hook_script_dir: bool,
    force: bool,
) -> CliResult<()> {
    let scm = Scm::get(cwd)?;
    if !scm.supports_hook(&hook_name) {
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook_name.to_string(),
        )));
    }

    let hooks_path = scm.ensure_hooks_directory()?;
    let hook_path = hooks_path.join(&hook_name);

    clean_hook(&hook_name, &hook_path, force)?;
    add_hook(&hook_name, &hook_path)?;

    if create_hook_script_dir {
        fs::create_dir_all(
            cwd.join(DEFAULT_CONFIG_DIR)
                .join("scmhooks")
                .join(hook_name),
        )?;
    }

    Ok(())
}
