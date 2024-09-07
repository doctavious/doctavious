use std::path::PathBuf;
use std::str::FromStr;

use changelog::changelog::ChangelogOutputType;
use clap::{Parser, ValueEnum};
use doctavious_cli::changelog::cmd::release::{release, BumpOption, ChangelogReleaseOptions};
use doctavious_cli::changelog::settings::{ChangelogCommitSort, ChangelogRange};
use doctavious_cli::errors::CliResult;
use doctavious_cli::settings::{load_settings, Settings};
use glob::Pattern;
use markup::MarkupFormat;
use regex::Regex;
use scm::drivers::git::TagSort;
use serde::{Deserialize, Serialize};
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
    #[arg(
        long,
        short,
        env = "DOCTAVIOUS_CHANGELOG_WORKDIR",
        value_parser = ReleaseCommand::parse_dir
    )]
    pub cwd: Option<PathBuf>,

    /// Path of the Doctavious config you use. If not present will default to configuration present
    /// in the working directory
    #[arg(
	    long,
	    env = "DOCTAVIOUS_CONFIG",
	    value_name = "PATH",
	    // default_value = DEFAULT_CONFIG,
	    value_parser = ReleaseCommand::parse_dir
    )]
    pub config: Option<PathBuf>,

    /// Path
    #[arg(
        long = "include_path",
        value_name = "PATH",
        env = "DOCTAVIOUS_CHANGELOG_INCLUDE_PATH"
    )]
    pub include_paths: Option<Vec<Pattern>>,

    /// Sets the path to exclude related commits
    #[arg(
        long = "exclude_path",
        value_name = "PATH",
        env = "DOCTAVIOUS_CHANGELOG_EXCLUDE_PATH"
    )]
    pub exclude_paths: Option<Vec<Pattern>>,

    /// The Git repositories to use. Useful to generate a changelog for multiple git repositories
    #[arg(
        long = "repository",
        short,
        value_name = "PATH",
        env = "DOCTAVIOUS_CHANGELOG_REPOSITORIES",
        value_parser = ReleaseCommand::parse_dir
    )]
    pub repositories: Option<Vec<PathBuf>>,

    #[arg(long, action)]
    pub individual: bool,

    #[arg(
        long,
        short,
        value_name = "PATH",
        env = "DOCTAVIOUS_CHANGELOG_OUTPUT",
        value_parser = ReleaseCommand::parse_dir
    )]
    pub output: Option<PathBuf>,

    #[arg(
        long,
        value_name = "TYPE",
        default_value_t = ChangelogOutputType::default(),
        value_parser = clap_enum_variants!(ChangelogOutputType),
        env = "DOCTAVIOUS_CHANGELOG_OUTPUT_TYPE"
    )]
    pub output_type: ChangelogOutputType,

    #[arg(
        long,
        value_name = "FORMAT",
        value_parser = clap_enum_variants!(MarkupFormat),
        requires = "individual",
        env = "DOCTAVIOUS_CHANGELOG_FORMAT"
    )]
    pub format: Option<MarkupFormat>,

    /// Patterns for tags that should be included in the changelog
    #[arg(
        long = "tag_pattern",
        value_name = "PATTERN",
        env = "DOCTAVIOUS_CHANGELOG_TAG_PATTERNS"
    )]
    pub tag_patterns: Option<Vec<String>>,

    /// Patterns for tags that should be skpped but the associated commits be part
    /// of the next valid tag
    #[arg(
        long = "skip_tag_pattern",
        value_name = "PATTERN",
        env = "DOCTAVIOUS_CHANGELOG_SKIP_PATTERNS"
    )]
    pub skip_tag_patterns: Option<Vec<String>>,

    /// Patterns for tags that should be ignored.
    /// Associated commits will not be present in the changelog
    #[arg(
        long = "ignore_tag_pattern",
        value_name = "PATTERN",
        env = "DOCTAVIOUS_CHANGELOG_IGNORE_TAG_PATTERNS"
    )]
    pub ignore_tag_patterns: Option<Vec<String>>,

    /// Commits IDs that will be ignored and not present in the changelog
    /// Will be merged with `.commitsignore` if present
    #[arg(
        long = "ignore_commit",
        value_name = "COMMIT",
        env = "DOCTAVIOUS_CHANGELOG_IGNORE_COMMITS"
    )]
    pub ignore_commits: Option<Vec<String>>,

    /// Determines method of sorting tags
    #[arg(
        long,
        default_value_t = TagSort::default(),
        value_parser = clap_enum_variants!(TagSort)
    )]
    pub tag_sort: TagSort,

    /// Prepends entries to the changelog file
    #[arg(
        long,
        short,
        conflicts_with = "individual",
        env = "DOCTAVIOUS_CHANGELOG_PREPEND",
        value_parser = ReleaseCommand::parse_dir
    )]
    pub prepend: Option<PathBuf>,

    /// Sets the tag for the latest version.
    #[arg(
        short,
        long,
        env = "DOCTAVIOUS_CHANGELOG_TAG",
        allow_hyphen_values = true
    )]
    pub tag: Option<String>,

    /// Determines how commits should be sorted within tags
    #[arg(
        long,
        default_value_t = ChangelogCommitSort::default(),
        value_parser = clap_enum_variants!(ChangelogCommitSort)
    )]
    pub commit_sort: ChangelogCommitSort,

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

    /// Bumps the version for unreleased changes
    #[arg(
        long,
        value_name = "BUMP",
        default_missing_value = "auto",
        value_parser = clap_enum_variants!(BumpOption),
        conflicts_with = "tag",
    )]
    pub bump: Option<BumpOption>,

    // /// Prints changelog context as JSON.
    // #[arg(
    //     short = 'x',
    //     long,
    // )]
    // pub context: bool,
    //
    // /// Generates changelog from a JSON context.
    // #[arg(
    //     long,
    //     value_name = "PATH",
    //     // value_parser = ReleaseCommand::parse_dir,
    // 	env = "DOCTAVIOUS_CHANGELOG_CONTEXT",
    // DOCTAVIOUS_CHANGELOG_PREPEND
    // )]
    // pub from_context: Option<PathBuf>,

    // TODO: could use -R and --range instead of index
    /// Sets the commit range to process [possible values: current, latest, unreleased, or
    /// in the format of <START>..<END>]
    #[arg(index = 1)]
    pub range: Option<ChangelogRange>,
}

impl ReleaseCommand {
    /// Custom string parser for directories.
    ///
    /// Expands the tilde (`~`) character in the beginning of the
    /// input string into contents of the path returned by [`home_dir`].
    fn parse_dir(dir: &str) -> Result<PathBuf, String> {
        Ok(PathBuf::from(shellexpand::tilde(dir).to_string()))
    }
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
        config_path: command.config.as_ref().map(|c| c.as_path()),
        repositories: command.repositories,
        output: command.output,
        output_type,
        prepend: command.prepend,
        range: command.range,
        include_paths: command.include_paths,
        exclude_paths: command.exclude_paths,
        tag_sort: Some(command.tag_sort),
        commit_sort: command.commit_sort,
        tag_patterns: command.tag_patterns,
        skip_tag_patterns: command.skip_tag_patterns,
        ignore_tag_patterns: command.ignore_tag_patterns,
        tag: command.tag,
        ignore_commits: command.ignore_commits,
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
            config_path: None,
            repositories: None,
            output: None,
            output_type: ChangelogOutputType::Single,
            prepend: None,
            range: None,
            include_paths: None,
            exclude_paths: None,
            tag_sort: Some(TagSort::default()),
            commit_sort: Default::default(),
            tag_patterns: None,
            skip_tag_patterns: None,
            ignore_tag_patterns: None,
            tag: None,
            ignore_commits: None,
        };

        let settings = ChangelogSettings {
            ..Default::default()
        };

        release_with_settings(cmd, settings).unwrap();
    }
}
