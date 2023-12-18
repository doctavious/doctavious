use std::env;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use clap::builder::PossibleValuesParser;
use doctavious_cli::file_structure::FileStructure;
use doctavious_cli::markup_format::MarkupFormat;

#[derive(Parser, Debug)]
#[command(about = "Build on your local machine")]
pub(crate) struct BuildCommand {
    #[arg(
        long,
        short,
        help = "The directory to build. Defaults to current directory."
    )]
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

#[derive(Parser, Debug)]
#[command(about = "Create a new deploy from the contents of a folder")]
pub(crate) struct DeployCommand {
    #[arg(
        long,
        short,
        help = "The directory to build. Defaults to current directory."
    )]
    pub cwd: Option<PathBuf>,

    #[arg(
        long,
        short,
        help = "Specifies the alias for deployment, the string at the beginning of the deploy subdomain. Useful for creating predictable deployment URLs."
    )]
    pub alias: Option<String>,

    #[arg(long, short, help = "Deploy to Production")]
    pub prod: bool,

    // build / --prebuilt': Boolean,
    // TODO: include details that this will fail if build is false and output dir is not found/empty
    #[arg(long, short, help = "Whether to build prior to deploy.")]
    pub build: bool,
    // -a --auth <token>
    // --build_mod Run build_mod command before deploying

    // -m, --message <message> A short message to include in the deploy log

    // -o, --open Open site after deploy (default: false)

    // vercel had something similar called name which is deprecated in favor of linking...i prefer
    // the link as well
    // -s, --site <name-or-id> A site name or ID to deploy to

    // --timeout <number>  Timeout to wait for deployment to finish

    // '-y': '--yes', is autoConfirm

    // allow for build-env
}

#[derive(Parser, Debug)]
#[command(about = "")]
pub(crate) struct FrameworksCommand {
    #[command(subcommand)]
    pub framework_command: FrameworkSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum FrameworkSubCommand {
    Detect(DetectFrameworks),
    Get(GetFramework),
    List(ListFrameworks),
}

#[derive(Parser, Debug)]
#[command(about = "Detect Frameworks")]
pub(crate) struct DetectFrameworks {
    #[arg(long, short, help = "Directory to detect framewoks in")]
    pub cwd: Option<PathBuf>,
}

#[derive(Parser, Debug)]
#[command(about = "List Frameworks")]
pub(crate) struct ListFrameworks {}

#[derive(Parser, Debug)]
#[command(about = "Get Framework Details")]
pub(crate) struct GetFramework {
    #[arg(long, short, help = "Name of the framework")]
    pub name: String,
}

#[derive(Parser, Debug)]
#[command(about = "Show the username of the user currently logged into Doctavious CLI.")]
pub(crate) struct WhoAmICommand;

#[derive(Parser, Debug)]
#[command(about = "Initialize Doctavious Projects locally")]
pub(crate) struct InitCommand {
    #[arg(help = "Name of the Project", index = 0)]
    pub name: Option<String>,
}

#[derive(Parser, Debug)]
#[command(about = "Links your local directory to a Doctavious Project.")]
pub(crate) struct LinkCommand {
    #[arg(
        long,
        short,
        help = "Directory of the local Doctavious project",
        index = 0
    )]
    pub cwd: Option<PathBuf>,
    // TODO: auto confirm
}

#[derive(Parser, Debug)]
#[command(about = "Gathers ADR management commands")]
pub(crate) struct ADRCommand {
    #[command(subcommand)]
    pub adr_command: ADRSubCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum ADRSubCommand {
    Init(InitADR),
    Generate(GenerateADRs),
    List(ListADRs),
    Link(LinkADRs),
    New(NewADR),
    Reserve(ReserveADR),
}

#[derive(Parser, Debug)]
#[command(name = "init", about = "Init ADR")]
pub(crate) struct InitADR {
    /// Directory to store ADRs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// How ADRs should be structured
    #[arg(value_parser = PossibleValuesParser::new(FileStructure::variants()))]
    #[arg(
        value_enum,
        long,
        short,
        default_value = "FileStructure::default()",
        // default_value_t,
        // value_parser = parse_file_structure,
    )]
    pub structure: FileStructure,

    /// Extension that should be used
    #[arg(value_parser = PossibleValuesParser::new(MarkupFormat::variants()))]
    #[arg(
        long,
        short,
        // value_parser = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser,
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
        value_parser,
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
        value_parser,
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
        value_parser,
    )]
    pub extension: Option<MarkupFormat>,
}

#[derive(Parser, Debug)]
#[command(about = "Gathers RFD management commands")]
pub(crate) struct RFDCommand {
    #[command(subcommand)]
    pub rfd_command: RFDSubCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RFDSubCommand {
    Init(InitRFD),
    New(NewRFD),
    List(ListRFDs),
    Generate(GenerateRFDs),
    Reserve(ReserveRFD),
}

/// Init RFD
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct InitRFD {
    /// Directory to store RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: should we default here?
    /// How RFDs should be structured
    #[arg(value_parser = PossibleValuesParser::new(FileStructure::variants()))]
    #[arg(
        value_enum,
        long,
        short,
        default_value = "FileStructure::default()",
        // default_value_t,
        // value_parser = parse_file_structure,
    )]
    pub structure: FileStructure,

    /// Extension that should be used
    #[arg(
        long,
        short,
        default_value = "MarkupFormat::default()",
        // default_value_t,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
    )]
    pub extension: MarkupFormat,
}

/// New RFD
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct NewRFD {
    /// RFD number
    #[arg(long, short)]
    pub number: Option<i32>,

    /// title of RFD
    #[arg(long, short)]
    pub title: String,

    /// Extension that should be used
    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
    )]
    pub extension: Option<MarkupFormat>,
}

/// List RFDs
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ListRFDs {}

/// Gathers generate RFD commands
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct GenerateRFDs {
    #[command(subcommand)]
    pub generate_rfd_command: GenerateRFDsCommand,
}

// TODO: flush this out more?
// keeping ToC is probably fine
// but also want to generate CSV
// Generate README / index file
// Update README with table (maybe even list)
#[derive(Parser, Debug)]
pub(crate) enum GenerateRFDsCommand {
    Toc(RFDToc), // template, csv file. what is the snippet?
    Csv(RFDCsv),
    File(RFDFile),
    // TODO: CSV - path, if exists just update. What about supporting it in another branch/remote. what about committing to that branch? flag for commit and commit message?
    // TODO: File - // template and path to where file should be created
    Graph(RFDGraph),
}

// optional file means to stdout
// add overwrite flag to not modify existing
// remote? commit message?
/// Generates RFD CSV
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RFDCsv {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    // #[clap(parse(from_os_str)] -> #[clap(value_parser)]
    // output_path
    #[arg(value_parser, long, short, help = "")]
    pub path: Option<PathBuf>, // where to write file to. stdout if not provided

    #[arg(long, short, help = "")]
    pub fields: Vec<String>, // which fields to include? default to all (*). should this just be a comma separate list?

    #[arg(long, short, help = "")]
    pub overwrite: bool,
}

/// Generates RFD File
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RFDFile {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// Template that will be used to generate file.
    /// If not present use value from config otherwise default template based on output_path extension
    /// will be used. See <location> for default template
    #[arg(long, short)]
    pub template: Option<String>, // optional. use config, use provided here. use default

    // output_path
    /// Path to file which to write table of contents to. File must contain snippet.
    /// If not present ToC will be written to stdout
    #[arg(long, short, value_parser)]
    pub path: PathBuf, // where to write file to. required
}

/// Generates RFD table of contents (Toc) to stdout
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RFDToc {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    /// Template that will be used to generate file.
    /// If not present use value from config otherwise default template based on
    /// output_path extension will be used. See <location> for default template
    #[arg(long, short)]
    pub template: Option<String>, // optional. use config, use provided here. use default

    /// Path to file which to write table of contents to. File must contain snippet.
    /// If not present ToC will be written to stdout
    #[arg(long, short, value_parser)]
    pub output_path: PathBuf, // where to write file to. required

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
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
    )]
    pub format: Option<MarkupFormat>,
}

/// Create RFD Graph
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct RFDGraph {
    /// Directory of RFDs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short, help = "")]
    pub link_extension: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,
}

/// Reserve RFD
#[derive(Parser, Debug)]
#[command(name = "reserve")]
pub(crate) struct ReserveRFD {
    /// RFD Number
    #[arg(long, short)]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    /// title of RFD
    #[arg(long, short)]
    pub title: String,

    /// Extension that should be used
    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
    )]
    pub extension: Option<MarkupFormat>,
}
