use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::scm_hooks::install::install;

/// Synchronize SCM hooks with your configuration.
#[derive(Parser, Debug)]
#[command()]
pub struct InstallScmHook {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Overwrite .old hooks
    #[arg(long, short, action)]
    pub force: bool,
}

#[async_trait::async_trait]
impl crate::commands::Command for InstallScmHook {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        install(&cwd, self.force)?;
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use doctavious_cli::cmd::scm_hooks::{HOOK_TEMPLATE, HOOK_TEMPLATE_CHECKSUM};
    use doctavious_cli::settings::Config;
    use doctavious_std::fs::copy_dir;
    use scm::drivers::ScmRepository;
    use scm::drivers::git::GitScmRepository;
    use scm::hooks::OLD_HOOK_POSTFIX;
    use tempfile::TempDir;
    use testing::cleanup::CleanUp;

    use crate::commands::Command;
    use crate::commands::scmhook::install::InstallScmHook;

    // TODO: should_install_without_doctavious_config

    #[tokio::test]
    async fn should_install_with_existing_doctavious_config() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let cmd = InstallScmHook {
            cwd: Some(temp_path.clone()),
            force: false,
        };

        let result = cmd.execute().await;

        let hooks_path = scm.hooks_path().unwrap();
        assert!(result.is_ok());
        assert!(hooks_path.join("pre-commit").exists());
        assert!(hooks_path.join("post-commit").exists());

        let synchronization_file = scm.info_path().unwrap().join("doctavious.synchronization");
        assert!(synchronization_file.exists());
        assert_eq!(
            format!(
                "{}|ae6e1c36d1f298f4692eed15be33a2ae",
                HOOK_TEMPLATE_CHECKSUM.as_str()
            ),
            fs::read_to_string(synchronization_file).unwrap()
        );
    }

    #[tokio::test]
    async fn should_install_with_existing_hooks() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        fs::write(&pre_commit_path, "some hook content").unwrap();

        let cmd = InstallScmHook {
            cwd: Some(temp_path.clone()),
            force: false,
        };

        let result = cmd.execute().await;

        let hooks_path = scm.hooks_path().unwrap();
        assert!(result.is_ok());
        assert!(hooks_path.join("pre-commit").exists());
        assert!(
            fs::read_to_string(&pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );

        let old_hook_path = hooks_path.join("pre-commit.old");
        assert!(old_hook_path.exists());
        assert_eq!(
            "some hook content",
            fs::read_to_string(&old_hook_path).unwrap()
        );
        assert!(hooks_path.join("post-commit").exists());
        assert!(
            scm.info_path()
                .unwrap()
                .join("doctavious.synchronization")
                .exists()
        );
    }

    #[tokio::test]
    async fn should_install_with_existing_doctavious_hooks() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        fs::write(&pre_commit_path, HOOK_TEMPLATE).unwrap();

        let cmd = InstallScmHook {
            cwd: Some(temp_path.clone()),
            force: false,
        };

        let result = cmd.execute().await;

        let hooks_path = scm.hooks_path().unwrap();
        assert!(result.is_ok());
        assert!(hooks_path.join("pre-commit").exists());
        assert!(
            fs::read_to_string(&pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );

        assert!(!hooks_path.join("pre-commit.old").exists());
    }

    #[tokio::test]
    async fn should_update_stale_synchronization_file() {
        for (hook_template_checksum, settings_checksum) in vec![
            ("123abc", "ae6e1c36d1f298f4692eed15be33a2ae"), // stale hook template checksum
            (HOOK_TEMPLATE_CHECKSUM.as_str(), "123abc"),    // stale settings checksum
        ] {
            let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

            let (temp_path, scm) = setup(Some(config));
            let _c = CleanUp::new(Box::new(|| {
                let _ = fs::remove_dir_all(&temp_path);
            }));

            let scm_hooks_path = scm.hooks_path().unwrap();
            let pre_commit_path = scm_hooks_path.join("pre-commit");
            fs::write(&pre_commit_path, HOOK_TEMPLATE).unwrap();

            let synchronization_file = scm.info_path().unwrap().join("doctavious.synchronization");
            fs::write(&synchronization_file, "123|").unwrap();

            let cmd = InstallScmHook {
                cwd: Some(temp_path.clone()),
                force: false,
            };

            let result = cmd.execute().await;

            let hooks_path = scm.hooks_path().unwrap();
            assert!(result.is_ok());
            assert!(hooks_path.join("pre-commit").exists());
            assert!(
                fs::read_to_string(&pre_commit_path)
                    .unwrap()
                    .contains("doctavious")
            );

            assert!(!hooks_path.join("pre-commit.old").exists());
            assert!(hooks_path.join("post-commit").exists());

            assert_eq!(
                format!(
                    "{}|ae6e1c36d1f298f4692eed15be33a2ae",
                    HOOK_TEMPLATE_CHECKSUM.as_str()
                ),
                fs::read_to_string(synchronization_file).unwrap()
            );
        }
    }

    #[tokio::test]
    async fn should_fail_with_existing_hook_and_old() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let old_pre_commit_path =
            doctavious_std::path::append_to_path(&pre_commit_path, OLD_HOOK_POSTFIX);
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&old_pre_commit_path, "some old hook content").unwrap();

        let cmd = InstallScmHook {
            cwd: Some(temp_path.clone()),
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
        assert!(!hooks_path.join("post-commit").exists());
        assert!(
            !scm.info_path()
                .unwrap()
                .join("doctavious.synchronization")
                .exists()
        );
    }

    #[tokio::test]
    async fn should_install_with_existing_hook_and_old_with_force() {
        let config = r###"[scmhook_settings]
[scmhook_settings.hooks.pre-commit.executions.format-backend]
type = "command"
run = "cargo fmt"
root = "backend"
[scmhook_settings.hooks.post-commit.executions.echo]
type = "command"
run = "echo 'Done!'"
"###;

        let (temp_path, scm) = setup(Some(config));
        let _c = CleanUp::new(Box::new(|| {
            let _ = fs::remove_dir_all(&temp_path);
        }));

        let scm_hooks_path = scm.hooks_path().unwrap();
        let pre_commit_path = scm_hooks_path.join("pre-commit");
        let old_pre_commit_path =
            doctavious_std::path::append_to_path(&pre_commit_path, OLD_HOOK_POSTFIX);
        fs::write(&pre_commit_path, "some hook content").unwrap();
        fs::write(&old_pre_commit_path, "some old hook content").unwrap();

        let cmd = InstallScmHook {
            cwd: Some(temp_path.clone()),
            force: true,
        };

        let result = cmd.execute().await;

        assert!(result.is_ok());

        let hooks_path = scm.hooks_path().unwrap();
        assert!(hooks_path.join("pre-commit").exists());
        assert!(
            fs::read_to_string(pre_commit_path)
                .unwrap()
                .contains("doctavious")
        );

        assert!(hooks_path.join("pre-commit.old").exists());
        assert_eq!(
            "some hook content",
            fs::read_to_string(old_pre_commit_path).unwrap()
        );

        let post_commit_path = hooks_path.join("post-commit");
        assert!(post_commit_path.exists());
        assert!(
            fs::read_to_string(post_commit_path)
                .unwrap()
                .contains("doctavious")
        );
        assert!(
            scm.info_path()
                .unwrap()
                .join("doctavious.synchronization")
                .exists()
        );
    }

    fn setup(doctavous_config: Option<&str>) -> (PathBuf, GitScmRepository) {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.keep();

        copy_dir("./tests/fixtures/scmhook/", &temp_path).expect("copy test fixtures");
        if let Some(config) = doctavous_config {
            fs::write(temp_path.join(Config::config_file_path()), config)
                .expect("write doctavious.toml");
        }

        let scm = GitScmRepository::init(&temp_path).expect("init git");
        scm.add_all().expect("Should add all files to SCM");

        (temp_path, scm)
    }
}
