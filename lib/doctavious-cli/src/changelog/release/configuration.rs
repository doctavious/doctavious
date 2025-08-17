use std::path::{Path, PathBuf};
use std::str::FromStr;

use changelog::changelog::ChangelogOutputType;
use changelog::settings::ChangelogCommitSort;
use glob::Pattern;
use lazy_static::lazy_static;
use regex::Regex;
use scm::drivers::git::TagSort;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, VariantNames};

lazy_static! {
    static ref CHANGELOG_RANGE_REGEX: Regex = Regex::new(r"^.+\.\..+$").unwrap();
}

// TODO: replace cwd with repositories
// TODO: ignore commits will live in .doctavious/changelog/.commitsignore
// TODO: do we want to allow users to specify the configuration file?
// pass in config path instead of cwd
#[derive(Clone, Debug)]
pub struct ChangelogReleaseOptions<'a> {
    pub cwd: &'a Path, // TODO: make optional and implement default
    pub config_path: Option<&'a Path>,
    // this doesn't below in the configuration file
    pub repositories: Option<Vec<PathBuf>>,
    pub output: Option<PathBuf>,
    pub output_type: ChangelogOutputType,
    pub prepend: Option<PathBuf>,
    // does range? I feel like you could make a case either way
    pub range: Option<ChangelogRange>,
    pub include_paths: Option<Vec<Pattern>>,
    pub exclude_paths: Option<Vec<Pattern>>,
    pub commit_sort: ChangelogCommitSort,

    pub ignore_commits: Option<Vec<String>>,

    /// Changes to these will be retained
    pub tag_patterns: Option<Vec<String>>,

    /// Changes belonging to these releases will be included in the next non-skipped release
    pub skip_tag_patterns: Option<Vec<String>>,

    /// Changes belonging to these releases will not appear in the changelog
    pub ignore_tag_patterns: Option<Vec<String>>,

    // this doesnt below in the configuration file
    pub tag: Option<String>,

    // TODO: this needs to fit into Somever sorting
    pub tag_sort: Option<TagSort>,

    pub strip: Option<StrippableChangelogSection>,
}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Deserialize, EnumString, Serialize, VariantNames)]
pub enum StrippableChangelogSection {
    All,
    Footer,
    Header,
}

#[remain::sorted]
#[derive(
    Clone, Debug, Display, Default, Deserialize, EnumString, Eq, PartialEq, Serialize, VariantNames,
)]
pub enum BumpOption {
    #[default]
    Auto,
    Specific(BumpType),
}

/// Version bump type.
#[derive(
    Clone, Copy, Debug, Default, Deserialize, EnumString, Eq, PartialEq, Serialize, VariantNames,
)]
pub enum BumpType {
    /// Bump major version.
    Major,
    /// Bump minor version.
    Minor,
    /// Bump patch version.
    #[default]
    Patch,
}

#[derive(Clone, Debug, Display, EnumIter, VariantNames, PartialEq)]
pub enum ChangelogRange {
    Current,
    Latest,
    Unreleased,
    Range(String),
}

impl FromStr for ChangelogRange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "current" => Ok(ChangelogRange::Current),
            "latest" => Ok(ChangelogRange::Latest),
            "unreleased" => Ok(ChangelogRange::Unreleased),
            _ => {
                if CHANGELOG_RANGE_REGEX.is_match(s) {
                    Ok(ChangelogRange::Range(s.to_string()))
                } else {
                    Err(
                        "Invalid changelog range. Value should be current, latest, unreleased, or \
                        in the format of <START>..<END>"
                            .to_string(),
                    )
                }
            }
        }
    }
}
