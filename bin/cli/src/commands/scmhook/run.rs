use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::run::run;
use doctavious_cli::CliResult;

/// Execute commands/scripts associated to the specified hook.
///
/// This is called for every hook managed by doctavious.
/// You can also provide your own hooks that can only be called manually.
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RunScmHookCommand {
    /// Name of the hook to run
    #[arg(index = 1)]
    pub hook: String,

    /// Path to execute run
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    // TODO: can use group = "files" to only allow one to be used?
    /// Run on specified file (repeat for multiple files). takes precedence over --all-files
    #[arg(long, short)]
    pub file: Option<Vec<PathBuf>>,

    /// Run hooks on all files
    #[arg(long, short, action)]
    pub all_files: bool,

    /// Run only specified executions (commands / scripts)
    #[arg(long = "executions", short = 'e')]
    pub run_only_executions: Option<Vec<String>>,

    /// Force execution of commands that can be skipped
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: RunScmHookCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    // TODO: turn all_files / files into an enum

    run(
        &path,
        &command.hook,
        command.all_files,
        command.file.unwrap_or_default(),
        command.run_only_executions.unwrap_or_default(),
        command.force,
    )?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use common::fs::copy_dir;
    use doctavious_cli::CliResult;
    use scm::drivers::git::GitScmRepository;
    use tempfile::TempDir;
    use testing::CleanUp;

    use crate::commands::scmhook::run::{execute, RunScmHookCommand};

    #[test]
    fn execute_hook() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        let c = CleanUp::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        });

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: true,
            run_only_executions: None,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/lib.rs")).unwrap());
    }

    // test all_files

    // test run_only_executions

    // test specific files

    // test force
}
