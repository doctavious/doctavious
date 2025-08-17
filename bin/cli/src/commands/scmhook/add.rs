use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::add::add;

/// Create a SCM Hook.
///
/// Similar to `scmhook install` but doesn't create a configuration first.
#[derive(Parser, Debug)]
#[command()]
pub struct AddScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// SCM Hook name
    #[arg(index = 1)]
    pub name: String,

    /// Whether to create a directory for scripts. When `true` directory
    /// [`DEFAULT_CONFIG_DIR`]/scmhooks/<hook_name> will be created.
    #[arg(long, short, action)]
    pub dir: bool,

    /// Overwrite .old hooks
    #[arg(long, short, action)]
    pub force: bool,
}

#[async_trait::async_trait]
impl crate::commands::Command for AddScmHook {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        add(&cwd, self.name.clone(), self.dir, self.force)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use doctavious_cli::cmd::scm_hooks::HOOK_TEMPLATE;
    use doctavious_std::fs::copy_dir;
    use scm::drivers::ScmRepository;
    use scm::drivers::git::GitScmRepository;
    use scm::hooks::OLD_HOOK_POSTFIX;
    use tempfile::TempDir;
    use testing::cleanup::CleanUp;

    use crate::commands::Command;
    use crate::commands::scmhook::add::AddScmHook;

    #[tokio::test]
    async fn should_fail_if_scm_not_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.keep();
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_err());
        assert_eq!(
            "SCM error: Could not find supported SCM",
            result.unwrap_err().to_string()
        );
    }

    #[tokio::test]
    async fn should_fail_with_invalid_hook() {
        let (temp_path, _) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "invalid-hook".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_err());
        assert_eq!(
            "SCM error: Hook invalid-hook is not supported",
            result.unwrap_err().to_string()
        );
    }

    #[tokio::test]
    async fn should_add_without_doctavious_configuration() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        assert!(scm.hooks_path().unwrap().join("pre-commit").exists());
        assert!(!temp_path.join(".doctavious/scmhooks/pre-commit").exists());
    }

    #[tokio::test]
    async fn should_add_with_doctavious_configured() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        let pre_commit_path = scm.hooks_path().unwrap().join("pre-commit");
        assert!(pre_commit_path.exists());
        assert!(
            fs::read_to_string(pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );
    }

    #[tokio::test]
    async fn should_create_hooks_script_configuration_directory() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: true,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        assert!(scm.hooks_path().unwrap().join("pre-commit").exists());
        assert!(temp_path.join(".doctavious/scmhooks/pre-commit").is_dir());
    }

    #[tokio::test]
    async fn should_replace_existing_hook() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        assert!(pre_commit_path.exists());
        assert!(
            fs::read_to_string(pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );
        assert!(scm_hooks_path.join("pre-commit.old").exists());
    }

    #[tokio::test]
    async fn should_replace_existing_doctavious_hook() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        fs::write(&pre_commit_path, HOOK_TEMPLATE).unwrap();

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        assert!(pre_commit_path.exists());
        assert!(
            fs::read_to_string(pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );
        assert!(!scm_hooks_path.join("pre-commit.old").exists());
    }

    #[tokio::test]
    async fn should_fail_with_existing_old_hook() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let old_pre_commit_path =
            doctavious_std::path::append_to_path(&pre_commit_path, OLD_HOOK_POSTFIX);
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&old_pre_commit_path, "some old hook content").unwrap();

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: false,
        };

        let result = cmd.execute().await;

        assert!(result.is_err());
        assert_eq!(
            "SCM error: Can't rename pre-commit to pre-commit.old as file already exists. If you wish to overwrite use 'force' option",
            result.unwrap_err().to_string()
        );

        let hooks_path = scm.hooks_path().unwrap();
        assert!(hooks_path.join("pre-commit").exists());
        assert!(hooks_path.join("pre-commit.old").exists());
    }

    #[tokio::test]
    async fn should_replace_hook_and_overwrite_old_with_force() {
        let (temp_path, scm) = setup(None);
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let old_pre_commit_path =
            doctavious_std::path::append_to_path(&pre_commit_path, OLD_HOOK_POSTFIX);
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&old_pre_commit_path, "some old hook content").unwrap();

        let cmd = AddScmHook {
            cwd: Some(temp_path.clone()),
            name: "pre-commit".to_string(),
            dir: false,
            force: true,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());
        assert!(pre_commit_path.exists());
        assert!(
            fs::read_to_string(pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );
        assert_eq!(
            "some hook content",
            fs::read_to_string(&old_pre_commit_path).unwrap()
        );
    }

    fn setup(doctavous_config: Option<&str>) -> (PathBuf, GitScmRepository) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.keep();

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        if let Some(config) = doctavous_config {
            fs::write(temp_path.join("doctavious.toml"), config).expect("write doctavious.toml");
        }

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all().expect("Should add all files to SCM");

        (temp_path, scm)
    }
}
