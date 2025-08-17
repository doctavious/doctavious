pub mod adr;
pub mod build;
pub mod changelog;
pub mod codenotify;
pub mod codeowners;
pub mod deploy;
pub mod frameworks;
pub mod init;
pub mod link;
mod macros;
pub mod rfd;
pub mod scmhook;
pub mod til;
pub mod version;
pub mod whoami;

use anyhow::Result;

use crate::context::Context;

#[async_trait::async_trait()]
pub trait Command: Send + Sync {
    async fn execute(&self, ctx: &Context) -> Result<Option<String>>;
}
