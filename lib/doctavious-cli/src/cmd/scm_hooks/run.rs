use std::fs;
use std::path::{Path, PathBuf};

use scm::drivers::Scm;
use scm::{ScmError, ScmRepository};

use crate::cmd::scm_hooks::{add_hook, clean};
use crate::settings::{load_settings, ScmHookSettings, SettingErrors, Settings};
use crate::{CliResult, DoctaviousCliError};

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
    all_files: bool,
    files: Vec<String>,
    commands: Option<Vec<String>>,
    force: bool,
) -> CliResult<()> {
    let mut settings: Settings = load_settings(cwd)?.into_owned();
    let Some(scm_settings) = settings.scmhook_settings else {
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

    // TODO: create hooks if needed

    // TODO: make sure hook is valid
    let Some(hook) = scm_settings.hooks.get(hook_name) else {
        // TODO: return appropriate error
        return Err(DoctaviousCliError::ScmError(ScmError::Unsupported));
    };

    // TODO: validate hook configuration

    // TODO: setup and execute hook runner
    let mut f: Vec<PathBuf> = Vec::new();
    if !files.is_empty() {
    } else if all_files {
        // get all files from scm
        f.extend(scm.all_files()?);
    } else {
    }

    // TODO: decide if hook should be skipped

    // TODO: run scripts

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

    // capture results including summary details if requested

    Ok(())
}
