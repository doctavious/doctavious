use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::til;
use doctavious_cli::errors::CliResult;
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

pub fn execute(cmd: InitTil) -> CliResult<Option<String>> {
    til::init(cmd.cwd.as_deref(), cmd.format, cmd.local)?;

    Ok(Some(String::new()))
}
