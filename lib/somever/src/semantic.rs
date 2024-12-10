use std::fmt::{Display, Formatter};
use std::str::FromStr;

use thiserror::Error;

use crate::{SomeverError, SomeverResult};

/// An lenient implementation of semantic version (semver).
/// Semver crate implements a fairly strict semver implementation and we need something that isn't
/// as strict. A `lenient-semver` crate does exist unfortunately it doesn't seem like it's being
/// actively maintained at the moment. Rather than vendoring that crate's implementation we decided
/// to leverage the `semver` crates implementation and updating it allow for wider range of use cases
/// using tests cases from the `lenient-semver` crate. The `lenient-semver` allows for some interesting
/// version strings that we aren't sure make 100% sense however and we may reconsider them at a
/// later date if we feel like we can remove support for edge cases that we aren't likely to see
/// in the wild.
///
/// The main things we want to support is
/// - allow a leaning v or V (e.g. "v1.2.3" parses as "1.2.3")
/// - allow leading zeros
/// - Minor and Patch are optional an default to 0 (e.g. "1" parses as "1.0.0")
/// - Pre-release identifier may be separated by '.' as well (e.g. "1.2.3.rc1" parses as "1.2.3-rc1").
/// - Some pre-release identifiers (e.g. ".Final", "-final") should be parsed as build identifier
///     (e.g. "1.2.3.Final" parses as "1.2.3+Final").
/// - Additional numeric identifiers are parsed as build identifier (e.g "1.2.3.4.5" parses as "1.2.3+4.5")
///
/// We were considering attempting to make the implementation as close to lossless as possible but
/// given the number use cases we are currently supporting we, at least for the time being, opted
/// not to strive for that. We do try to retain formatting for some scenarios, e.g. pre-release
/// identifier as a dot (.) we attempt to kep that intact but this is not applicable for all scenarios
#[derive(Debug, PartialEq)]
pub struct Semver {
    pub prefix: Option<String>,
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    // pub pre: Prerelease,
    pub prerelease: Option<String>,
    pub lenient_prerelease: bool,
    // pub build: BuildMetadata,
    pub build: Option<String>,
    pub build_prerelease_release_identifier: bool,
}

#[remain::sorted]
#[derive(Debug, Error, PartialEq)]
pub enum SemanticError {
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

    // #[error("...")]
    // EmptySegment(Position),
    //
    // #[error("...")]
    // Overflow(Position),
    #[error("...")]
    UnexpectedChar(String), // Position,String

                            // UnexpectedCharAfter Position,String
                            // UnexpectedChar Position,String
                            // UnexpectedEnd Position

                            // #[error(transparent)]
                            // SemverError(#[from] semver::Error),
}

impl FromStr for Semver {
    type Err = SomeverError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        // TODO: can I do something about text being consistently shadowed?
        let (prefix, text) = version_prefix(text.trim());
        let (major, text) = numeric_identifier(text)?;
        let (minor, text) = get_numeric_identifier(text)?;
        let (patch, text) = get_numeric_identifier(text)?;

        let mut prerelease = None;
        let mut lenient_prerelease = false;
        let mut build = None;
        let mut build_prerelease_release_identifier = false;

        let text = if let Some(text) = text.strip_prefix('.') {
            let (pr, b, text) = lenient_identifiers(text)?;
            if let Some(pr) = &pr {
                lenient_prerelease = true;
                if Semver::is_release_identifier(pr) {
                    build = Some(pr.to_string());
                    build_prerelease_release_identifier = true;
                } else {
                    prerelease = Some(pr.to_string());
                }
            }

            if let Some(b) = b {
                if let Some(val) = build {
                    build = Some(format!("{val}.{b}"))
                } else {
                    build = Some(b);
                }
            }

            text
        } else {
            text
        };

        let text = if let Some(text) = text.strip_prefix('-') {
            if prerelease.is_some() {
                // TODO: return error
            }

            let (ident, text) = prerelease_identifier(text)?;
            if ident.is_empty() {
                // TODO: return Err(Error::new(ErrorKind::EmptySegment(pos)));
                return Err(SomeverError::Invalid);
            }

            if Semver::is_release_identifier(ident) {
                if let Some(val) = build {
                    build = Some(format!("{val}.{ident}"));
                } else {
                    build = Some(ident.to_string());
                }
                build_prerelease_release_identifier = true;
            } else {
                prerelease = Some(ident.to_string());
            }

            text
        } else {
            // (Prerelease::EMPTY, text)
            text
        };

        let text = if let Some(text) = text.strip_prefix('+') {
            if build.is_some() {
                // TODO: return error
            }

            let (ident, text) = build_identifier(text)?;
            if ident.is_empty() {
                // TODO: return Err(Error::new(ErrorKind::EmptySegment(pos)));
                return Err(SomeverError::Invalid);
            }

            // This logic is defined in lenient-semver crate. Does it make sense to keep?
            // if we already have build then append with a dot separator
            if let Some(val) = build {
                build = Some(format!("{val}.{ident}"));
            } else {
                build = Some(ident.to_string());
            }

            text
        } else {
            // (BuildMetadata::EMPTY, text)
            text
        };

        if let Some(unexpected) = text.chars().next() {
            return Err(SomeverError::Invalid);
            // return Err(Error::new(ErrorKind::UnexpectedCharAfter(pos, unexpected)));
        }

        Ok(Self {
            prefix: prefix.map(ToString::to_string),
            major,
            minor,
            patch,
            prerelease,
            lenient_prerelease,
            build,
            build_prerelease_release_identifier,
        })
    }
}

fn get_numeric_identifier(input: &str) -> SomeverResult<(u64, &str)> {
    let (found_dot, text) = dot(input);
    if found_dot {
        // TODO: how to handle invalid case like "1."
        if let Some((num, text)) = numeric_identifier(text).ok() {
            return Ok((num, text));
        }

        if text.is_empty() {
            return Err(SomeverError::Invalid);
        }
    }

    Ok((0, input))
}

fn numeric_identifier(input: &str) -> SomeverResult<(u64, &str)> {
    let mut len = 0;
    let mut value = 0u64;

    while let Some(&digit) = input.as_bytes().get(len) {
        if digit < b'0' || digit > b'9' {
            break;
        }

        match value
            .checked_mul(10)
            .and_then(|value| value.checked_add((digit - b'0') as u64))
        {
            Some(sum) => value = sum,
            // None => return Err(Error::new(ErrorKind::Overflow(pos))),
            None => return Err(SomeverError::Invalid),
        }
        len += 1;
    }

    if len > 0 {
        Ok((value, &input[len..]))
    } else if let Some(unexpected) = input[len..].chars().next() {
        // Err(Error::new(ErrorKind::UnexpectedChar(pos, unexpected)))
        Err(SomeverError::Invalid)
    } else {
        // Err(Error::new(ErrorKind::UnexpectedEnd(pos)))
        Err(SomeverError::Invalid)
    }
}

fn dot(input: &str) -> (bool, &str) {
    if let Some(rest) = input.strip_prefix('.') {
        (true, rest)
    } else {
        (false, input)
    }
}

fn version_prefix(input: &str) -> (Option<&str>, &str) {
    if let Some(c) = input.get(0..1) {
        if c.to_lowercase() == "v" {
            return (Some(c), &input[1..]);
        }
    }

    (None, input)
}

fn prerelease_identifier(input: &str) -> SomeverResult<(&str, &str)> {
    let (string, rest) = identifier(input)?;
    Ok((string, rest))
}

fn build_identifier(input: &str) -> SomeverResult<(&str, &str)> {
    let (string, rest) = identifier(input)?;
    Ok((string, rest))
}

fn lenient_identifiers(input: &str) -> SomeverResult<(Option<String>, Option<String>, &str)> {
    let mut position = 0;
    let mut accumulated_len = 0;
    let mut segment_len = 0;
    let mut accumulation_has_nondigit = false;
    let mut prerelease = None;
    let mut build = None;

    loop {
        match input.as_bytes().get(position) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                if build.is_none() && accumulated_len != 0 {
                    build = Some(
                        input
                            .split_at(accumulated_len - segment_len - 1)
                            .0
                            .to_string(),
                    );
                    accumulated_len = 0;
                }
                accumulation_has_nondigit = true;
                segment_len += 1;
            }
            boundary => {
                if segment_len == 0 {
                    return if accumulated_len == 0 && boundary != Some(&b'.') {
                        Ok((prerelease, build, input))
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        Err(SomeverError::Invalid)
                    };
                }

                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                } else {
                    if let Some(val) = &build {
                        // we already have a build identifier so this must be a prerelease
                        // add one to avoid including the dot ('.') boundary
                        prerelease = Some(input[val.len() + 1..].to_string())
                    } else {
                        // if there are non-digits we assume prerelease
                        if accumulation_has_nondigit {
                            prerelease = Some(input[..accumulated_len].to_string())
                        } else {
                            build = Some(input[..accumulated_len].to_string())
                        }
                    }

                    return Ok((prerelease, build, &input[position..]));
                }
            }
        }

        position += 1;
    }
}

fn identifier(input: &str) -> SomeverResult<(&str, &str)> {
    let mut accumulated_len = 0;
    let mut segment_len = 0;

    loop {
        match input.as_bytes().get(accumulated_len + segment_len) {
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                segment_len += 1;
            }
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            boundary => {
                if segment_len == 0 {
                    return if accumulated_len == 0 && boundary != Some(&b'.') {
                        Ok(("", input))
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        Err(SomeverError::Invalid)
                    };
                }

                accumulated_len += segment_len;
                if boundary == Some(&b'.') || boundary == Some(&b'-') {
                    accumulated_len += 1;
                    segment_len = 0;
                } else {
                    return Ok(input.split_at(accumulated_len));
                }
            }
        }
    }
}

impl Semver {
    pub fn parse(text: String) -> SomeverResult<Self> {
        Semver::from_str(text.as_str())
    }

    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            prefix: None,
            major,
            minor,
            patch,
            prerelease: None,
            lenient_prerelease: false,
            build: None,
            build_prerelease_release_identifier: false,
        }
    }

    fn is_release_identifier(v: &str) -> bool {
        v == "r" || v.eq_ignore_ascii_case("final") || v.eq_ignore_ascii_case("release")
    }
}

impl Display for Semver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if let Some(prefix) = &self.prefix {
            write!(f, "{}", prefix)?;
        }

        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;

        if let Some(prerelease) = &self.prerelease {
            let separator = prerelease_separator(self.lenient_prerelease);
            write!(f, "{}{}", separator, prerelease)?;
        }

        if let Some(build) = &self.build {
            if self.build_prerelease_release_identifier {
                let separator = prerelease_separator(self.lenient_prerelease);
                write!(f, "{}{}", separator, build)?;
            } else {
                write!(f, "+{}", build)?;
            }
        }

        Ok(())
    }
}

fn prerelease_separator(lenient: bool) -> &'static str {
    if lenient {
        "."
    } else {
        "-"
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::semantic::Semver;
    use crate::SomeverResult;

    fn full_semver(
        prefix: Option<&str>,
        major: u64,
        minor: u64,
        patch: u64,
        prerelease: Option<&str>,
        lenient_prerelease: bool,
        build: Option<&str>,
        build_prerelease_release_identifier: bool,
    ) -> Semver {
        Semver {
            prefix: prefix.map(ToString::to_string),
            major,
            minor,
            patch,
            prerelease: prerelease.map(ToString::to_string),
            lenient_prerelease,
            build: build.map(ToString::to_string),
            build_prerelease_release_identifier,
        }
    }

    #[test_case("1" => Ok(Semver::new(1, 0, 0)); "major only")]
    #[test_case("1.2" => Ok(Semver::new(1, 2, 0)); "major.minor")]
    #[test_case("1.2.3" => Ok(Semver::new(1, 2, 3)); "major.minor.patch")]
    #[test_case(" 1.2.3  " => Ok(Semver::new(1, 2, 3)); "whitespace")]
    fn should_parse_simple_values(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("1.2.3-alpha1" => Ok(full_semver(None, 1, 2, 3, Some("alpha1"), false, None, false)))]
    #[test_case("  1.2.3-alpha2  " => Ok(full_semver(None, 1, 2, 3, Some("alpha2"), false, None, false)))]
    #[test_case("1.2.3-M13-beta3" => Ok(full_semver(None, 1, 2, 3, Some("M13-beta3"), false, None, false)))]
    #[test_case("1.2.3-alpha01.drop02" => Ok(full_semver(None, 1, 2, 3, Some("alpha01.drop02"), false, None, false)))]
    #[test_case("1.4.1-alpha01" => Ok(full_semver(None, 1, 4, 1, Some("alpha01"), false, None, false)))]
    #[test_case("1.4-alpha02" => Ok(full_semver(None, 1, 4, 0, Some("alpha02"), false, None, false)))]
    #[test_case("1-alpha03" => Ok(full_semver(None, 1, 0, 0, Some("alpha03"), false, None, false)))]
    #[test_case("1.9.3.RC1" => Ok(full_semver(None, 1, 9, 3, Some("RC1"), true, None, false)))]
    #[test_case("1.9.RC2" => Ok(full_semver(None, 1, 9, 0, Some("RC2"), true, None, false)))]
    #[test_case("1.RC3" => Ok(full_semver(None, 1, 0, 0, Some("RC3"), true, None, false)))]
    #[test_case("1.3.3-7" => Ok(full_semver(None, 1, 3, 3, Some("7"), false, None, false)))]
    #[test_case("5.9.0-202009080501-r" => Ok(full_semver(None, 5, 9, 0, Some("202009080501-r"), false, None, false)))]
    #[test_case("1.2.3.RC.4" => Ok(full_semver(None, 1, 2, 3, Some("RC.4"), true, None, false)))]
    fn should_parse_pre_release(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("1.2.3+build1" => Ok(full_semver(None, 1, 2, 3, None, false, Some("build1"), false)))]
    #[test_case("  1.2.3+build2  " => Ok(full_semver(None, 1, 2, 3, None, false, Some("build2"), false)))]
    #[test_case("3.1.0+build3-r021" => Ok(full_semver(None, 3, 1, 0, None, false, Some("build3-r021"), false)))]
    #[test_case("1.2.3+build1.drop02" => Ok(full_semver(None, 1, 2, 3, None, false, Some("build1.drop02"), false)))]
    #[test_case("1.4.1+build01" => Ok(full_semver(None, 1, 4, 1, None, false, Some("build01"), false)))]
    #[test_case("1.4+build02" => Ok(full_semver(None, 1, 4, 0, None, false, Some("build02"), false)))]
    #[test_case("1+build03" => Ok(full_semver(None, 1, 0, 0, None, false, Some("build03"), false)))]
    #[test_case("7.2.0+28-2f9fb552" => Ok(full_semver(None, 7, 2, 0, None, false, Some("28-2f9fb552"), false)))]
    #[test_case("1.3.3.7" => Ok(full_semver(None, 1, 3, 3, None, false, Some("7"), false)))]
    #[test_case("5.9.0.202009080501-r" => Ok(full_semver(None, 5, 9, 0, None, false, Some("202009080501.r"), true)))]
    fn should_parse_build(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("1.2.1.7" => Ok(full_semver(None, 1, 2, 1, None, false, Some("7"), false)))]
    #[test_case("1.2.2.0" => Ok(full_semver(None, 1, 2, 2, None, false, Some("0"), false)))]
    #[test_case("1.2.3.00" => Ok(full_semver(None, 1, 2, 3, None, false, Some("00"), false)))]
    #[test_case("1.2.4.07" => Ok(full_semver(None, 1, 2, 4, None, false, Some("07"), false)))]
    #[test_case("1.2.5.7.4.2" => Ok(full_semver(None, 1, 2, 5, None, false, Some("7.4.2"), false)))]
    #[test_case("1.2.6.7.04.02" => Ok(full_semver(None, 1, 2, 6, None, false, Some("7.04.02"), false)))]
    #[test_case("1.2.7.9876543210987654321098765432109876543210" => Ok(full_semver(None, 1, 2, 7, None, false, Some("9876543210987654321098765432109876543210"), false)))]
    #[test_case("1.2.8.9876543210987654321098765432109876543210.4.2" => Ok(full_semver(None, 1, 2, 8, None, false, Some("9876543210987654321098765432109876543210.4.2"), false)))]
    #[test_case("1.4.4.7.foo" => Ok(full_semver(None, 1, 4, 4, Some("foo"), true, Some("7"), false)))] // lenient has foo as pre-release and 7 as build
    #[test_case("1.4.3.7-bar" => Ok(full_semver(None, 1, 4, 3, Some("bar"), false, Some("7"), false)))] // do we want this behavior. lenient has bar as pre-release
    #[test_case("1.3.3.7+bar" => Ok(full_semver(None, 1, 3, 3, None, false, Some("7.bar"), false)))]
    fn should_allow_additional_numbers(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("1.2.3-alpha1+build5" => Ok(full_semver(None, 1, 2, 3, Some("alpha1"), false, Some("build5"), false)))]
    #[test_case("  1.2.3-alpha2+build6  " => Ok(full_semver(None, 1, 2, 3, Some("alpha2"), false, Some("build6"), false)))]
    #[test_case("1.2.3-1.alpha1.9+build5.7.3aedf" => Ok(full_semver(None, 1, 2, 3, Some("1.alpha1.9"), false, Some("build5.7.3aedf"), false)))]
    #[test_case("0.4.0-beta.1+0851523" => Ok(full_semver(None, 0, 4, 0, Some("beta.1"), false, Some("0851523"), false)))]
    fn should_allow_pre_release_and_build(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("2.7.3.Final" => Ok(full_semver(None, 2, 7, 3, None, true, Some("Final"), true)); "full dot final")] // should lenient prerelease be true?
    #[test_case("2.7.3-Final" => Ok(full_semver(None, 2, 7, 3, None, false, Some("Final"), true)); "full hyphen final")]
    #[test_case("2.7.3+Final" => Ok(full_semver(None, 2, 7, 3, None, false, Some("Final"), false)); "full plus final")]
    #[test_case("2.7.3.Release" => Ok(full_semver(None, 2, 7, 3, None, true, Some("Release"), true)); "full dot release")]
    #[test_case("2.7.3-Release" => Ok(full_semver(None, 2, 7, 3, None, false, Some("Release"), true)); "full hyphen release")]
    #[test_case("2.7.3+Release" => Ok(full_semver(None, 2, 7, 3, None, false, Some("Release"), false)); "full plus release")]
    #[test_case("2.7.Final" => Ok(full_semver(None, 2, 7, 0, None, true, Some("Final"), true)); "minor dot final")]
    #[test_case("2.7-Final" => Ok(full_semver(None, 2, 7, 0, None, false, Some("Final"), true)); "minor hyphen final")]
    #[test_case("2.7+Final" => Ok(full_semver(None, 2, 7, 0, None, false, Some("Final"), false)); "minor plus final")]
    #[test_case("2.7.Release" => Ok(full_semver(None, 2, 7, 0, None, true, Some("Release"), true)); "minor dot release")]
    #[test_case("2.7-Release" => Ok(full_semver(None, 2, 7, 0, None, false, Some("Release"), true)); "minor hyphen release")]
    #[test_case("2.7+Release" => Ok(full_semver(None, 2, 7, 0, None, false, Some("Release"), false)); "minor plus release")]
    #[test_case("2.Final" => Ok(full_semver(None, 2, 0, 0, None, true, Some("Final"), true)); "major dot final")]
    #[test_case("2-Final" => Ok(full_semver(None, 2, 0, 0, None, false, Some("Final"), true)); "major hyphen final")]
    #[test_case("2+Final" => Ok(full_semver(None, 2, 0, 0, None, false, Some("Final"), false)); "major plus final")]
    #[test_case("2.Release" => Ok(full_semver(None, 2, 0, 0, None, true, Some("Release"), true)); "major dot release")]
    #[test_case("2-Release" => Ok(full_semver(None, 2, 0, 0, None, false, Some("Release"), true)); "major hyphen release")]
    #[test_case("2+Release" => Ok(full_semver(None, 2, 0, 0, None, false, Some("Release"), false)); "major plus release")]
    #[test_case("2.7.3.r" => Ok(full_semver(None, 2, 7, 3, None, true, Some("r"), true)); "full dot r")]
    #[test_case("2.7.3-r" => Ok(full_semver(None, 2, 7, 3, None, false, Some("r"), true)); "full hyphen r")]
    #[test_case("2.7.3+r" => Ok(full_semver(None, 2, 7, 3, None, false, Some("r"), false)); "full plus r")]
    #[test_case("2.7.r" => Ok(full_semver(None, 2, 7, 0, None, true, Some("r"), true)); "minor dot r")]
    #[test_case("2.7-r" => Ok(full_semver(None, 2, 7, 0, None, false, Some("r"), true)); "minor hyphen r")]
    #[test_case("2.7+r" => Ok(full_semver(None, 2, 7, 0, None, false, Some("r"), false)); "minor plus r")]
    #[test_case("2.r" => Ok(full_semver(None, 2, 0, 0, None, true, Some("r"), true)); "major dot r")]
    #[test_case("2-r" => Ok(full_semver(None, 2, 0, 0, None, false, Some("r"), true)); "major hyphen r")]
    #[test_case("2+r" => Ok(full_semver(None, 2, 0, 0, None, false, Some("r"), false)); "major plus r")]
    fn should_allow_release_identifier(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("1" => Ok(full_semver(None, 1, 0, 0, None, false, None, false)))]
    #[test_case("01" => Ok(full_semver(None, 1, 0, 0, None, false, None, false)))]
    #[test_case("00001" => Ok(full_semver(None, 1, 0, 0, None, false, None, false)))]
    #[test_case("1.2.3-1" => Ok(full_semver(None, 1, 2, 3, Some("1"), false, None, false)))]
    #[test_case("1.2.3-01" => Ok(full_semver(None, 1, 2, 3, Some("01"), false, None, false)))]
    #[test_case("1.2.3-0001" => Ok(full_semver(None, 1, 2, 3, Some("0001"), false, None, false)))]
    #[test_case("2.3.4+1" => Ok(full_semver(None, 2, 3, 4, None, false, Some("1"), false)))]
    #[test_case("2.3.4+01" => Ok(full_semver(None, 2, 3, 4, None, false, Some("01"), false)))]
    #[test_case("2.3.4+0001" => Ok(full_semver(None, 2, 3, 4, None, false, Some("0001"), false)))]
    fn should_allow_leading_zeros(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("v1" => Ok(full_semver(Some("v"), 1, 0, 0, None, false, None, false)))]
    #[test_case("  v2  " => Ok(full_semver(Some("v"), 2, 0, 0, None, false, None, false)))]
    #[test_case("v1.2.3" => Ok(full_semver(Some("v"), 1, 2, 3, None, false, None, false)))]
    #[test_case("v1.2.3-7" => Ok(full_semver(Some("v"), 1, 2, 3, Some("7"), false, None, false)))]
    #[test_case("V3" => Ok(full_semver(Some("V"), 3, 0, 0, None, false, None, false)))]
    #[test_case("  V5  " => Ok(full_semver(Some("V"), 5, 0, 0, None, false, None, false)))]
    #[test_case("V2.3.4" => Ok(full_semver(Some("V"), 2, 3, 4, None, false, None, false)))]
    #[test_case("V4.2.4-2" => Ok(full_semver(Some("V"), 4, 2, 4, Some("2"), false, None, false)))]
    fn should_allow_version_prefix(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }

    #[test_case("" => Err(SomeverError::Empty); "empty input")]
    #[test_case(" " => Err(SomeverError::Invalid); "whitespace input")]
    #[test_case("." => Err(SomeverError::Invalid); "dot")]
    #[test_case("🙈" => Err(SomeverError::Invalid); "emoji")]
    #[test_case("v" => Err(SomeverError::Invalid); "v")]
    #[test_case("val" => Err(SomeverError::Invalid); "val")]
    #[test_case("1." => Err(SomeverError::Invalid); "eoi after major")]
    #[test_case("1.2.3-" => Err(SomeverError::Invalid); "eoi after hyphen")]
    #[test_case("1.2.3- " => Err(SomeverError::Invalid); "whitespace after hyphen")]
    #[test_case("1.2.3+" => Err(SomeverError::Invalid); "eoi after plus")]
    #[test_case("1.2.3+ " => Err(SomeverError::Invalid); "whitespace after plus")]
    #[test_case("1.2.3-." => Err(SomeverError::Invalid); "prerelease trailing dot")]
    #[test_case("1.2.3--" => Err(SomeverError::Invalid); "prerelease trailing hyphen")]
    #[test_case("1.2.3-+" => Err(SomeverError::Invalid); "prerelease trailing plus")]
    #[test_case("1.2.3-🙈" => Err(SomeverError::Invalid); "prerelease trailing emoji")]
    #[test_case("1.2.3+." => Err(SomeverError::Invalid); "build trailing dot")]
    #[test_case("1.2.3+-" => Err(SomeverError::Invalid); "build trailing hyphen")]
    #[test_case("1.2.3++" => Err(SomeverError::Invalid); "build trailing plus")]
    #[test_case("1.2.3+🙈" => Err(SomeverError::Invalid); "build trailing emoji")]
    #[test_case("v.1.2.3" => Err(SomeverError::Invalid); "v followed by dot")]
    #[test_case("v-1.2.3" => Err(SomeverError::Invalid); "v followed by hyphen")]
    #[test_case("v+1.2.3" => Err(SomeverError::Invalid); "v followed by plus")]
    #[test_case("vv1.2.3" => Err(SomeverError::Invalid); "v followed by v")]
    #[test_case("v v1.2.3" => Err(SomeverError::Invalid); "v followed by whitespace")]
    #[test_case("a1.2.3" => Err(SomeverError::Invalid); "starting with a")]
    #[test_case("a.b.c" => Err(SomeverError::Invalid); "non-numeric major")]
    #[test_case("1.+.0" => Err(SomeverError::Invalid); "plus as minor")]
    #[test_case("1.2.." => Err(SomeverError::Invalid); "dot as patch")]
    #[test_case("123456789012345678901234567890" => Err(SomeverError::Invalid); "number overflows u64")]
    #[test_case("1 abc" => Err(SomeverError::Invalid); "a following parsed number 1")]
    #[test_case("1.2.3 abc" => Err(SomeverError::Invalid); "a following parsed number 1.2.3")]
    #[test_case("1.*" => Err(SomeverError::Invalid); "asterisk as early prerelease")]
    #[test_case("1.2.3-*" => Err(SomeverError::Invalid); "asterisk as prerelease")]
    #[test_case("1.2.3-ab*" => Err(SomeverError::Invalid); "asterisk within prerelease")]
    #[test_case("1.?" => Err(SomeverError::Invalid); "question mark as early prerelease")]
    #[test_case("1.2.3-?" => Err(SomeverError::Invalid); "question mark as prerelease")]
    #[test_case("1.2.3-ab?" => Err(SomeverError::Invalid); "question mark within prerelease")]
    fn should_error_when_invalid(input: &str) -> SomeverResult<Semver> {
        Semver::parse(input.into())
    }
}
