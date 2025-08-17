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

use std::path::PathBuf;
use std::{env, io};

use anyhow::Result;

use crate::context::Context;

#[async_trait::async_trait]
pub trait Command: Send + Sync {
    // TODO: include ctx: &Context
    async fn execute(&self) -> Result<Option<String>>;

    fn resolve_cwd(&self, cwd: Option<&PathBuf>) -> io::Result<PathBuf> {
        match cwd {
            Some(p) => Ok(p.clone()),
            None => env::current_dir(),
        }
    }
}
