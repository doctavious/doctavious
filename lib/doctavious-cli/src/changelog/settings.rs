use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use strum::{Display, EnumIter, EnumString, VariantNames};

lazy_static! {
    static ref CHANGELOG_RANGE_REGEX: Regex = Regex::new(r"^.+\.\..+$").unwrap();
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
