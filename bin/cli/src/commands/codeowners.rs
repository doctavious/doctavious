mod audit;

use clap::{Args, Parser, Subcommand};
use doctavious_cli::errors::CliResult;

#[derive(Parser, Debug)]
#[command()]
pub struct CodeOwnersCli {
    #[command(subcommand)]
    command: Commands,
}

#[remain::sorted]
#[derive(Debug, Subcommand)]
enum Commands {
    // TODO: do we want to check all repositories
    // Do we want to check for consistency
    // See references
    // - https://github.com/sigio/github-audit-org-codeowner
    // - https://github.com/toptal/codeowners-checker
    // - https://github.com/timdawborn/github-codeowners-checker
    // /// Check for hte presence of a CODEOWNERS file and check if all listed users and groups in the
    // /// file have actual write-permissions
    // Audit(AuditCommand),
    Unowned(UnownedCommand),
}

#[derive(Args, Debug)]
#[command()]
pub struct AuditCommand {}

#[derive(Args, Debug)]
#[command()]
pub struct UnownedCommand {}

pub fn execute(cli: CodeOwnersCli) -> CliResult<Option<String>> {
    match cli.command {
        Commands::Unowned(_) => {}
    };

    Ok(Some(String::new()))
}
