use std::io::Write;
use std::path::Path;

use strum::{Display, EnumIter, EnumString, IntoEnumIterator, VariantNames};

use crate::changelog::release::Release;

pub mod cmd;
pub mod commit;
mod release;
mod tag;

// could we put these all behind a `--range` flag?
enum ChangelogRange {
    Current,
    Latest,
    Unreleased,
    Range(String),
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

    // TODO: process_commits

    // TODO: process_releases

    // TODO: generate
    pub fn generate<W: Write>(&self, out: &mut W) {}

    // TODO: prepend

    pub fn bump_version(&self) {}
}
