use clap::builder::PossibleValuesParser;
use clap::Parser;
use doctavious_cli::file_structure::FileStructure;
use doctavious_cli::markup_format::MarkupFormat;

/// Gathers ADR management commands
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ADRCommand {
    #[command(subcommand)]
    pub adr_command: ADRSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum ADRSubCommand {
    Generate(GenerateADRs),
    Init(InitADR),
    Link(LinkADRs),
    List(ListADRs),
    New(NewADR),
    Reserve(ReserveADR),
}

/// Initialises the directory of architecture decision records:
/// * creates a subdirectory of the current working directory
/// * creates the first ADR in that subdirectory, recording the decision to record architectural decisions with ADRs.
#[derive(Parser, Debug)]
#[command(name = "init")]
pub(crate) struct InitADR {
    /// Directory to store ADRs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// How ADRs should be structured
    #[arg(
        value_enum,
        long,
        short,
        default_value = "FileStructure::default()",
        value_parser = PossibleValuesParser::new(FileStructure::variants()),
        // default_value_t,
        // value_parser = parse_file_structure,
    )]
    pub structure: FileStructure,

    /// Extension that should be used
    #[arg(
        long,
        short,
        // value_parser = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants())
    )]
    pub extension: Option<MarkupFormat>,
}

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
/// New ADR
#[derive(Parser, Debug)]
#[command(name = "new")]
pub(crate) struct NewADR {
    /// ADR Number
    #[arg(long, short)]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of ADR
    #[arg(long, short)]
    pub title: String,

    /// Extension that should be used
    #[arg(
    long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants())
    )]
    pub extension: Option<MarkupFormat>,

    /// A reference (number or partial filename) of a previous decision that the new decision supersedes.
    /// A Markdown link to the superseded ADR is inserted into the Status section.
    /// The status of the superseded ADR is changed to record that it has been superseded by the new ADR.
    #[arg(long, short)]
    pub supersede: Option<Vec<String>>,

    // Links the new ADR to a previous ADR.
    // TARGET is a reference (number or partial filename) of a
    // previous decision.
    // LINK is the description of the link created in the new ADR.
    // REVERSE-LINK is the description of the link created in the
    // existing ADR that will refer to the new ADR.
    #[arg(long, short, help = "")]
    pub link: Option<Vec<String>>,
}

/// List ADRs
#[derive(Parser, Debug)]
#[command(name = "list")]
pub(crate) struct ListADRs {}

/// Link ADRs
#[derive(Parser, Debug)]
#[command(name = "link")]
pub(crate) struct LinkADRs {
    /// Reference number of source ADR
    #[arg(long, short)]
    pub source: i32,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// Description of the link created in the new ADR
    #[arg(long, short)]
    pub link: String,

    #[arg(long, short, help = "Reference number of target ADR")]
    pub target: i32,

    /// Description of the link created in the existing ADR that will refer to new ADR
    #[arg(long, short)]
    pub reverse_link: String,
}

/// Gathers generate ADR commands
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct GenerateADRs {
    #[command(subcommand)]
    pub generate_adr_command: GenerateAdrsCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

/// Generates ADR table of contents (Toc) to stdout
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AdrToc {
    #[arg(long, short, help = "")]
    pub intro: Option<String>,

    #[arg(long, help = "")]
    pub outro: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,

    /// Output format
    #[arg(
        long,
        short,
        // value_parser = parse_markup_format_extension,
        // value_parser,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants())
    )]
    pub format: Option<MarkupFormat>,
}

/// Create ADR Graph
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct AdrGraph {
    /// Directory of ADRs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short, help = "")]
    pub link_extension: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,
}

/// Reserve ADR
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub(crate) struct ReserveADR {
    /// ADR Number
    #[arg(long, short)]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of ADR
    #[arg(long, short)]
    pub title: String,

    /// Extension that should be used
    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        // value_parser,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants())
    )]
    pub extension: Option<MarkupFormat>,
}
