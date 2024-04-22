use std::path::{Path, PathBuf};

use scm::drivers::Scm;
use scm::hooks::ScmHook;
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
    let hooks_path = scm.ensure_hooks_directory()?;

    ensure_hooks(&scm_settings, &scm, force)?;

    let Some(hook) = scm_settings.hooks.get(hook_name) else {
        return Err(DoctaviousCliError::ScmError(ScmError::UnsupportedHook(
            hook_name.to_string(),
        )));
    };

    // TODO: validate hook configuration
    if hook.parallel && hook.stop_on_failure {
        // TODO: return error
        // conflicting options 'piped' and 'parallel' are set to 'true', remove one of this option from hook group
    }

    // TODO: setup and execute hook runner
    if files.is_empty() && all_files {
        files.extend(scm.all_files()?);
    }

    let runner = ScmHookRunner::new(ScmHookRunnerOptions {
        scm: &scm,
        hook,
        hook_name: hook_name.to_string(),
        files,
        run_only_executions,
    });

    let results = runner.run_all();

    // TODO: print summary results

    let staged = scm.staged_files()?;
    let pushed = scm.push_files()?;

    // TODO: decide if hook should be skipped

    // TODO: run scripts
    // for (name, script) in &hook.scripts {}

    // TODO: run commands
    // - see if runOnlyCommands contains hook
    // - prepare command
    //   - see if should skip
    //   - check exclude_tags and tags and exclude_tags with name
    //   - validate - is runner flags compatible which checks for {staged_files} and {push_files}
    //   - build_run
    //     - filesCmd := r.Hook.Files
    //     - if command.Files > 0 set filesCmd = command.Files
    //     - if filesCmd > 0 filesCmd = replacePositionalArguments(filesCmd, r.GitArgs)
    //     - if r.Files > 0 pushFiles / allFiles / cmdFiles = r.Files
    //     - else stagedFiles = r.Repo.StagedFiles, pushFiles = r.Repo.PushFiles, allFiles = r.Repo.AllFiles, cmdFiles = r.Repo.FilesByCommand(filesCmd)
    //     - set fileFns and for each resolve template
    //     - replace run command positional args
    //     - check if hook uses staged files and skip if no matching staged
    //     - check if hook uses push files and skip if no matching push
    // - run

    // let mut files_cmd = hook.files.clone();
    // for (name, cmd) in &hook.executions {
    //     if let Some(files) = &cmd.files {
    //         files_cmd = cmd.files.clone();
    //     }
    //
    //     // make optional
    //     if !files_cmd.is_empty() {
    //         // replace positional args
    //     }
    //
    //     // what is r.Files? better way to do this?
    //     let (staged_files, push_files, all_files, cmd_files) = if !files.is_empty() {
    //         (files.clone(), files.clone(), files.clone(), files.clone())
    //     } else {
    //         (
    //             scm.staged_files()?,
    //             scm.push_files()?,
    //             scm.all_files()?,
    //             scm.files_by_command(files_cmd)?,
    //         )
    //     };
    // }

    // capture results including summary details if requested

    Ok(())
}
