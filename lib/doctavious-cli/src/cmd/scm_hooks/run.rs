use std::path::{Path, PathBuf};

use scm::drivers::{Scm, ScmRepository};
use scm::errors::ScmError;
use thiserror::Error;
use tracing::{info, warn};

use crate::cmd::scm_hooks::ensure_hooks;
use crate::cmd::scm_hooks::runner::{
    ScmHookRunner, ScmHookRunnerOptions, ScmHookRunnerOutcome, ScmHookRunnerResult,
};
use crate::errors::{CliResult, DoctaviousCliError};
use crate::settings::{load_settings, SettingErrors, Settings};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum DoctaviousScmHookRunError {
    #[error("{0}")]
    ConflictingOptions(String),
}

#[remain::sorted]
pub enum ScmHookRunFiles {
    All,
    Specific(Vec<PathBuf>),
}

pub fn run(
    cwd: &Path,
    hook_name: &str,
    files: Option<ScmHookRunFiles>,
    run_only_executions: Vec<String>,
    synchronize_hooks: bool,
    force: bool,
) -> CliResult<()> {
    let settings: Settings = load_settings(cwd)?;
    let Some(scm_settings) = settings.scmhook_settings else {
        return Err(DoctaviousCliError::SettingError(
            SettingErrors::SectionNotFound(
                "Either edit `scm` settings in doctavious configuration or use `scmhook add`"
                    .to_string(),
            ),
        ));
    };

    let scm = Scm::get(cwd)?;
    if synchronize_hooks {
        // TODO: do we want to return error?
        if ensure_hooks(&scm_settings, &scm, true, force).is_err() {
            warn!("There was a problem synchronizing hooks. Try running 'doctavious scmhook install' manually")
        }
    }

    let Some(hook) = scm_settings.hooks.get(hook_name) else {
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook_name.to_string(),
        )));
    };

    if hook.parallel && hook.stop_on_failure {
        // TODO: return error
        // conflicting options 'piped' and 'parallel' are set to 'true', remove one of this option from hook group
    }

    let files = match files {
        None => vec![],
        Some(f) => match f {
            ScmHookRunFiles::All => scm.all_files()?,
            ScmHookRunFiles::Specific(files) => files,
        },
    };

    let runner = ScmHookRunner::new(ScmHookRunnerOptions {
        cwd,
        hook,
        hook_name: hook_name.to_string(),
        files,
        run_only_executions,
        force,
    });

    let results = runner.run_all();
    print_summary(results);

    Ok(())
}

fn print_summary(results: Vec<ScmHookRunnerResult<ScmHookRunnerOutcome>>) {
    // TODO: have log settings
    // TODO: print summary
    for result in results {
        match result {
            Ok(r) => {
                // TODO: color green
                info!("✔️  {}", "name")
            }
            Err(e) => {
                // TODO: color red
                info!("{}", e)
            }
        }
    }
}
