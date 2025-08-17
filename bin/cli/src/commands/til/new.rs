use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til;

/// New TIL
#[derive(Parser, Debug)]
#[command()]
pub struct NewTil {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Additional tags associated with the TIL entry
    #[arg(short, long)]
    pub tags: Option<Vec<String>>,

    // TODO: should this be an action?
    // TODO: should this also be a setting in TilSettings?
    /// Whether to build_mod a README after a new TIL is added
    #[arg(long)]
    pub toc: bool,

    /// The post, in the format of <category/title>, to create.
    ///
    /// The category will be the folder while title will be used for the filename. You can also include an extension
    #[arg(index = 1)]
    pub post: String,
}

#[async_trait::async_trait]
impl crate::commands::Command for NewTil {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        // let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = til::new(
            self.cwd.as_deref(),
            self.post.clone(),
            self.tags.clone(),
            self.toc.clone(),
        )?;
        Ok(Some(output.to_string_lossy().to_string()))
    }
}
