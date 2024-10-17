mod calendar;
mod semantic;

use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

pub use calendar::Calver;
// use semver::{BuildMetadata, Prerelease};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use thiserror::Error;

use crate::semantic::Semver;

#[remain::sorted]
#[derive(Debug, Error, PartialEq)]
pub enum SomeverError {
    #[error("Version text is empty")]
    Empty,

    #[error("invalid")]
    Invalid,

    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    // TODO: maybe map to better error
    // #[error(transparent)]
    // ParseInt(#[from] ParseIntError),
    #[error("Could not parse {0} into digit")]
    ParseInt(String),
    // #[error(transparent)]
    // SemverError(#[from] semver::Error),
}

pub type SomeverResult<T> = Result<T, SomeverError>;

pub trait Bumpable {}

#[remain::sorted]
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VersioningScheme {
    // do we need to know more information? Such as format/separator so each part can be
    // appropriated compared
    Calver,
    #[default]
    Semver,
}

#[derive(Debug)]
pub enum Somever {
    Calver(Calver),
    // Semver(semver::Version),
    Semver(Semver),
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

impl PartialEq for Somever {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for Somever {}

impl PartialOrd for Somever {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            self.major()
                .cmp(&other.major())
                .then(self.minor().cmp(&other.minor()))
                .then(self.patch().cmp(&other.patch()))
                .then(self.modifier().cmp(&other.modifier())),
        )
    }
}

impl Somever {
    pub fn new(scheme: VersioningScheme, value: &str) -> SomeverResult<Self> {
        Ok(match scheme {
            // TODO: need to pass in format
            VersioningScheme::Calver => Somever::Calver(Calver::parse(value, "")?),
            // VersioningScheme::Semver => Somever::Semver(semver::Version::parse(value)?),
            VersioningScheme::Semver => Somever::Semver(Semver::parse(value.to_string())?),
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
            Somever::Calver(c) => c.patch.map(|m| m as u64),
            Somever::Semver(s) => Some(s.patch),
        }
    }

    // TODO: would like to be able to sort this via something like git versionsort.suffix
    //      I think we would need to pass in a Vec of suffixes and then use the index to sort
    pub fn modifier(&self) -> Option<&str> {
        match self {
            Somever::Calver(c) => c.modifier.as_ref().map(|s| s.as_str()),
            // Somever::Semver(s) => {
            //     if s.pre != Prerelease::EMPTY {
            //         Some(s.pre.as_str())
            //     } else if s.build != BuildMetadata::EMPTY {
            //         Some(s.build.as_str())
            //     } else {
            //         None
            //     }
            // }
            Somever::Semver(s) => {
                if let Some(prerelease) = &s.prerelease {
                    Some(prerelease.as_str())
                } else if let Some(build) = &s.build {
                    Some(build.as_str())
                } else {
                    None
                }
            }
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
    use crate::semantic::Semver;
    use crate::{Calver, Somever, VersioningScheme};

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
            versions.push(Calver::parse(f, "").unwrap());
        }

        versions.sort();

        assert_eq!(
            vec![
                "6.1.28",
                "6.52.1",
                "24.1",
                "24.1.28",
                "24.1.28-final",
                "2024.1-suffix",
                "2024.1.suffix",
                "2024.1.28",
                "2024.1.28-final",
                "2024.1.28-suffix",
                "2024-6-28",
            ],
            versions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
        );
    }

    // TODO: confirm deserialization
    #[test]
    fn serde() {
        assert_eq!(
            "{\"type\":\"calver\",\"value\":\"2024.9.2\"}".to_string(),
            serde_json::to_string(&Somever::Calver(Calver::parse("2024.9.2", "").unwrap()))
                .unwrap()
        );
        assert_eq!(
            "{\"type\":\"calver\",\"value\":\"2024.9.2\"}".to_string(),
            serde_json::to_string(&Somever::new(VersioningScheme::Calver, "2024.9.2").unwrap())
                .unwrap()
        );

        // let calver: Calver = serde_json::from_str("{\"type\":\"calver\",\"value\":\"2024.9.2\"}").unwrap();
        // assert_eq!(calver.major, 2024);
        // assert_eq!(calver.minor, 9);
        // assert_eq!(calver.micro.unwrap_or_default(), 2);
        // assert_eq!(calver.modifier.unwrap_or_default(), "-suffix");

        assert_eq!(
            "{\"type\":\"semver\",\"value\":\"1.5.2\"}".to_string(),
            serde_json::to_string(&Somever::Semver(
                Semver::parse("1.5.2".to_string()).unwrap()
            ))
            .unwrap()
        );
        assert_eq!(
            "{\"type\":\"semver\",\"value\":\"1.5.2\"}".to_string(),
            serde_json::to_string(&Somever::new(VersioningScheme::Semver, "1.5.2").unwrap())
                .unwrap()
        );

        assert_eq!(
            "{\"type\":\"semver\",\"value\":\"v1.5.2\"}".to_string(),
            serde_json::to_string(&Somever::new(VersioningScheme::Semver, "v1.5.2").unwrap())
                .unwrap()
        );
    }
}
