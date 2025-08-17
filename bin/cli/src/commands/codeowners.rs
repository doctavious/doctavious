mod audit;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command()]
pub struct CodeOwnersCommand {
    #[command(subcommand)]
    sub_command: CodeOwnersSubcommand,
}

#[remain::sorted]
#[derive(Debug, Subcommand)]
enum CodeOwnersSubcommand {
    // TODO: do we want to check all repositories
    // Do we want to check for consistency
    // See references
    // - https://github.com/sigio/github-audit-org-codeowner
    // - https://github.com/toptal/codeowners-checker
    // - https://github.com/timdawborn/github-codeowners-checker
    // /// Check for hte presence of a CODEOWNERS file and check if all listed users and groups in the
    // /// file have actual write-permissions
    Audit(AuditCommand),
    Unowned(UnownedCommand),
}

#[async_trait::async_trait]
impl crate::commands::Command for CodeOwnersCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        match &self.sub_command {
            CodeOwnersSubcommand::Audit(cmd) => cmd.execute().await,
            CodeOwnersSubcommand::Unowned(cmd) => cmd.execute().await,
        }
    }
}

#[derive(Args, Debug)]
#[command()]
pub struct AuditCommand {}

#[async_trait::async_trait]
impl crate::commands::Command for AuditCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}

#[derive(Args, Debug)]
#[command()]
pub struct UnownedCommand {}

#[async_trait::async_trait]
impl crate::commands::Command for UnownedCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        unimplemented!()
    }
}
