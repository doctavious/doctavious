use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::run::{run, ScmHookRunFiles};
use doctavious_cli::CliResult;

/// Execute commands/scripts associated to the specified hook.
///
/// This is called for every hook managed by Doctavious.
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

    // TODO: better explanation as to what files mean. Explain {files}
    // TODO: can use group = "files" to only allow one to be used?
    /// Run on specified file (repeat for multiple files). takes precedence over --all-files
    #[arg(long, short)]
    pub file: Option<Vec<PathBuf>>,

    // TODO: better explanation as to what files mean. Explain {files}
    /// Run hooks on all files
    #[arg(long, short, action)]
    pub all_files: Option<bool>,

    /// Run only specified executions (commands / scripts)
    #[arg(long = "executions", short = 'e')]
    pub run_only_executions: Option<Vec<String>>,

    /// Force execution of commands that can be skipped
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: RunScmHookCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    let cmd_files = command.file.unwrap_or_default();
    let files = if !cmd_files.is_empty() {
        Some(ScmHookRunFiles::Specific(cmd_files))
    } else if let Some(true) = command.all_files {
        Some(ScmHookRunFiles::All)
    } else {
        None
    };

    run(
        &path,
        &command.hook,
        files,
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
        fs::write(
            temp_path.join("doctavious.toml"),
            r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit]
name = "pre-commit"
[scmhook_settings.hooks.pre-commit.executions.format-backend]
name = "format-backend"
type = "command"
run = "cargo fmt"
root = "backend"
tags = ["backed", "style"]
"###,
        )
            .expect("write doctavious.toml");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        // std::process::Command::new("pnpm").arg("install").output().expect("pnpm install");

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: None,
            run_only_executions: None,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());
    }

    #[test]
    fn specified_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        let c = CleanUp::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        });

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        fs::write(
            temp_path.join("doctavious.toml"),
            r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit]
name = "pre-commit"
[scmhook_settings.hooks.pre-commit.executions.specified-files]
name = "specified-files"
type = "command"
run = "echo '{files}' > test_specified_files.txt"
"###,
        )
        .expect("write doctavious.toml");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: Some(vec![PathBuf::from("/backend/src/lib.rs")]),
            all_files: None,
            run_only_executions: None,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("test_specified_files.txt")).unwrap());
    }

    #[test]
    fn all_files() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        let c = CleanUp::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        });

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        fs::write(
            temp_path.join("doctavious.toml"),
            r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit]
name = "pre-commit"
[scmhook_settings.hooks.pre-commit.executions.all-files]
name = "all-files"
type = "command"
run = "echo '{files}' > test_all_files.txt"
"###,
        )
            .expect("write doctavious.toml");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: Some(true),
            run_only_executions: None,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("test_all_files.txt")).unwrap());
    }

    #[test]
    fn script() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        let c = CleanUp::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        });

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        fs::write(
            temp_path.join("doctavious.toml"),
            r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit]
name = "pre-commit"
[scmhook_settings.hooks.pre-commit.executions.good-script]
file_name = "good-script.sh"
type = "script"
runner = "bash"
"###,
        )
            .expect("write doctavious.toml");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: None,
            run_only_executions: None,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("script_output.txt")).unwrap());
    }

    // test run_only_executions

    // test tags

    // test force

    // test script
}
