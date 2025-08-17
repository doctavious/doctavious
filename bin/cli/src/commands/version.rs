use clap::Parser;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[derive(Parser, Debug, Clone)]
pub struct VersionCommand;

#[async_trait::async_trait]
impl crate::commands::Command for VersionCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        println!("Doctavious version: {}", built_info::PKG_VERSION);
        println!(
            "Built from commit: {} {}",
            built_info::GIT_COMMIT_HASH.unwrap(),
            if matches!(built_info::GIT_DIRTY, Some(true)) {
                "(dirty)"
            } else {
                ""
            }
        );

        // TODO: include Doctavious API version
        Ok(None)
    }
}
