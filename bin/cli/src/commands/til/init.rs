use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

/// Init Today I Learned (TIL)
///
/// TODO: explain creation of configurations local vs global
#[derive(Parser, Debug)]
#[command()]
pub struct InitTil {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Format that should be used
    #[arg(
        long,
        short,
        default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: MarkupFormat,

    /// Initialize a local TIL directory, with local configuration, at `CWD` option if provided or in the current directory.
    /// When not provided a global TIL will be created at `CWD` option if provided or in the `$HOME` directory with a
    /// global configuration
    #[arg(long, action)]
    pub local: bool,
    // TODO: should we have toc details?
}

#[async_trait::async_trait]
impl crate::commands::Command for InitTil {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        // let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        til::init(self.cwd.as_deref(), self.format, self.local)?;
        Ok(None)
    }
}
