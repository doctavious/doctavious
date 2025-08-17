use std::env;
use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::build;

// TODO: needs more details about what "build" means
#[derive(Parser, Debug)]
#[command(about = "Build on your local machine")]
pub struct BuildCommand {
    /// The directory to build. Defaults to current directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    // Dry run: show instructions without running them (default: false)
    // should this just find framework and show command it will run?
    #[arg(long, short, help = "Dry run: show instructions without running them")]
    pub dry: bool,

    // not sure if this is needed
    // context Specify a build_mod context or branch (contexts: "production", "deploy-preview", "branch-deploy", "dev") (default: "production")

    // yes want this
    // option can be used to provide a working directory (that can be different from the current directory) when running CLI commands.
    // --cwd
    // pub cwd: String

    // this is global
    // The --debug option, shorthand -d, can be used to provide a more verbose output when running Vercel CLI commands.
    #[arg(long, short, help = "Skip installing dependencies")]
    pub skip_install: bool,
}

impl Default for BuildCommand {
    fn default() -> Self {
        Self {
            cwd: Some(env::current_dir().expect("Should be able to get current working directory")),
            dry: false,
            skip_install: false,
        }
    }
}

#[async_trait::async_trait]
impl crate::commands::Command for BuildCommand {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let result = build::execute(cwd, self.dry, self.skip_install)?;
        Ok(result)
    }
}
