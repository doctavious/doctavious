use std::io::Write;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use scm::ScmCommitRange;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, VariantNames};
use thiserror::Error;

use crate::changelog::commit::Commit;
use crate::changelog::release::Release;
use crate::settings::ChangelogSettings;

pub mod cmd;
pub mod commit;
mod release;
mod tag;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangelogErrors {
    /// Error that may occur while generating changelog.
    #[error("Changelog error: `{0}`")]
    ChangelogError(String),
}

// could we put these all behind a `--range` flag?
#[derive(Clone, Debug, Display, EnumIter, VariantNames, PartialEq)]
pub enum ChangelogRange {
    Current,
    Latest,
    Unreleased,
    Range(String),
}

lazy_static! {
    static ref CHANGELOG_RANGE_REGEX: Regex = Regex::new(r"^.+\.\..+$").unwrap();
}

impl FromStr for ChangelogRange {
    type Err = String;
    // const re: Regex = regex!("42");

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

#[derive(Clone, Copy, Debug, Default, Display, EnumIter, EnumString, VariantNames, PartialEq)]
pub enum ChangelogCommitSort {
    Newest,
    #[default]
    Oldest,
}

impl ChangelogCommitSort {
    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
}

// Not sure about the name but essentially controls if changelog should write details to a single
// file or if they should be separated.
// Might be able to solve this just with specifying range/tags/commits and passing in a specific output file
#[non_exhaustive]
#[remain::sorted]
#[derive(Default)]
pub enum ChangelogKind {
    Multiple,
    #[default]
    Single,
}

#[remain::sorted]
enum ChangeLogFormat {
    /// changelog similar to cocogitto's format.
    Cocogitto,

    /// changelog that contains links to the commits.
    Detailed,

    /// changelog in the GitHub's format.
    GitHub,

    /// combination of the previous two formats.
    GitHubKeepAChangelog,

    /// changelog in Keep a Changelog format.
    KeepAChangelog,

    Minimal,

    /// changelog with commits are grouped by their scopes.
    Scoped,

    /// changelog with commits grouped by their scopes and sorted by group.
    ScopedSorted,

    /// changelog for unconventional commits.
    Unconventional,
}

#[derive(Debug)]
pub struct Changelog {
    releases: Vec<Release>,
    // settings: &'a ChangeLogSettings
    // body_template:   Template,
    // footer_template: Option<Template>,
    // config:          &'a Config,
}

impl Changelog {
    // TODO: new

    fn new(releases: Vec<Release>, settings: ChangelogSettings) -> Self {
        Self { releases }
    }

    // TODO: process_commits
    fn process_commits(&mut self) {
        self.releases.iter_mut().for_each(|release| {
            release.commits = release
                .commits
                .iter()
                .cloned()
                .filter_map(|commit| {
                    match commit.process() {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                    None
                })
                .flat_map(|commit| {
                    // if self.config.git.split_commits.unwrap_or(false) {
                    //     commit
                    //         .message
                    //         .lines()
                    //         .filter_map(|line| {
                    //             let mut c = commit.clone();
                    //             c.message = line.to_string();
                    //             if !c.message.is_empty() {
                    //                 Self::process_commit(c, &self.config.git)
                    //             } else {
                    //                 None
                    //             }
                    //         })
                    //         .collect()
                    // } else {
                    //     vec![commit]
                    // }
                    vec![commit]
                })
                .collect::<Vec<Commit>>();
        });
    }

    // TODO: process_releases

    // TODO: generate
    pub fn generate<W: Write>(&self, out: &mut W) {}

    // TODO: prepend

    pub fn bump_version(&self) {}
}
