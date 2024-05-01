use std::path::{Path, PathBuf};

use scm::drivers::Scm;
use scm::{ScmError, ScmRepository};

use crate::cmd::scm_hooks::ensure_hooks;
use crate::cmd::scm_hooks::runner::{ScmHookRunner, ScmHookRunnerOptions};
use crate::settings::{load_settings, SettingErrors, Settings};
use crate::{CliResult, DoctaviousCliError};

#[remain::sorted]
pub enum ScmHookRunFiles {
    All,
    Specific(Vec<String>),
}

// staged files?

// force - force execution of commands that can be skipped
// no-tty - run hook non-interactively, disable spinner
// all-files - run hooks on all files
// files-from-stdin - get files from standard input, null-separated
// files - run on specified files, comma-separated
// commands - run only specified commands
// hook_name
pub fn run(
    cwd: &Path,
    hook_name: &str,
    // files: ScmHookRunFiles,
    all_files: bool,
    mut files: Vec<PathBuf>,
    run_only_executions: Vec<String>,
    force: bool,
) -> CliResult<()> {
    let settings: Settings = load_settings(cwd)?.into_owned();
    let Some(scm_settings) = settings.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scm_hook add`"
                    .to_string(),
            ),
        ));
    };

    let scm = Scm::get(cwd)?;
    ensure_hooks(&scm_settings, &scm, force)?;

    let Some(hook) = scm_settings.hooks.get(hook_name) else {
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook_name.to_string(),
        )));
    };

    if hook.parallel && hook.stop_on_failure {
        // TODO: return error
        // conflicting options 'piped' and 'parallel' are set to 'true', remove one of this option from hook group
    }

    if files.is_empty() && all_files {
        files.extend(scm.all_files()?);
    }

    let runner = ScmHookRunner::new(ScmHookRunnerOptions {
        cwd,
        scm: &scm,
        hook,
        hook_name: hook_name.to_string(),
        files,
        run_only_executions,
    });

    let results = runner.run_all();

    // TODO: print summary results

    Ok(())
}
