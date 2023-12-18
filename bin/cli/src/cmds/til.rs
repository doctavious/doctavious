use clap::builder::PossibleValuesParser;
use clap::Parser;
use doctavious_cli::markup_format::MarkupFormat;

/// Gathers Today I Learned (TIL) management commands
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct TilCommand {
    #[command(subcommand)]
    pub til_command: TilSubCommand,
}

#[remain::sorted]
#[derive(Parser, Debug)]
pub(crate) enum TilSubCommand {
    Init(InitTil),
    List(ListTils),
    New(NewTil),
    Readme(BuildTilReadMe),
}

/// Init TIL
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct InitTil {
    /// Directory of TILs
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: path to readme template or template string. two fields? one and we determine if its a path?
    // what do others do? Terraform has `var` and `var-file`
    /// Extension that should be used
    #[arg(
        // value_enum,
        long,
        short,
        default_value = "MarkupFormat::default()",
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        // value_parser,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub extension: MarkupFormat,
}

/// New TIL
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct NewTil {
    /// TIL category. Represents the directory to place TIL entry under
    #[arg(short, long)]
    pub category: String,

    /// title of the TIL entry
    #[arg(long, short)]
    pub title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    /// Additional tags associated with the TIL entry
    #[arg(short = 'T', long)]
    pub tags: Option<Vec<String>>,

    /// File name that should be used.
    /// If extension is included will take precedence over extension argument and configuration file.
    #[arg(long, short)]
    pub file_name: Option<String>,

    /// Extension that should be used. This overrides value from configuration file.
    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        // value_parser,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub extension: Option<MarkupFormat>,

    // TODO: should this also be a setting in TilSettings?
    /// Whether to build_mod a README after a new TIL is added
    #[arg(short, long)]
    pub readme: bool,
}

/// List TILs
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ListTils {}

/// Build TIL ReadMe
#[derive(Parser, Debug)]
#[command()]
pub(crate) struct BuildTilReadMe {
    /// Directory where TILs are stored
    #[arg(long, short)]
    pub directory: Option<String>,

    // TODO: optional path to template.
    /// Extension that should be used
    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        // value_parser,
        value_parser = PossibleValuesParser::new(MarkupFormat::variants()),
    )]
    pub extension: Option<MarkupFormat>,
}
