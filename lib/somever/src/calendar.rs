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

use std::fmt::{Display, Formatter};
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{SomeverError, SomeverResult};

lazy_static! {
    static ref RE: Regex =
        Regex::new(r"(?<major>\d+)[.-](?<minor>\d+)([.-](?<micro>\d+))?(?<modifier>.+)?").unwrap();
}

// TODO: Confirm sorting/ordering especially with modifier
// TODO: Handling specific formats? Ex: 0M - Zero-padded month. As of right now we provide a lossy
//      conversion where if user provides "2024-06-08" we would output "2024-6-8"
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Calver {
    pub major: u16,
    pub minor: u8,
    pub micro: Option<u16>,
    pub modifier: Option<String>,
    pub separator: char,
    // pub format: String,
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

        let major_match = caps
            .name("major")
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?;

        let major = major_match
            .as_str()
            .parse::<u16>()
            .map_err(|e| SomeverError::ParseInt(text.to_string()))?;

        let separator = text
            .chars()
            .nth(major_match.len())
            .ok_or(SomeverError::InvalidFormat(text.to_string()))?;

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
        write!(f, "{}{}{}", self.major, self.separator, self.minor)?;

        if let Some(micro) = &self.micro {
            write!(f, "{}{}", self.separator, micro)?;
        }

        if let Some(modifier) = &self.modifier {
            write!(f, "{}", modifier)?;
        }

        Ok(())
    }
}

pub(crate) struct FormatConvention {
    pub(crate) representation: &'static str,
    pub(crate) format: &'static str,
}

impl FormatConvention {
    fn conventions() {
        let YYYY = FormatConvention {
            representation: "YYYY",
            format: "%04d",
        };

        let YY = FormatConvention {
            representation: "YY",
            format: "%d",
        };

        let zeroY = FormatConvention {
            representation: "0Y",
            format: "%02d",
        };

        let MM = FormatConvention {
            representation: "MM",
            format: "%d",
        };

        let M0 = FormatConvention {
            representation: "M0",
            format: "%02d",
        };

        let zeroM = FormatConvention {
            representation: "0M",
            format: "%02d",
        };

        let DD = FormatConvention {
            representation: "DD",
            format: "%d",
        };

        let D0 = FormatConvention {
            representation: "DD",
            format: "%d",
        };

        let zeroD = FormatConvention {
            representation: "0D",
            format: "%02d",
        };
    }
}
