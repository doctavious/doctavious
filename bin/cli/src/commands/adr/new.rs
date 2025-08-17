use std::path::PathBuf;

use clap::Parser;
use doctavious_cli::cmd::design_decisions::adr;
use doctavious_cli::templating::AdrTemplateType;
use markup::MarkupFormat;
use strum::VariantNames;

use crate::clap_enum_variants;

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
/// Creates a new, numbered ADR.
///
/// The ADR is opened for editing in the editor specified by the VISUAL or EDITOR environment variable
/// (VISUAL is preferred; EDITOR is used if VISUAL is not set).
///
/// If the CWD directory contains a file `templates/record.md`, this is used as the template for the new ADR otherwise
/// a default template is used.
#[derive(Parser, Debug)]
#[command(name = "new")]
pub struct NewADR {
    /// Provide a working directory (that can be different from the current directory) when running Doctavius CLI commands.
    /// Will use the ADR directory in settings if present or fallback to the default ADR directory.
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// ADR Number
    #[arg(long, short)]
    pub number: Option<u32>,

    /// title of ADR
    #[arg(long, short)]
    pub title: String,

    /// Format that should be used
    #[arg(
        long,
        short,
        // default_value_t = MarkupFormat::default(),
        value_parser = clap_enum_variants!(MarkupFormat)
    )]
    pub format: Option<MarkupFormat>,

    /// A reference (number or partial filename) of a previous decision that the new decision supersedes.
    /// A Markdown link to the superseded ADR is inserted into the Status section.
    /// The status of the superseded ADR is changed to record that it has been superseded by the new ADR.
    #[arg(long, short)]
    pub supersede: Option<Vec<String>>,

    /// Links the new ADR to a previous ADR with format of `TARGET:LINK:REVERSE-LINK`
    ///
    /// LINK is the description of the link created in the new ADR.
    /// TARGET is a reference number or (partial) filename of a previous decision.
    /// REVERSE-LINK is the description of the link created in the existing ADR that will refer to the new ADR.
    #[arg(long, short, value_name = "TARGET:LINK:REVERSE-LINK")]
    pub link: Option<Vec<String>>,
}

#[async_trait::async_trait]
impl crate::commands::Command for NewADR {
    async fn execute(&self) -> anyhow::Result<Option<String>> {
        let cwd = self.resolve_cwd(self.cwd.as_ref())?;
        let output = adr::new(
            &cwd,
            self.number,
            self.title.as_str(),
            AdrTemplateType::Record,
            self.format,
            self.supersede.clone(),
            self.link.clone(),
        )?;

        Ok(Some(output.to_string_lossy().to_string()))
    }
}
