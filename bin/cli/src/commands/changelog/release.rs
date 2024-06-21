use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

use clap::builder::ValueParser;
use clap::{Parser, ValueEnum};
use doctavious_cli::changelog::cmd::release::{release, ChangelogReleaseOptions};
use doctavious_cli::changelog::{ChangelogCommitSort, ChangelogRange};
use doctavious_cli::{CliResult, DoctaviousCliError};
use glob::Pattern;
use regex::Regex;
use strum::VariantNames;

use crate::clap_enum_variants;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StrippableChangelogSection {
    Header,
    Footer,
    All,
}

#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ReleaseCommand {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Sets the path to include related commits [env: DOCTAVIOUS_CHANGELOG_INCLUDE_PATH=]
    #[arg(long = "include_path", value_name = "PATH")]
    include_paths: Option<Vec<Pattern>>,

    /// Sets the path to exclude related commits [env: DOCTAVIOUS_CHANGELOG_EXCLUDE_PATH=]
    #[arg(long = "exclude_path", value_name = "PATH")]
    exclude_paths: Option<Vec<Pattern>>,

    /// Sets the git repository [env: DOCTAVIOUS_CHANGELOG_REPOSITORY=]
    /// To generate a changelog for multiple git repositories:
    #[arg(long = "repository", short)]
    repositories: Option<Vec<PathBuf>>,

    // To calculate and set the next semantic version (i.e. bump the version) for the unreleased changes:
    /// Bumps the version for unreleased changes
    #[arg(long, action)]
    pub bump: bool,

    #[arg(long, short)]
    output: Option<String>,

    /// Sets the regex for matching git tags [env: DOCTAVIOUS_CHANGELOG_TAG_PATTERN=]
    #[arg(long, value_name = "PATTERN")]
    tag_pattern: Option<Regex>,

    /// Sets custom commit messages to include in the changelog [env: DOCTAVIOUS_CHANGELOG_SKIP_COMMIT=]
    #[arg(long, value_name = "COMMIT")]
    skip_commit: Vec<String>,

    // TODO: could use -R and --range instead of index
    /// Sets the commit range to process.
    #[arg(index = 1)]
    pub range: Option<ChangelogRange>,

    /// Sorts the tags topologically.
    #[arg(long)]
    pub topo_order: bool,

    /// Prepends entries to the changelog file [env: DOCTAVIOUS_CHANGELOG_PREPEND=]
    #[arg(long, short)]
    pub prepend: Option<PathBuf>,

    /// Sets the tag for the latest version.
    #[arg(
        short,
        long,
        env = "DOCTAVIOUS_CHANGELOG_TAG",
        allow_hyphen_values = true
    )]
    pub tag: Option<String>,

    /// Sets sorting of the commits inside sections.
    #[arg(
        long,
        default_value_t = ChangelogCommitSort::default(),
        value_parser = clap_enum_variants!(ChangelogCommitSort)
    )]
    pub sort: ChangelogCommitSort,

    /// Sets the template for the changelog body.
    #[arg(
        short,
        long,
        env = "DOCTAVIOUS_CHANGELOG_TEMPLATE",
        value_name = "TEMPLATE",
        allow_hyphen_values = true
    )]
    pub body: Option<String>,

    /// Strips the given parts from the changelog.
    #[arg(short, long, value_name = "PART", value_enum)]
    pub strip: Option<StrippableChangelogSection>,
    // -p, --prepend <PATH>             Prepends entries to the given changelog file [env: GIT_CLIFF_PREPEND=]
    // -o, --output [<PATH>]            Writes output to the given file [env: GIT_CLIFF_OUTPUT=]
    // -t, --tag <TAG>                  Sets the tag for the latest version [env: GIT_CLIFF_TAG=]
    // -b, --body <TEMPLATE>            Sets the template for the changelog body [env: GIT_CLIFF_TEMPLATE=]
    // -s, --strip <PART>               Strips the given parts from the changelog [possible values: header, footer, all]
    // --sort <SORT>                Sets sorting of the commits inside sections [default: oldest] [possible values: oldest, newest]

    // "'-u' or '-l' is not specified",

    // "'-o' and '-p' can only be used together if they point to different files",
}

pub(crate) fn execute(command: ReleaseCommand) -> CliResult<Option<String>> {
    let path = command.cwd.unwrap_or(std::env::current_dir()?);

    release(ChangelogReleaseOptions {
        cwd: &path,
        repositories: command.repositories,
        prepend: command.prepend,
        range: command.range,
        include_paths: command.include_paths,
        exclude_paths: command.exclude_paths,
        topo_order: command.topo_order,
        sort: command.sort,
        tag_pattern: command.tag_pattern,
        tag: command.tag,
    })?;

    Ok(None)
}

// pub fn validator_regex(r: &'static str) -> ValueParser {
//     ValueParser::from(move |s: &str| -> std::result::Result<String, DoctaviousCliError> {
//         // let reg = regex::Regex::new(r).unwrap();
//         // match reg.is_match(s) {
//         //     true => Ok(s.to_owned()),
//         //     false => Err(Error::from(format!("not matches {}", r))),
//         // }
//         println!("{}", s);
//         Ok(s.to_owned())
//     })
// }

// fn parse_changelog_range(arg: &str) -> CliResult<ChangelogRange> {
//     Ok(match arg {
//         "current" => ChangelogRange::Current,
//         "latest" => ChangelogRange::Latest,
//         "unreleased" => ChangelogRange::Unreleased,
//         _ => ChangelogRange::Range(arg.to_string())
//     })
// }

#[cfg(test)]
mod tests {}
