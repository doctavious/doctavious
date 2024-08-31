use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize, Serializer};
use serde::ser::SerializeStruct;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SomeverError {
    #[error("Version text is empty")]
    Empty,

    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    // TODO: maybe map to better error
    // #[error(transparent)]
    // ParseInt(#[from] ParseIntError),
    #[error("Could not parse {0} into digit")]
    ParseInt(String),

    #[error(transparent)]
    SemverError(#[from] semver::Error),
}

pub type SomeverResult<T> = Result<T, SomeverError>;

pub trait Bumpable {}

#[remain::sorted]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VersioningScheme {
    // do we need to know more information? Such as format/separator so each part can be
    // appropriated compared
    Calver,
    #[default]
    Semver,
}

// do we need a separator?
// whats the best way to handle parsing micro from modifier?
// how to handle sorting modifier?
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Calver {
    pub major: u16,
    pub minor: u8,
    pub micro: Option<u16>,
    pub modifier: Option<String>,
    pub separator: char,
}

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"(?<major>\d+)[.-](?<minor>\d+)([.-](?<micro>\d+))?(?<modifier>.+)?").unwrap();
}

impl Calver {
    pub fn parse(text: &str) -> SomeverResult<Self> {
        Calver::from_str(text)
    }
}

impl FromStr for Calver {
    type Err = SomeverError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        // not the most performant way of doing this but good enough for now
        let caps = RE
            .captures(text)
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
        // .map_err(|e|Err(SomeverError::ParseInt(text.to_string())))?;

        let major_match = caps
            .name("major")
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
        // .map_err(|e|Err(SomeverError::ParseInt(text.to_string())))?;

        let major = major_match
            .as_str()
            .parse::<u16>()
            .map_err(|e| SomeverError::ParseInt(text.to_string()))?;

        let separator = text
            .chars()
            .nth(major_match.len())
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
        // .map_err(|e|SomeverError::ParseInt(text.to_string()))?;

        let minor = caps
            .name("minor")
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?
            .as_str()
            .parse::<u8>()
            .map_err(|e| SomeverError::ParseInt(text.to_string()))?;

        let micro = if let Some(micro) = caps.name("micro") {
            Some(
                micro
                    .as_str()
                    .parse::<u16>()
                    .map_err(|e| SomeverError::ParseInt(text.to_string()))?,
            )
        } else {
            None
        };

        let modifier = caps.name("modifier").map(|m| m.as_str().to_string());

        Ok(Self {
            major,
            minor,
            micro,
            modifier,
            separator,
        })
    }
}

impl Display for Calver {
    // TODO: might be better to just store raw
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{:?}{:?}",
            self.major, self.minor, self.micro, self.modifier
        )
    }
}

// Want to support the following
// YYYY - Full year - 2006, 2016, 2106
// YY - Short year - 6, 16, 106
// 0Y - Zero-padded year - 06, 16, 106
// MM - Short month - 1, 2 ... 11, 12
// 0M - Zero-padded month - 01, 02 ... 11, 12
// WW - Short week (since start of year) - 1, 2, 33, 52
// 0W - Zero-padded week - 01, 02, 33, 52
// DD - Short day - 1, 2 ... 30, 31
// 0D - Zero-padded day - 01, 02 ... 30, 31

// Examples
// https://stripe.com/blog/api-versioning - YYYY-MM-DD
// https://unity3d.com/unity/whats-new/ - YYYY.MINOR.MICRO
// https://www.cockroachlabs.com/blog/calendar-versioning/ - YY.RELEASE_NUMBER.PATCH
// two-digit year for the major component and release number within the year for the minor one
// For patch releases, we'll use the third, "micro" number in the versioning scheme to indicate the
// patch number, omitting the micro number on the first release number for external representations of the version number.

// (<MAJOR>\d).(<MINOR>\d)(?<MICRO>.\d)(?<MODIFIER>.+)

#[derive(Debug)]
pub enum Somever {
    Calver(Calver),
    Semver(semver::Version),
}

// TODO: impl Deserialize
impl Serialize for Somever {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {

        let mut somever = serializer.serialize_struct("Somever", 2)?;
        match self {
            Somever::Calver(c) => {
                somever.serialize_field("type", "calver")?;
                somever.serialize_field("value", c.to_string().as_str())?;
            }
            Somever::Semver(s) => {
                somever.serialize_field("type", "semver")?;
                somever.serialize_field("value", s.to_string().as_str())?;
            }
        }

        somever.end()
    }
}

impl Somever {
    pub fn new(scheme: &VersioningScheme, value: &str) -> SomeverResult<Self> {
        Ok(match scheme {
            VersioningScheme::Calver => Somever::Calver(Calver::parse(value)?),
            VersioningScheme::Semver => Somever::Semver(semver::Version::parse(value)?),
        })
    }

    pub fn major(&self) -> u64 {
        match self {
            Somever::Calver(c) => c.major as u64,
            Somever::Semver(s) => s.major,
        }
    }

    pub fn minor(&self) -> u64 {
        match self {
            Somever::Calver(c) => c.minor as u64,
            Somever::Semver(s) => s.minor,
        }
    }

    pub fn patch(&self) -> Option<u64> {
        match self {
            Somever::Calver(c) => c.micro.map(|m| m as u64),
            Somever::Semver(s) => Some(s.patch),
        }
    }
}

impl Display for Somever {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Somever::Calver(c) => write!(f, "{}", c.to_string()),
            Somever::Semver(s) => write!(f, "{}", s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Calver;

    #[test]
    fn should_parse_and_sort() {
        let mut versions = vec![];
        for f in [
            "2024.01.28",
            "2024.01.28-final",
            "2024.01.28-suffix",
            "24.1",
            "24.1.28",
            "24.1.28-final",
            "06.01.28",
            "06.52.01",
            "2024.1-suffix",
            "2024.1.suffix",
            "2024-06-28",
        ] {
            versions.push(Calver::parse(f).unwrap());
        }

        versions.sort();

        for v in versions {
            println!("{:?}", v.to_string());
        }
    }
}
