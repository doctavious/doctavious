use crate::{SomeverError, SomeverResult};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// This wraps the semver crate's implementation
/// This is a hack however we need support for a more lenient semver implementation and
/// unfortunately the lenient-semver crate doesn't appear to be maintained and at the moment I
/// would like to avoid maintaining something similar unless completely necessary.
/// The main things we want to support is
/// - allow a leaning v or V (e.g. "v1.2.3" parses as "1.2.3")
/// - allow leading zeros
/// - Minor and Patch are optional an default to 0 (e.g. "1" parses as "1.0.0") - TODO
/// - Additional numeric identifiers are parsed as build identifier (e.g "1.2.3.4.5" parses as "1.2.3+4.5" - TODO
/// - Pre-release identifier may be separated by '.' as well (e.g. "1.2.3.rc1" parses as "1.2.3-rc1").
/// - Some pre-release identifiers (e.g. ".Final", "-final") should be parsed as build identifier
///     (e.g. "1.2.3.Final" parses as "1.2.3+Final").
/// - For all of the above we want to keep a lossless representation so that if we write/display
///     the string it remains unchanged from the original value provided
#[derive(Debug, PartialEq)]
pub struct Semver {
    // raw: String,
    // prefix: Option<String>,
    // // true if prerelease is separated by a dot rather than the strict dash
    // lenient_prerelease_separator: bool,
    // prerelease_identifier_as_build_identifier: bool,
    // inner: semver::Version

    pub prefix: Option<String>,
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    // pub pre: Prerelease,
    pub prerelease: Option<String>,
    pub lenient_prerelease: bool,
    // pub build: BuildMetadata,
    pub build: Option<String>,
    pub build_prerelease_release_identifier: bool
}

impl FromStr for Semver {
    type Err = SomeverError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        // let raw = text.to_string();
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        // TODO: simple state machine might help here
        let (prefix, text) = version_prefix(text.trim());
        let (major, text) = numeric_identifier(text)?;
        let (minor, text) = get_numeric_identifier(text);
        let (patch, text) = get_numeric_identifier(text);

        let mut prerelease = None;
        let mut lenient_prerelease = false;
        let mut build = None;
        let mut build_prerelease_release_identifier = false;

        let text = if let Some(text) = text.strip_prefix('.') {
            // let (ident, text) = identifier(text)?;
            // if ident.is_empty() {
            //
            // }

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

            // let t = if Semver::is_release_identifier(ident) {
            //     build = Some(ident.to_string());
            //     lenient_prerelease = true;
            //     build_prerelease_release_identifier = true;
            //     text
            // } else {
            //     // let pr = prerelease_identifier(ident);
            //     // println!("pr: {:?}", pr);
            //     //
            //     // let n = numeric_identifier(ident);
            //     // println!("n: {:?}", n);
            //     //
            //     // let i = identifier(ident);
            //     // println!("i: {:?}", i);
            //     //
            //     // let nb = numeric_build(ident);
            //     // println!("nb: {:?}", nb);
            //     //
            //     // let l = lenient_identifier(ident);
            //     // println!("l: {:?}", l);
            //
            //     let (pr, b, text) = lenient_identifiers(ident)?;
            //     prerelease = pr;
            //     build = b;
            //     text
            //     // println!("is: {:?}", is);
            //
            //     // bit of a hack for now but if first char starts with a number we'll assume its a
            //     // numeric build identifier.
            //     // if ident.chars().next().unwrap_or_default().is_numeric() {
            //     //     build = Some(ident.to_string())
            //     //     // TODO: need a flag for this scenario. Ex: 1.3.3.7
            //     // } else {
            //     //     lenient_prerelease = true;
            //     //     prerelease = Some(ident);
            //     // }
            // };

            text
        } else {
            text
        };


        let text = if let Some(text) = text.strip_prefix('-') {
            if prerelease.is_some() {
                // return error
            }

            let (ident, text) = prerelease_identifier(text)?;
            if ident.is_empty() {
                // return Err(Error::new(ErrorKind::EmptySegment(pos)));
            }

            // let pr = prerelease_identifier(ident);
            // println!("pr: {:?}", pr);
            //
            // let i = identifier(ident);
            // println!("i: {:?}", i);
            //
            // let is = lenient_identifiers(ident);
            // println!("is: {:?}", is);

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
                // return error
            }

            let (ident, text) = build_identifier(text)?;
            if ident.is_empty() {
                // return Err(Error::new(ErrorKind::EmptySegment(pos)));
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


        Ok(Self {
            // raw,
            prefix: prefix.map(ToString::to_string),
            major,
            minor,
            patch,
            prerelease,
            lenient_prerelease,
            build,
            build_prerelease_release_identifier
        })

        // get major
        // optionally get minor / patch
        // if after dot is not numeric identifier use as pre-release (unless release identifier)
        // get pre-release (supporting dot '.') and check if release identifier

        // let mut pos = Position::Major;
        // let (major, text) = numeric_identifier(text, pos)?;
        // let text = dot(text, pos)?;
        //
        // pos = Position::Minor;
        // let (minor, text) = numeric_identifier(text, pos)?;
        // let text = dot(text, pos)?;
        //
        // pos = Position::Patch;
        // let (patch, text) = numeric_identifier(text, pos)?;

        // if text.is_empty() {
        //     return Ok(Version::new(major, minor, patch));
        // }
        //
        // let (pre, text) = if let Some(text) = text.strip_prefix('-') {
        //     pos = Position::Pre;
        //     let (pre, text) = prerelease_identifier(text)?;
        //     if pre.is_empty() {
        //         return Err(Error::new(ErrorKind::EmptySegment(pos)));
        //     }
        //     (pre, text)
        // } else {
        //     (Prerelease::EMPTY, text)
        // };


    }

}

fn get_numeric_identifier(input: &str) -> (u64, &str) {
    let (found_dot, text) = dot(input);
    if found_dot {
        if let Some((num, text)) = optional_numeric_identifier(text) {
            return (num, text);
        }
    }

    (0, input)
}

fn optional_numeric_identifier(input: &str) -> Option<(u64, &str)> {
    numeric_identifier(input).ok()
}

fn numeric_identifier(input: &str) -> SomeverResult<(u64, &str)> {
    let mut len = 0;
    let mut value = 0u64;

    while let Some(&digit) = input.as_bytes().get(len) {
        if digit < b'0' || digit > b'9' {
            break;
        }

        // if value == 0 && len > 0 {
        //     // return Err(Error::new(ErrorKind::LeadingZero(pos)));
        //     return Err(SomeverError::Invalid());
        // }

        match value
            .checked_mul(10)
            .and_then(|value| value.checked_add((digit - b'0') as u64))
        {
            Some(sum) => value = sum,
            // None => return Err(Error::new(ErrorKind::Overflow(pos))),
            None => return Err(SomeverError::Invalid()),
        }
        len += 1;
    }

    if len > 0 {
        Ok((value, &input[len..]))
    } else if let Some(unexpected) = input[len..].chars().next() {
        // Err(Error::new(ErrorKind::UnexpectedChar(pos, unexpected)))
        Err(SomeverError::Invalid())
    } else {
        // Err(Error::new(ErrorKind::UnexpectedEnd(pos)))
        Err(SomeverError::Invalid())
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
    let mut segment_has_nondigit = false;
    let mut prerelease = None;
    let mut build = None;

    loop {
        match input.as_bytes().get(position) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                if build.is_none() && accumulated_len != 0 {
                    build = Some(input.split_at(accumulated_len - segment_len - 1).0.to_string());
                    accumulated_len = 0;
                }
                accumulation_has_nondigit = true;
                segment_len += 1;
            }
            boundary => {
                if segment_len == 0 {
                    if accumulated_len == 0 && boundary != Some(&b'.') {
                        return Ok((prerelease, build, input));
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        return Err(SomeverError::Invalid())
                    }
                }

                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                    segment_has_nondigit = false;
                } else {
                    // if accumulation_has_nondigit {
                    //     prerelease = Some(input[..accumulated_len].to_string());
                    // } else {
                    //     build = Some(input[..accumulated_len].to_string());
                    // }

                    if let Some(val) = &build {
                        // we already have a build identifier so this must be a prerelease
                        // add one to avoid including the dot ('.') boundary
                        prerelease = Some(input[val.len() + 1..].to_string())
                    } else {
                        if accumulation_has_nondigit {
                            prerelease = Some(input[..accumulated_len].to_string())
                        } else {
                            build = Some(input[..accumulated_len].to_string())
                        }
                        // build = Some(input[..accumulated_len].to_string())
                    }

                    return Ok((prerelease, build, &input[position..]));
                }
            }
        }

        position += 1;
    }
}

fn lenient_identifier(input: &str) -> SomeverResult<(&str, &str)> {
    let mut accumulated_len = 0;
    let mut segment_len = 0;
    let mut segment_has_nondigit = false;

    loop {
        match input.as_bytes().get(accumulated_len + segment_len) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                return Ok(input.split_at(accumulated_len));
            }
            boundary => {
                if segment_len == 0 {
                    if accumulated_len == 0 && boundary != Some(&b'.') {
                        return Ok(("", input));
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        return Err(SomeverError::Invalid())
                    }
                }

                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                    segment_has_nondigit = false;
                } else {
                    return Ok(input.split_at(accumulated_len));
                }
            }
        }
    }
}

fn numeric_build(input: &str) -> SomeverResult<(&str, &str)> {
    let mut accumulated_len = 0;
    let mut segment_len = 0;
    let mut segment_has_nondigit = false;

    loop {
        match input.as_bytes().get(accumulated_len + segment_len) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                return Ok(input.split_at(accumulated_len));
            }
            boundary => {
                if segment_len == 0 {
                    if accumulated_len == 0 && boundary != Some(&b'.') {
                        return Ok(("", input));
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        return Err(SomeverError::Invalid())
                    }
                }
                // if pos == Position::Pre
                //     && segment_len > 1
                //     && !segment_has_nondigit
                //     && input[accumulated_len..].starts_with('0')
                // {
                //     return Err(Error::new(ErrorKind::LeadingZero(pos)));
                // }
                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                    segment_has_nondigit = false;
                } else {
                    return Ok(input.split_at(accumulated_len));
                }
            }
        }
    }
}

fn identifier(input: &str) -> SomeverResult<(&str, &str)> {
    let mut accumulated_len = 0;
    let mut segment_len = 0;
    let mut segment_has_nondigit = false;

    loop {
        match input.as_bytes().get(accumulated_len + segment_len) {
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') | Some(b'-') => {
                segment_len += 1;
                segment_has_nondigit = true;
            }
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            boundary => {
                if segment_len == 0 {
                    if accumulated_len == 0 && boundary != Some(&b'.') {
                        return Ok(("", input));
                    } else {
                        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                        return Err(SomeverError::Invalid())
                    }
                }
                // if pos == Position::Pre
                //     && segment_len > 1
                //     && !segment_has_nondigit
                //     && input[accumulated_len..].starts_with('0')
                // {
                //     return Err(Error::new(ErrorKind::LeadingZero(pos)));
                // }
                accumulated_len += segment_len;
                if boundary == Some(&b'.') {
                    accumulated_len += 1;
                    segment_len = 0;
                    segment_has_nondigit = false;
                } else {
                    return Ok(input.split_at(accumulated_len));
                }
            }
        }
    }
}

impl Semver {

    // TODO: This code is trash and needs a revisit
    pub fn parse(text: String) -> SomeverResult<Self> {
        // let mut lenient_prerelease_separator = false;
        // let mut prerelease_identifier_as_build_identifier = false;
        //
        // let mut inner = text.clone().to_string();
        //
        // let mut prefix = None;
        // if let Some(c) = inner.get(0..1) {
        //     if c.to_lowercase() == "v" {
        //         prefix = Some(c.to_string());
        //         inner = inner[1..].to_string();
        //     }
        // }
        //
        // // TODO: this is probably the dumb way to do this but good enough for now
        // let periods = inner.chars().filter(|c| *c == '.').count();
        // if periods == 3 {
        //     // semver doesnt allow 3 periods so lets convert the 3rd to a dash and presume its
        //     // a pre-release identifier which we'll correct if its not next.
        //     // TODO: could probably handle determining pre-release or not here rather than an additional
        //     //       step.
        //     if let Some(i) = inner.rfind('.') {
        //         inner.replace_range(i..i+1,"-");
        //         lenient_prerelease_separator = true;
        //     }
        // }
        //
        // let mut parsed = semver::Version::parse(inner.as_str())?;
        // if Self::is_release_identifier(parsed.pre.as_str()) {
        //     parsed.build = BuildMetadata::new(parsed.pre.as_str())?;
        //     parsed.pre = Prerelease::EMPTY;
        //     prerelease_identifier_as_build_identifier = true;
        // }
        //
        // Ok(Self {
        //     raw: text.to_string(),
        //     inner: parsed,
        //     prefix,
        //     lenient_prerelease_separator,
        //     prerelease_identifier_as_build_identifier
        // })
        Semver::from_str(text.as_str())
    }


    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Self {
            // raw: format!("{major}.{minor}.{patch}"),
            prefix: None,
            major,
            minor,
            patch,
            prerelease: None,
            lenient_prerelease: false,
            build: None,
            build_prerelease_release_identifier: false
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
            let separator = if self.lenient_prerelease {
              "."
            } else {
                "-"
            };
            write!(f, "{}{}", separator, prerelease)?;
        }

        if let Some(build) = &self.build {
            if self.build_prerelease_release_identifier {
                let separator = if self.lenient_prerelease {
                    "."
                } else {
                    "-"
                };
                write!(f, "{}{}", separator, build)?;
            } else {
                write!(f, "+{}", build)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::semantic::Semver;
    use crate::SomeverResult;
    use test_case::test_case;

    fn full_semver(
        prefix: Option<&str>,
        major: u64,
        minor: u64,
        patch: u64,
        prerelease: Option<&str>,
        lenient_prerelease: bool,
        build: Option<&str>,
        build_prerelease_release_identifier: bool
    ) -> Semver {
        Semver {
            prefix: prefix.map(ToString::to_string),
            major,
            minor,
            patch,
            prerelease: prerelease.map(ToString::to_string),
            lenient_prerelease,
            build: build.map(ToString::to_string),
            build_prerelease_release_identifier
        }
    }


    // 5.9.0.202009080501-r
    // 1.2.3.4.5
    // 1.3.3.7+baz => build: 7.bar
    // 1.3.3.7-bar => release: bar / build: 7
    // 1.4.3.7-bar
    #[test]
    fn t() {
        // 1.3.3.7-bar
        // pr: Ok(("7-bar", ""))
        // n: Ok((7, "-bar"))
        // i: Ok((7, "-bar"))

        // 1.3.3.7+baz
        // pr: Ok(("7", ""))
        // n: Ok((7, ""))
        // i: Ok((7, ""))

        // 1.4.4.7.foo

        // 1.2.6.7.04.02
        // pr: Ok(("7.04.02", ""))
        // n: Ok((7, ".04.02"))
        // i: Ok((7, ".04.02"))

        // 1.3.3-12.3.foo+bar
        // 1.4.4.7.foo
        // 5.9.0.202009080501-r
        let s = Semver::parse("00001".to_string()).unwrap();
        println!("{:?}", s);
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

}