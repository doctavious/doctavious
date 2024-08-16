use std::path::PathBuf;
use std::str::FromStr;

use changelog::changelog::ChangelogOutputType;
use clap::{Parser, ValueEnum};
use doctavious_cli::changelog::cmd::release::{release, ChangelogReleaseOptions};
use doctavious_cli::changelog::settings::{ChangelogCommitSort, ChangelogRange};
use doctavious_cli::errors::CliResult;
use glob::Pattern;
use markup::MarkupFormat;
use regex::Regex;
use scm::drivers::git::TagSort;
use strum::VariantNames;

use crate::clap_enum_variants;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum StrippableChangelogSection {
    Header,
    Footer,
    All,
}

// TODO: handling single vs individual changelogs
// 1. could base if off output where if directory write individual otherwise write single.
// if nothing is provided it defaults to single with file name being changelog.md
// what I dont like about this I dont think we can determine if extension flag is required or not
// 2. separate fields of output and output_dir
// 3. --single <PATH> vs --individual <PATH>
// 4. --individual / --multiple flag. keep output as is without the clap default. include output_format if multiple

// TODO: handling output type for dir? if we are writing to a directory how do we know extension

#[derive(Parser, Debug)]
#[command()]
pub(crate) struct ReleaseCommand {
    #[arg(long, short)]
    pub cwd: Option<PathBuf>,

    /// Sets the path to include related commits [env: DOCTAVIOUS_CHANGELOG_INCLUDE_PATH=]
    #[arg(long = "include_path", value_name = "PATH")]
    pub include_paths: Option<Vec<Pattern>>,

    /// Sets the path to exclude related commits [env: DOCTAVIOUS_CHANGELOG_EXCLUDE_PATH=]
    #[arg(long = "exclude_path", value_name = "PATH")]
    pub exclude_paths: Option<Vec<Pattern>>,

    /// Sets the git repository [env: DOCTAVIOUS_CHANGELOG_REPOSITORY=]
    /// To generate a changelog for multiple git repositories:
    #[arg(long = "repository", short)]
    pub repositories: Option<Vec<PathBuf>>,

    // To calculate and set the next semantic version (i.e. bump the version) for the unreleased changes:
    /// Bumps the version for unreleased changes
    #[arg(long, action)]
    pub bump: bool,

    #[arg(long, action)]
    pub individual: bool,

    // env = "DOCTAVIOUS_CHANGELOG_OUTPUT",
    #[arg(long, short, value_name = "PATH")]
    pub output: Option<PathBuf>,

    // env = "DOCTAVIOUS_CHANGELOG_OUTPUT_TYPE",
    #[arg(
        long,
        value_name = "TYPE",
        default_value_t = ChangelogOutputType::default(),
        value_parser = clap_enum_variants!(ChangelogOutputType)
    )]
    pub output_type: ChangelogOutputType,

    // DOCTAVIOUS_CHANGELOG_FORMAT
    #[arg(
        long,
        value_name = "FORMAT",
        value_parser = clap_enum_variants!(MarkupFormat),
        requires = "individual"
    )]
    pub format: Option<MarkupFormat>,

    /// Sets the regex for matching git tags [env: DOCTAVIOUS_CHANGELOG_TAG_PATTERN=]
    #[arg(long, value_name = "PATTERN")]
    pub tag_pattern: Option<Regex>,

    /// Sets custom commit messages to include in the changelog [env: DOCTAVIOUS_CHANGELOG_SKIP_COMMIT=]
    #[arg(long = "skip_commit", value_name = "COMMIT")]
    pub skip_commits: Option<Vec<String>>,

    // TODO: could use -R and --range instead of index
    /// Sets the commit range to process [possible values: current, latest, unreleased, or
    /// in the format of <START>..<END>]
    #[arg(index = 1)]
    pub range: Option<ChangelogRange>,

    /// Determines method of sorting tags
    #[arg(
        long,
        default_value_t = TagSort::default(),
        value_parser = clap_enum_variants!(TagSort)
    )]
    pub tag_sort: TagSort,

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
    let output_type = if command.individual {
        ChangelogOutputType::Individual
    } else {
        ChangelogOutputType::Single
    };

    release(ChangelogReleaseOptions {
        cwd: &path,
        repositories: command.repositories,
        output: command.output,
        output_type,
        prepend: command.prepend,
        range: command.range,
        include_paths: command.include_paths,
        exclude_paths: command.exclude_paths,
        tag_sort: Some(command.tag_sort),
        sort: command.sort,
        tag_pattern: command.tag_pattern,
        tag: command.tag,
        skip_commits: command.skip_commits,
    })?;

    Ok(None)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use changelog::changelog::ChangelogOutputType;
    use changelog::settings::ChangelogSettings;
    use doctavious_cli::changelog::cmd::release::{release_with_settings, ChangelogReleaseOptions};
    use scm::drivers::git::TagSort;

    #[test]
    fn validate_release() {
        let cwd = PathBuf::from("../../");

        println!("{}", cwd.canonicalize().unwrap().to_string_lossy());

        let cmd = ChangelogReleaseOptions {
            cwd: cwd.as_path(),
            repositories: None,
            output: None,
            output_type: ChangelogOutputType::Single,
            prepend: None,
            range: None,
            include_paths: None,
            exclude_paths: None,
            tag_sort: Some(TagSort::default()),
            sort: Default::default(),
            tag_pattern: None,
            tag: None,
            skip_commits: None,
        };

        let settings = ChangelogSettings {
            ..Default::default()
        };

        release_with_settings(cmd, settings).unwrap();
    }
}
