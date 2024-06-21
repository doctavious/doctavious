use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::uninstall::uninstall;
use doctavious_cli::CliResult;

/// Clear hooks related to Doctavious configuration
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct UninstallScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Flag to remove all SCM hooks even those not related to doctavious
    #[arg(long, short, action)]
    pub force: bool,

    /// Flag to remove SCM hook configuration from doctavious configuration
    #[arg(long, short, action)]
    pub remove_settings: bool,
}

pub(crate) fn execute(command: UninstallScmHook) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    uninstall(&path, command.force, command.remove_settings)?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use common::fs::copy_dir;
    use scm::drivers::git::GitScmRepository;
    use scm::drivers::ScmRepository;
    use scm::HOOK_TEMPLATE;
    use tempfile::TempDir;
    use testing::cleanup::CleanUp;

    use crate::commands::scmhook::uninstall::{execute, UninstallScmHook};

    #[test]
    fn should_only_delete_doctavious_hooks() {
        let (temp_path, scm) = setup(Some(""));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let post_commit_path = scm_hooks_path.join("post-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&post_commit_path, HOOK_TEMPLATE).unwrap();

        let result = execute(UninstallScmHook {
            cwd: Some(temp_path.clone()),
            force: false,
            remove_settings: false,
        });

        assert!(result.is_ok());

        let hooks_path = scm.hooks_path().unwrap();
        assert!(hooks_path.join("pre-commit").exists());
        assert!(!hooks_path.join("post-commit").exists());
        // TODO: confirm config
    }

    #[test]
    fn should_delete_all_hooks_when_forced() {
        let (temp_path, scm) = setup(Some(""));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let post_commit_path = scm_hooks_path.join("post-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&post_commit_path, HOOK_TEMPLATE).unwrap();

        let result = execute(UninstallScmHook {
            cwd: Some(temp_path.clone()),
            force: true,
            remove_settings: false,
        });

        assert!(result.is_ok());

        let hooks_path = scm.hooks_path().unwrap();
        assert!(!hooks_path.join("pre-commit").exists());
        assert!(!hooks_path.join("post-commit").exists());
        // TODO: confirm config
    }

    // TODO: uninstall with remove_settings
    #[test]
    fn should_delete_config_when_remove_settings_true() {
        let (temp_path, scm) = setup(Some(""));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let post_commit_path = scm_hooks_path.join("post-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&post_commit_path, HOOK_TEMPLATE).unwrap();

        let result = execute(UninstallScmHook {
            cwd: Some(temp_path.clone()),
            force: true,
            remove_settings: false,
        });

        assert!(result.is_ok());

        let hooks_path = scm.hooks_path().unwrap();
        assert!(!hooks_path.join("pre-commit").exists());
        assert!(!hooks_path.join("post-commit").exists());
        // TODO: confirm config
    }

    #[test]
    fn should_recover_old_files() {
        let (temp_path, scm) = setup(Some(""));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();

        let post_commit_path = scm_hooks_path.join("post-commit");
        fs::write(&post_commit_path, HOOK_TEMPLATE).unwrap();

        let post_commit_old_path = scm_hooks_path.join("post-commit.old");
        fs::write(&post_commit_old_path, "old hook content").unwrap();

        let result = execute(UninstallScmHook {
            cwd: Some(temp_path.clone()),
            force: false,
            remove_settings: false,
        });

        assert!(result.is_ok());

        let hooks_path = scm.hooks_path().unwrap();
        assert!(hooks_path.join("pre-commit").exists());

        let post_commit_path = hooks_path.join("post-commit");
        assert!(post_commit_path.exists());
        assert_eq!(
            "old hook content",
            fs::read_to_string(post_commit_path).unwrap()
        );
        assert!(!hooks_path.join("post-commit.old").exists());
        // TODO: confirm config
    }

    fn setup(doctavous_config: Option<&str>) -> (PathBuf, GitScmRepository) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.into_path();

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        if let Some(config) = doctavous_config {
            fs::write(temp_path.join("doctavious.toml"), config).expect("write doctavious.toml");
        }

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all();

        (temp_path, scm)
    }
}
