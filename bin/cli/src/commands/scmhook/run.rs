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
    /// Run on specified file (repeat for multiple files).
    #[arg(long, group = "files")]
    pub file: Option<Vec<PathBuf>>,

    // TODO: better explanation as to what files mean. Explain {files}
    /// Run hooks on all files
    #[arg(long, action, group = "files")]
    pub all_files: bool,

    /// Run only specified executions (commands / scripts)
    #[arg(long = "executions", short = 'e')]
    pub run_only_executions: Option<Vec<String>>,

    /// Skip synchronization (installing/updating) hooks
    #[arg(long, action)]
    pub skip_auto_synchronize: bool,

    /// Force execution of commands that can be skipped
    #[arg(long, short, action)]
    pub force: bool,
}

pub(crate) fn execute(command: RunScmHookCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    let cmd_files = command.file.unwrap_or_default();
    let files = if !cmd_files.is_empty() {
        Some(ScmHookRunFiles::Specific(cmd_files))
    } else if command.all_files {
        Some(ScmHookRunFiles::All)
    } else {
        None
    };

    run(
        &path,
        &command.hook,
        files,
        command.run_only_executions.unwrap_or_default(),
        !command.skip_auto_synchronize,
        command.force,
    )?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};

    use common::fs::copy_dir;
    use doctavious_cli::CliResult;
    use scm::drivers::git::GitScmRepository;
    use scm::hooks::OLD_HOOK_POSTFIX;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;
    use testing::cleanup::CleanUp;

    use crate::commands::scmhook::install::InstallScmHook;
    use crate::commands::scmhook::run::{execute, RunScmHookCommand};

    #[test]
    fn execute_hook() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
tags = ["backed", "style"]
"###;

        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());
    }

    #[test]
    fn specified_files() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.specified-files]
name = "specified-files"
type = "command"
run = "echo '{files}' > test_specified_files.txt"
"###;

        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: Some(vec![PathBuf::from("/backend/src/lib.rs")]),
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(
            fs::read_to_string(&temp_path.join("test_specified_files.txt")).unwrap()
        );
    }

    #[test]
    fn all_files() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.all-files]
name = "all-files"
type = "command"
run = "echo '{files}' > test_all_files.txt"
"###;
        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: true,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("test_all_files.txt")).unwrap());
    }

    #[test]
    fn script() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.script]
file_name = "good-script.sh"
type = "script"
runner = "bash"
"###;

        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("script_output.txt")).unwrap());
    }

    #[test]
    fn run_only_executions() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.pre-commit.executions.script]
file_name = "good-script.sh"
type = "script"
runner = "bash"
"###;
        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: Some(vec!["format-backend".to_string()]),
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());
        assert!(!&temp_path.join("script_output.txt").exists())
    }

    #[test]
    fn force() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
skip = true
"###;

        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        // first confirm that we skip execution...
        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: false,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());

        // then confirm that execution is run when force is set to true...
        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: true,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());
    }

    #[test]
    fn should_handle_parallel_processing() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.pre-commit.executions.script]
file_name = "good-script.sh"
type = "script"
runner = "bash"
"###;

        let temp_path = setup(config);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let result = execute(RunScmHookCommand {
            hook: "pre-commit".to_string(),
            cwd: Some(temp_path.clone()),
            file: None,
            all_files: false,
            run_only_executions: None,
            skip_auto_synchronize: false,
            force: true,
        });

        assert!(result.is_ok());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("backend/src/lib.rs")).unwrap());
        assert!(&temp_path.join("script_output.txt").exists());
        insta::assert_snapshot!(fs::read_to_string(&temp_path.join("script_output.txt")).unwrap());
    }

    fn setup(doctavous_config: &str) -> PathBuf {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        fs::write(temp_path.join("doctavious.toml"), doctavous_config)
            .expect("write doctavious.toml");

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        temp_path
    }
}
