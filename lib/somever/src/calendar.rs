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

// TODO: Good validations - https://github.com/k1LoW/calver/blob/main/token.go#L182

use std::cmp::min;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chrono::{DateTime, Datelike, TimeZone, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{SomeverError, SomeverResult};

lazy_static! {
    // (?<major>\d+)[._-](?<minor>\d+)([._-](?<micro>\d+))?((?<modifier_sep>[._-])(?<modifier>.+))?
    static ref RE: Regex =
        Regex::new(r"(?<major>\d+)[.-_](?<minor>\d+)([.-_](?<micro>\d+))?(?<modifier>.+)?").unwrap();
}

/// FullYear notation for CalVer - 2006, 2016, 2106
const FULL_YEAR: &str = "YYYY";
/// ShortYear notation for CalVer - 6, 16, 106
const SHORT_YEAR: &str = "YY";
/// PaddedYear notation for CalVer - 06, 16, 106
const PADDED_YEAR: &str = "0Y";
/// ShortMonth notation for CalVer - 1, 2 ... 11, 12
const SHORT_MONTH: &str = "MM";
/// PaddedMonth notation for CalVer - 01, 02 ... 11, 12
const PADDED_MONTH: &str = "0M";
/// ShortWeek notation for CalVer - 1, 2, 33, 52
const SHORT_WEEK: &str = "WW";
/// PaddedWeek notation for CalVer - 01, 02, 33, 52
const PADDED_WEEK: &str = "0W";
/// ShortDay notation for CalVer - 1, 2 ... 30, 31
const SHORT_DAY: &str = "DD";
/// PaddedDay notation for CalVer - 01, 02 ... 30, 31
const PADDED_DAY: &str = "0D";

// support for YYYY.MM.DD_MICRO which could work if _MICRO is a modifier or DD_MICRO is the micro
// TODO: Confirm sorting/ordering especially with modifier
// TODO: Handling specific formats? Ex: 0M - Zero-padded month. As of right now we provide a lossy
//      conversion where if user provides "2024-06-08" we would output "2024-6-8"
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Calver {
    pub prefix: Option<String>,
    pub major: u16,
    pub minor: u8,
    pub patch: Option<u16>,
    // generally discouraged by Calver however potentially useful in rare scenarios such as
    // fusefs-ntfs which uses a format of YYYY.MM.DD_MICRO
    // not in love with build as the name. Could do patch and micro rather than micro and build
    pub micro: Option<u16>,
    pub separator: char,
    // pub modifier_separator: Option<String>
    pub modifier: Option<String>,
    pub format: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum Position {
    Major,
    Minor,
    Patch,
    Micro,
    Modifier,
}

impl Position {

    fn next_token_position(&self) -> Option<Self> {
        match self {
            Position::Major => Some(Position::Minor),
            Position::Minor => Some(Position::Patch),
            Position::Patch => Some(Position::Micro),
            Position::Micro => None,
            _ => None
        }
    }

    fn next(&self, numeric: bool) -> Option<Self> {
        match (self, numeric) {
            (Position::Major, _) => Some(Position::Minor),
            (Position::Minor, true) => Some(Position::Patch),
            (Position::Minor, false) => Some(Position::Modifier),
            (Position::Patch, true) => Some(Position::Micro),
            (Position::Patch, false) => Some(Position::Modifier),
            (Position::Micro, _) => Some(Position::Modifier),
            (Position::Modifier, _) => None,
        }
    }
}

impl Calver {
    pub fn new(format: String, modifier: Option<String>) -> SomeverResult<Self> {
        // TODO: improve this
        let d = chrono::Utc::now();

        // let (format_segments, separators) = Calver::parse_format_pattern(&format)?;
        // let major = format_segments.get(0).unwrap().conv(d);
        // let minor = format_segments.get(1).unwrap().conv(d);
        // let patch = format_segments.get(2).and_then(|s| Some(s.conv(d)));
        // let micro = format_segments.get(3).and_then(|s| Some(s.conv(d)));

        let tokenized = TokenizedFormat::tokenize(&format)?;
        let major = tokenized.get_token(Position::Major).unwrap().convention.conv(d);
        let minor = tokenized.get_token(Position::Minor).unwrap().convention.conv(d);
        let patch = tokenized.get_token(Position::Patch).and_then(|s| Some(s.convention.conv(d)));
        let micro = tokenized.get_token(Position::Micro).and_then(|s| Some(s.convention.conv(d)));

        // TODO: use ?
        Ok(Self {
            prefix: tokenized.prefix,
            major: major.parse::<u16>().unwrap(),
            minor: minor.parse::<u8>().unwrap(),
            patch: patch.and_then(|v| Some(v.parse::<u16>().unwrap())),
            micro: micro.and_then(|v| Some(v.parse::<u16>().unwrap())),
            separator: '.',
            modifier,
            format
        })
    }

    // support YYYY.MM.DD_MICRO?
    pub fn parse(text: &str, format: &str) -> SomeverResult<Self> {
        // Calver::from_str(text)

        // let format_segments = Calver::parse_format(&format)?;
        // let (format_segments, format_separators) = Calver::parse_format_pattern(&format)?;
        let tokenized = TokenizedFormat::tokenize(&format)?;
        let (text_segments, text_separators) = Calver::parse_text(&text)?;

        // TODO: handle validation when modifier segment is provided
        // if format_segments.len() != text_segments.len() {
        //     // TODO: number of segments on value does not match pattern
        //     return Err(SomeverError::Invalid);
        // }

        // TODO: validate format/separator
        // if !tokenized.separators.eq(&text_separators) {
        //     return Err(SomeverError::Invalid);
        // }

        let mut major = 0;
        let mut minor = 0;
        let mut patch = None;
        let mut micro = None;
        let mut separator = '.';
        let mut modifier = None;

        for (pos, segment) in tokenized.tokens.iter().enumerate() {
            // let convention = Conventions::new(pat)?;
            // TODO: there has to be a better way to convert/validate.
            //       The the parse dont validate would just have a parse method on something that
            //       would take care of the conversion
            let v = text_segments.get(pos).unwrap().parse::<u16>().unwrap();
            if !segment.convention.validate(v) {
                // TODO: return error
            }

            match segment.position {
                Position::Major => {
                    major = v;
                }
                Position::Minor => {
                    // TODO: dont use unwrap
                    minor = u8::try_from(v).unwrap();
                }
                Position::Patch => {
                    patch = Some(v);
                }
                Position::Micro => {
                    micro = Some(v);
                }
                Position::Modifier => {}
            }
        }

        let segment_difference = text_segments.len() - tokenized.tokens.len();
        if segment_difference == 1 {
            if let Some(modifier_text) = text_segments.get(tokenized.tokens.len()) {
                modifier = Some(modifier_text.to_string());
            }
        } else if segment_difference > 1 {
            // TODO: return error
        }

        Ok(Self {
            prefix: tokenized.prefix,
            major,
            minor,
            patch,
            micro,
            separator,
            modifier,
            format: format.to_string(),
        })
    }

    fn parse_format(format: &str) -> SomeverResult<Vec<String>> {
        let caps = RE
            .captures(format)
            .ok_or(SomeverError::InvalidFormat(format.to_string()))?;

        Ok(caps
            .iter()
            .flatten()
            .map(|c| c.as_str().to_string())
            .collect())
    }

    pub(crate) fn parse_format_pattern(
        text: &str,
    ) -> SomeverResult<(Vec<Conventions>, Vec<String>)> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut t = text.to_string();

        let mut segments = Vec::new();
        let mut separators = Vec::new();
        while segments.len() < 4 {
            // let (identifier, text) = format_identifier(t.as_str())?;
            let (ident, has_nondigit, separator, text) = identifier(t.as_str())?;
            segments.push(Conventions::new(ident)?);
            separators.push(separator.to_string());
            t = text.to_string();
            if t.is_empty() {
                break;
            }
        }

        if !t.is_empty() {
            // TODO: only support up to 4 segments
            return Err(SomeverError::Invalid);
        }

        if segments.len() < 2 {
            return Err(SomeverError::Invalid);
        }

        Ok((segments, separators))
    }

    pub(crate) fn parse_text(text: &str) -> SomeverResult<(Vec<String>, Vec<String>)> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut t = text.to_string();
        let mut segments = Vec::new();
        let mut separators = Vec::new();
        while segments.len() < 4 {
            // let (identifier, text) = format_identifier(t.as_str())?;
            let (ident, has_nondigit, separator, text) = identifier(t.as_str())?;
            segments.push(ident.to_string());

            if !separator.is_empty() {
                separators.push(separator.to_string());
            }

            // TODO: is this necessary?
            // assume identifier with nondigit is modifier and we can stop parsing
            // if has_nondigit {
            //     break;
            // }

            t = text.to_string();
            if t.is_empty() {
                break;
            }
        }

        if segments.len() < 2 {
            return Err(SomeverError::Invalid);
        }

        Ok((segments, separators))
    }

    // pub fn parse_without_regex(text: &str) -> SomeverResult<Self> {
    //     if text.is_empty() {
    //         return Err(SomeverError::Empty);
    //     }
    //
    //     let mut format = String::new();
    //
    //     let (major, text) = numeric_identifier(text)?;
    //     let (found, text, boundary) = separator(text, true)?;
    //     let (minor, text) = numeric_identifier(text)?;
    //     let text = if let Ok((true, text, boundary)) = separator(text, false) {
    //         if let Ok(patch) = numeric_identifier(text) {
    //             ""
    //         } else if let Ok((modifier, text)) = identifier(text) {
    //             ""
    //         } else {
    //             ""
    //         }
    //     } else {
    //         ""
    //     };
    //
    //     let text = if let Ok((true, text, boundary)) = separator(text, false) {
    //         // if let Ok((modifier, text)) = identifier(text) {
    //         //
    //         // }
    //         ""
    //     } else {
    //         ""
    //     };
    //
    //     if let Some(unexpected) = text.chars().next() {
    //         return Err(SomeverError::Invalid);
    //         // return Err(Error::new(ErrorKind::UnexpectedCharAfter(pos, unexpected)));
    //     }
    //
    //     // let p = CONVENTIONS.get()
    //
    //     // for year_convention in [FULL_YEAR, SHORT_YEAR, PADDED_YEAR] {
    //     //     if major
    //     // }
    //
    //     // dot or hyphen
    //     // let text = dot(text)?;
    //     // let (minor, text) = numeric_identifier(text)?;
    //
    //     // optional dot or hyphen
    //     // optional micro if numeric
    //
    //     // optional dot or hyphen
    //     // modifier
    //
    //     todo!()
    // }
}

#[derive(Debug)]
struct TokenizedFormat {
    prefix: Option<String>,
    tokens: Vec<FormatSegment>,
    separators: Vec<String>,
}

impl TokenizedFormat {

    fn tokenize(text: &str) -> SomeverResult<Self> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut t = text.to_string();
        let mut pos = Some(Position::Major);
        let mut prefix = None;
        let mut tokens = Vec::new();
        let mut separators = Vec::new();

        if let Some(c) = t.get(0..1) {
            if c.to_lowercase() == "v" {
               prefix = Some(c.to_string());
                t = text.strip_prefix(c).unwrap().to_string();
            }
        }

        while !t.is_empty() && pos.is_some() {
            let current_pos = pos.unwrap();
            let (ident, separator, text) = token(t.as_str())?;
            tokens.push(FormatSegment {
                position: current_pos,
                convention: Conventions::new(ident)?,
            });

            if !separator.is_empty() {
                separators.push(separator.to_string());
            }

            pos = current_pos.next_token_position();
            t = text.to_string();
        }

        if tokens.len() < 2 {
            return Err(SomeverError::Invalid);
        }

        // TODO: more validations

        Ok(Self {
            prefix,
            tokens,
            separators
        })
    }

    fn get_token(&self, position: Position) -> Option<&FormatSegment> {
        for token in &self.tokens {
            if position == token.position {
                return Some(token);
            }
        }

        None
    }

}

#[derive(Debug)]
struct Segments {
    values: Vec<FormatSegment>,
    separators: Vec<String>,
}

impl Segments {
    fn parse(text: &str) -> SomeverResult<Self> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut t = text.to_string();
        let mut pos = Some(Position::Major);
        let mut segments = Vec::new();
        let mut separators = Vec::new();

        while !t.is_empty() && pos.is_some() {
            let current_pos = pos.unwrap();
            let (ident, has_nondigit, separator, text) = identifier(t.as_str())?;
            segments.push(FormatSegment {
                position: current_pos,
                convention: Conventions::new(ident)?,
            });
            separators.push(separator.to_string());
            pos = current_pos.next(true);
            t = text.to_string();
        }

        // while segments.len() < 4 {
        //     // let (identifier, text) = format_identifier(t.as_str())?;
        //     let (ident, separator, text) = identifier(t.as_str())?;
        //     segments.push(ident.to_string());
        //     separators.push(ident.to_string());
        //     t = text.to_string();
        //     if t.is_empty() {
        //         break;
        //     }
        // }

        if segments.len() < 2 {
            return Err(SomeverError::Invalid);
        }

        Ok(Self {
            values: segments,
            separators
        })
    }

    fn numeric_segments(&self) -> usize {
        if self.values.is_empty() {
            return 0;
        }

        let total_segments = self.values.len();
        if self.has_modifier() {
            total_segments - 1
        } else {
            total_segments
        }
    }

    fn has_modifier(&self) -> bool {
        self.values
            .iter()
            .any(|v| matches!(v.position, Position::Modifier))
    }

    // fn has_modifier(&self) -> bool {
    //     self.values.iter().any(|v| matches!(v, Segment::MODIFIER(_)))
    // }
}

impl Default for Segments {
    fn default() -> Self {
        Self {
            values: Vec::new(),
            separators: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct FormatSegment {
    position: Position,
    convention: Conventions,
}

struct TextSegment {
    position: Position,
    value: String,
}

// enum Segment {
//     MAJOR(u16, Conventions),
//     MINOR(u8, Conventions),
//     PATCH(u16, Conventions),
//     MICRO(u16, Conventions),
//     MODIFIER(String),
// }

// impl Segment {
//
//     fn from_str(val: &str, convention: Conventions) -> Self {
//
//     }
//
// }

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

fn separator(input: &str, required: bool) -> SomeverResult<(bool, &str, &str)> {
    if let Some(rest) = input.strip_prefix('.') {
        return Ok((true, rest, "."));
    } else if let Some(rest) = input.strip_prefix('-') {
        return Ok((true, input, "-"));
    } else if let Some(rest) = input.strip_prefix('_') {
        return Ok((true, input, "_"));
    }

    if required {
        Err(SomeverError::Invalid)
    } else {
        Ok((false, input, ""))
    }
}

fn get_separator(input: &str) -> SomeverResult<(&str, &str)> {
    if let Some(rest) = input.strip_prefix('.') {
        Ok((".", rest))
    } else if let Some(rest) = input.strip_prefix('-') {
        Ok(("-", rest))
    } else {
        Err(SomeverError::Invalid)
    }
}

// fn identifier(input: &str) -> SomeverResult<(&str, &str)> {
//     let mut accumulated_len = 0;
//     let mut segment_len = 0;
//
//     loop {
//         match input.as_bytes().get(accumulated_len + segment_len) {
//             Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
//                 segment_len += 1;
//             }
//             Some(b'0'..=b'9') => {
//                 segment_len += 1;
//             }
//             boundary => {
//                 if segment_len == 0 {
//                     return if accumulated_len == 0 && boundary != Some(&b'.') {
//                         Ok(("", input))
//                     } else {
//                         // return Err(Error::new(ErrorKind::EmptySegment(pos)));
//                         Err(SomeverError::Invalid)
//                     };
//                 }
//
//                 accumulated_len += segment_len;
//                 if boundary == Some(&b'.') || boundary == Some(&b'-') {
//                     accumulated_len += 1;
//                     segment_len = 0;
//                 } else {
//                     return Ok(input.split_at(accumulated_len));
//                 }
//             }
//         }
//     }
// }

fn token(input: &str) -> SomeverResult<(&str, &str, &str)> {
    let mut segment_len = 0;
    loop {
        match input.as_bytes().get(segment_len) {
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') | Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            // TODO: should probably be explicit about boundary and everything else is invalid
            // Ex: I don't think we should support /, %, $, #, etc. For values we probably should
            // even limit to essentially digits for most things outside of maybe build?
            boundary => {
                if segment_len == 0 {
                    // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                    return Err(SomeverError::Invalid);
                }

                let (ident, rest) = input.split_at(segment_len);
                if rest.is_empty() {
                    return Ok((ident, "", rest));
                } else {
                    let (sep, rest) = rest.split_at(1);
                    return Ok((ident, sep, rest));
                }
            }
        }
    }
}

fn identifier(input: &str) -> SomeverResult<(&str, bool, &str, &str)> {
    let mut segment_len = 0;
    let mut has_nondigit = false;
    loop {
        match input.as_bytes().get(segment_len) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                segment_len += 1;
                has_nondigit = true;
            }
            // TODO: should probably be explicit about boundary and everything else is invalid
            // Ex: I don't think we should support /, %, $, #, etc. For values we probably should
            // even limit to essentially digits for most things outside of maybe build?
            boundary => {
                if segment_len == 0 {
                    // return Err(Error::new(ErrorKind::EmptySegment(pos)));
                    return Err(SomeverError::Invalid);
                }

                let (ident, rest) = input.split_at(segment_len);
                if rest.is_empty() {
                    return Ok((ident, has_nondigit, "", rest));
                } else {
                    let (sep, rest) = rest.split_at(1);
                    return Ok((ident, has_nondigit, sep, rest));
                }

            }
        }
    }
}


fn format_identifier(input: &str) -> SomeverResult<(&str, &str)> {
    if let Some((text, rest)) = input.split_once(['.', '-', '_']) {
        if text.is_empty() {
            // return Err(Error::new(ErrorKind::EmptySegment(pos)));
            return Err(SomeverError::Invalid);
        }

        Ok((text, rest))
    } else {
        // return Err(Error::new(ErrorKind::EmptySegment(pos)));
        // Err(SomeverError::Invalid)
        Ok((input, ""))
    }

    // let mut segment_len = 0;
    //
    // loop {
    //     match input.as_bytes().get(accumulated_len + segment_len) {
    //         Some(b'A'..=b'Z') | Some(b'a'..=b'z') | Some(b'0'..=b'9') => {
    //             segment_len += 1;
    //         }
    //
    //         // TODO: should probably be explicit about boundary and everything else is invalid
    //         // Ex: I don't think we should support /, %, $, #, etc. For values we probably should
    //         // even limit to essentially digits for most things outside of maybe build?
    //         boundary => {
    //             if segment_len == 0 {
    //                 // return Err(Error::new(ErrorKind::EmptySegment(pos)));
    //                 Err(SomeverError::Invalid)
    //             }
    //
    //             return Ok((input.split_at(accumulated_len), boundary));
    //         }
    //     }
    // }
}

// impl FromStr for Calver {
//     type Err = SomeverError;
//
//     fn from_str(text: &str) -> Result<Self, Self::Err> {
//         if text.is_empty() {
//             return Err(SomeverError::Empty);
//         }
//
//         // not the most performant way of doing this but good enough for now
//         let caps = RE
//             .captures(text)
//             .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
//
//         let major_match = caps
//             .name("major")
//             .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
//
//         let major = major_match
//             .as_str()
//             .parse::<u16>()
//             .map_err(|e| SomeverError::ParseInt(text.to_string()))?;
//
//         let separator = text
//             .chars()
//             .nth(major_match.len())
//             .ok_or(SomeverError::InvalidFormat(text.to_string()))?;
//
//         let minor = caps
//             .name("minor")
//             .ok_or(SomeverError::InvalidFormat(text.to_string()))?
//             .as_str()
//             .parse::<u8>()
//             .map_err(|e| SomeverError::ParseInt(text.to_string()))?;
//
//         let micro = if let Some(micro) = caps.name("micro") {
//             Some(
//                 micro
//                     .as_str()
//                     .parse::<u16>()
//                     .map_err(|e| SomeverError::ParseInt(text.to_string()))?,
//             )
//         } else {
//             None
//         };
//
//         let modifier = caps.name("modifier").map(|m| m.as_str().to_string());
//
//         Ok(Self {
//             prefix: None,
//             major,
//             minor,
//             patch: micro,
//             micro: None,
//             modifier,
//             separator,
//         })
//     }
// }

impl Display for Calver {
    // TODO: might be better to just store raw
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.major, self.separator, self.minor)?;

        if let Some(micro) = &self.patch {
            write!(f, "{}{}", self.separator, micro)?;
        }

        if let Some(modifier) = &self.modifier {
            write!(f, "{}", modifier)?;
        }

        Ok(())
    }
}

#[derive(PartialEq)]
pub(crate) struct FormatConvention {
    pub(crate) representation: &'static str,
    pub(crate) format: Format,
    // pub(crate) format: &'static str,
    // extract - function return u64
    // validate - function return Result<()>
}

#[derive(Debug)]
pub(crate) enum Conventions {
    FULL_YEAR,
    SHORT_YEAR,
    PADDED_YEAR,
    SHORT_MONTH,
    PADDED_MONTH,
    SHORT_WEEK,
    PADDED_WEEK,
    SHORT_DAY,
    PADDED_DAY,
    // support minor and micro (ex: pip and pycharm) where these are just integers
    MINOR,
    MICRO,
}

// https://docs.rs/chrono/latest/chrono/format/strftime/index.html
impl Conventions {
    // /// FullYear notation for CalVer - 2006, 2016, 2106
    // const FULL_YEAR: &str = "YYYY";
    // /// ShortYear notation for CalVer - 6, 16, 106
    // const SHORT_YEAR: &str = "YY";
    // /// PaddedYear notation for CalVer - 06, 16, 106
    // const PADDED_YEAR: &str = "0Y";
    // /// ShortMonth notation for CalVer - 1, 2 ... 11, 12
    // const SHORT_MONTH: &str = "MM";
    // /// PaddedMonth notation for CalVer - 01, 02 ... 11, 12
    // const PADDED_MONTH: &str = "0M";

    pub(crate) fn new(representation: &str) -> SomeverResult<Self> {
        match representation {
            "YYYY" => Ok(Conventions::FULL_YEAR),
            "YY" => Ok(Conventions::SHORT_YEAR),
            "OY" => Ok(Conventions::PADDED_YEAR),
            "MM" => Ok(Conventions::SHORT_MONTH),
            "0M" => Ok(Conventions::PADDED_MONTH),
            "WW" => Ok(Conventions::SHORT_WEEK),
            "0W" => Ok(Conventions::PADDED_WEEK),
            "DD" => Ok(Conventions::SHORT_DAY),
            "0D" => Ok(Conventions::PADDED_DAY),
            "MINOR" => Ok(Conventions::MINOR),
            "MICRO" => Ok(Conventions::MICRO),
            _ => Err(SomeverError::Invalid),
        }
    }

    pub(crate) fn format(&self, value: u64) -> String {
        match self {
            Conventions::FULL_YEAR => format!("{:04}", value),
            Conventions::SHORT_YEAR
            | Conventions::SHORT_MONTH
            | Conventions::SHORT_WEEK
            | Conventions::SHORT_DAY
            | Conventions::MINOR
            | Conventions::MICRO => format!("{}", value),
            Conventions::PADDED_YEAR
            | Conventions::PADDED_MONTH
            | Conventions::PADDED_WEEK
            | Conventions::PADDED_DAY => format!("{:02}", value),
        }
    }

    pub(crate) fn validate(&self, value: u16) -> bool {
        match &self {
            Conventions::FULL_YEAR => validate_in_range(value, 1900, 2500),
            Conventions::SHORT_YEAR => validate_in_range(value, 0, 99),
            Conventions::PADDED_YEAR => validate_in_range(value, 0, 99),
            Conventions::SHORT_MONTH => validate_in_range(value, 1, 12),
            Conventions::PADDED_MONTH => validate_in_range(value, 1, 12),
            Conventions::SHORT_WEEK => validate_in_range(value, 1, 52),
            Conventions::PADDED_WEEK => validate_in_range(value, 1, 52),
            Conventions::SHORT_DAY => validate_in_range(value, 1, 31),
            Conventions::PADDED_DAY => validate_in_range(value, 1, 31),
            Conventions::MINOR | Conventions::MICRO => true,
        }
    }

    pub(crate) fn conv(&self, value: DateTime<Utc>) -> String {
        // ISO week compliant
        // Conventions::FULL_YEAR => format!("{}", value.format("%G")),
        // Conventions::SHORT_YEAR => format!("{}", value.iso_week().year() % 1000)
        // Conventions::PADDED_YEAR => format!("{:02}", value.iso_week().year() % 1000),
        // Conventions::SHORT_WEEK => format!("{}", value.iso_week().week()),
        // Conventions::PADDED_WEEK => format!("{}", value.format("%V")),

        // Jan 1 week compliant
        // Conventions::FULL_YEAR => format!("{}", value.format("%Y")),
        // Conventions::SHORT_YEAR => format!("{}", value.year() % 1000),
        // Conventions::PADDED_YEAR => format!("{:02}", value.year() % 1000),
        // Conventions::SHORT_WEEK => format!("{}", week_starting_jan_1(value)),
        // Conventions::PADDED_WEEK => format!("{:02}", week_starting_jan_1(value)),
        match &self {
            Conventions::FULL_YEAR => format!("{}", value.format("%Y")),
            Conventions::SHORT_YEAR => format!("{}", value.year() % 1000),
            Conventions::PADDED_YEAR => format!("{:02}", value.year() % 1000),
            Conventions::SHORT_MONTH => format!("{}", value.month()),
            Conventions::PADDED_MONTH => format!("{}", value.format("%m")),
            Conventions::SHORT_WEEK => format!("{}", week_starting_jan_1(value)), // format!("{}", value.iso_week().week()),
            Conventions::PADDED_WEEK => format!("{:02}", week_starting_jan_1(value)), // format!("{}", value.format("%V")),
            Conventions::SHORT_DAY => format!("{}", value.day()),
            Conventions::PADDED_DAY => format!("{}", value.format("%d")),
            Conventions::MINOR | Conventions::MICRO => String::from("0"),
        }
    }

    // pub(crate) fn extract(&self, value: DateTime<Utc>) -> u64 {
    //     match &self {
    //         Conventions::FULL_YEAR => {
    //             let year = value.year();
    //         }
    //         Conventions::SHORT_YEAR => {
    //
    //         }
    //         Conventions::PADDED_YEAR => {}
    //         Conventions::SHORT_MONTH => {
    //             let month = value.month();
    //         }
    //         Conventions::PADDED_MONTH => {}
    //     }
    //
    //     0
    // }
}

fn week_starting_jan_1(value: DateTime<Utc>) -> u32 {
    let start_of_year = Utc.with_ymd_and_hms(value.year(), 1, 1, 0, 0, 0).unwrap();

    let days_difference = (value - start_of_year).num_days() as u32;
    days_difference / 7 + 1
}

const YYYY: FormatConvention = FormatConvention {
    representation: "YYYY",
    format: Format::THREE_LEADING_DIGIT, // "%04d",
};

const YY: FormatConvention = FormatConvention {
    representation: "YY",
    format: Format::DIGIT, // "%d",
};

const ZERO_Y: FormatConvention = FormatConvention {
    representation: "0Y",
    format: Format::LEADING_ZERO_DIGIT, // "%02d",
};

const MM: FormatConvention = FormatConvention {
    representation: "MM",
    format: Format::DIGIT, // "%d",
};

const M0: FormatConvention = FormatConvention {
    representation: "M0",
    format: Format::LEADING_ZERO_DIGIT, // "%02d",
};

const ZERO_M: FormatConvention = FormatConvention {
    representation: "0M",
    format: Format::LEADING_ZERO_DIGIT, // "%02d",
};

const WW: FormatConvention = FormatConvention {
    representation: "WW",
    format: Format::DIGIT, // "%d",
};

const ZERO_W: FormatConvention = FormatConvention {
    representation: "0W",
    format: Format::LEADING_ZERO_DIGIT, // "%02d",
};

const DD: FormatConvention = FormatConvention {
    representation: "DD",
    format: Format::DIGIT, // "%d",
};

const D0: FormatConvention = FormatConvention {
    representation: "DD",
    format: Format::DIGIT, // "%d",
};

const ZERO_D: FormatConvention = FormatConvention {
    representation: "0D",
    format: Format::LEADING_ZERO_DIGIT, // "%02d",
};

lazy_static! {
    static ref CONVENTIONS: HashMap<&'static str, FormatConvention> = HashMap::from([
        (YYYY.representation, YYYY),
        (YY.representation, YY),
        (ZERO_Y.representation, ZERO_Y),
        (MM.representation, MM),
        (M0.representation, M0),
        (ZERO_M.representation, ZERO_M),
        (WW.representation, WW),
        (ZERO_W.representation, ZERO_W),
        (DD.representation, DD),
        (D0.representation, D0),
        (ZERO_D.representation, ZERO_D)
    ]);
    static ref YEAR_CONVENTIONS: [FormatConvention; 3] = [YYYY, YY, ZERO_Y];
    static ref MONTH_CONVENTIONS: [FormatConvention; 3] = [MM, M0, ZERO_M];
    static ref WEEK_CONVENTIONS: [FormatConvention; 2] = [WW, ZERO_W];
    static ref DAY_CONVENTIONS: [FormatConvention; 3] = [DD, D0, ZERO_D];
}

// format! macro doesnt allow for dynamic formatting so unfortunately need an enum
#[derive(PartialEq)]
enum Format {
    DIGIT,
    LEADING_ZERO_DIGIT,
    THREE_LEADING_DIGIT,
}

// const CONVENTIONS: HashMap<&str, FormatConvention> = HashMap::from([
//     (YYYY.representation, YYYY),
//     (YY.representation, YY),
//     (ZERO_Y.representation, ZERO_Y),
//     (MM.representation, MM),
//     (M0.representation, M0),
//     (ZERO_M.representation, ZERO_M),
//     (WW.representation, WW),
//     (ZERO_W.representation, ZERO_W),
//     (DD.representation, DD),
//     (D0.representation, D0),
//     (ZERO_D.representation, ZERO_D)
// ]);

impl FormatConvention {
    // fn conventions() {
    //     let YYYY = FormatConvention {
    //         representation: "YYYY",
    //         format: "%04d",
    //     };
    //
    //     let YY = FormatConvention {
    //         representation: "YY",
    //         format: "%d",
    //     };
    //
    //     let zeroY = FormatConvention {
    //         representation: "0Y",
    //         format: "%02d",
    //     };
    //
    //     let MM = FormatConvention {
    //         representation: "MM",
    //         format: "%d",
    //     };
    //
    //     let M0 = FormatConvention {
    //         representation: "M0",
    //         format: "%02d",
    //     };
    //
    //     let zeroM = FormatConvention {
    //         representation: "0M",
    //         format: "%02d",
    //     };
    //
    //     let DD = FormatConvention {
    //         representation: "DD",
    //         format: "%d",
    //     };
    //
    //     let D0 = FormatConvention {
    //         representation: "DD",
    //         format: "%d",
    //     };
    //
    //     let zeroD = FormatConvention {
    //         representation: "0D",
    //         format: "%02d",
    //     };
    // }

    pub(crate) fn format(&self, value: u64) -> String {
        match &self.format {
            Format::DIGIT => format!("{}", value),
            Format::LEADING_ZERO_DIGIT => format!("{:02}", value),
            Format::THREE_LEADING_DIGIT => format!("{:04}", value),
        }
    }

    pub(crate) fn get_year_convention(value: &str) -> SomeverResult<FormatConvention> {
        if value.is_empty() {
            return Err(SomeverError::Empty);
        }

        if value.len() == 4 {
            return Ok(YYYY);
        }

        if value.len() > 0 && value.len() < 4 {
            return if value.starts_with('0') {
                Ok(ZERO_Y)
            } else {
                Ok(YY)
            };
        }

        Err(SomeverError::Invalid)
    }

    pub(crate) fn get_month_convention(value: &str) -> SomeverResult<FormatConvention> {
        if value.is_empty() {
            return Err(SomeverError::Empty);
        }

        if value.len() > 0 && value.len() <= 2 {
            return if value.starts_with('0') {
                Ok(ZERO_M)
            } else {
                Ok(MM)
            };
        }

        Err(SomeverError::Invalid)
    }

    pub(crate) fn get_week_convention(value: &str) -> SomeverResult<FormatConvention> {
        if value.is_empty() {
            return Err(SomeverError::Empty);
        }

        if value.len() > 0 && value.len() <= 2 {
            return if value.starts_with('0') {
                Ok(ZERO_W)
            } else {
                Ok(WW)
            };
        }

        Err(SomeverError::Invalid)
    }

    pub(crate) fn get_day_convention(value: &str) -> SomeverResult<FormatConvention> {
        if value.is_empty() {
            return Err(SomeverError::Empty);
        }

        if value.len() > 0 && value.len() <= 2 {
            return if value.starts_with('0') {
                Ok(ZERO_D)
            } else {
                Ok(DD)
            };
        }

        Err(SomeverError::Invalid)
    }
}

pub(crate) fn validate_in_range(val: u16, min: u16, max: u16) -> bool {
    val >= min && val <= max
}

pub(crate) fn validate_positive(val: u16) -> bool {
    val > 0
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use crate::calendar::{Conventions, Segments};
    use crate::{Calver, Somever, VersioningScheme};

    #[test]
    fn zero_padded_year() {
        let convention = Conventions::FULL_YEAR;
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2006, 1, 1, 0, 0, 0).unwrap())
        );
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2014, 1, 1, 0, 0, 0).unwrap())
        );
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2106, 1, 1, 0, 0, 0).unwrap())
        );
        //
        //
        // let convention = Conventions::PADDED_YEAR;
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2006, 1, 1, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2014, 1, 1, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2106, 1, 1, 0, 0, 0).unwrap()));
        //
        // let convention = Conventions::SHORT_YEAR;
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2006, 1, 1, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2014, 1, 1, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2106, 1, 1, 0, 0, 0).unwrap()));

        // let convention = Conventions::SHORT_WEEK;
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2006, 12, 30, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2014, 12, 30, 0, 0, 0).unwrap()));
        // println!("{}", convention.conv(Utc.with_ymd_and_hms(2106, 12, 30, 0, 0, 0).unwrap()));

        let convention = Conventions::PADDED_WEEK;
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2006, 1, 1, 0, 0, 0).unwrap())
        );
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2014, 1, 1, 0, 0, 0).unwrap())
        );
        println!(
            "{}",
            convention.conv(Utc.with_ymd_and_hms(2106, 1, 1, 0, 0, 0).unwrap())
        );
    }

    // #[test]
    // fn should_parse_and_sort() {
    //     let mut versions = vec![];
    //     for f in [
    //         "2024.01.28",
    //         "2024.01.28-final",
    //         "2024.01.28.final",
    //         "2024.01",
    //         "2024.01-suffix",
    //         "2024.01.suffix",
    //         "2024.1",
    //         "2024.1-suffix",
    //         "2024.1.suffix",
    //
    //          YYY-MM-DD
    //         "2024-06-28",
    //         "2024-06-28-final",
    //         "2024-06-28.final",
    //         "2024-06",
    //         "2024-06-suffix",
    //         "2024-06.suffix",
    //         "2024-6",
    //         "2024-6-suffix",
    //         "2024-6.suffix",
    //
    //         "24.01.28",
    //         "24.01.28-final",
    //         "06.01.28", // what format is this?
    //
    //         "06.52.01", // what format is this?
    //     ] {
    //         versions.push(Calver::parse(f).unwrap());
    //     }
    //
    //     versions.sort();
    //
    //     assert_eq!(
    //         vec![
    //             "6.1.28",
    //             "6.52.1",
    //             "24.1",
    //             "24.1.28",
    //             "24.1.28-final",
    //             "2024.1-suffix",
    //             "2024.1.suffix",
    //             "2024.1.28",
    //             "2024.1.28-final",
    //             "2024.1.28-suffix",
    //             "2024-6-28",
    //         ],
    //         versions
    //             .iter()
    //             .map(|v| v.to_string())
    //             .collect::<Vec<String>>()
    //     );
    // }

    #[test]
    fn test_parse_format() {
        println!("{:?}", Calver::parse_format_pattern("YYYY.MM.DD"));
        println!("{:?}", Calver::parse_format_pattern("YYYY-MM"));
        println!("{:?}", Calver::parse_format_pattern("YY.MINOR.MICRO"));
        println!("{:?}", Calver::parse_format_pattern("YYYY.0M"));
        println!("{:?}", Calver::parse_format_pattern("YYYY.MM.DD_MICRO"));
        println!("{:?}", Calver::parse_format_pattern("BLAH"));
        println!("{:?}", Calver::parse_format_pattern("YY.FOO"));
        println!("{:?}", Calver::parse_format_pattern("YY.2W"));
        println!("{:?}", Calver::parse_format_pattern("YY.MM.DD.MICRO-MINOR"));
    }

    #[test]
    fn test_parse() {
        println!("{:?}", Calver::parse("2024.2.1-rc1", "YY.MINOR.MICRO"));
        println!("{:?}", Calver::parse("2024.10.13", "YYYY.MM.DD"));
        println!("{:?}", Calver::parse("2024-10", "YYYY-MM"));
        println!("{:?}", Calver::parse("2024-10-alpha", "YYYY-MM"));
        println!("{:?}", Calver::parse("2024.2.1", "YY.MINOR.MICRO"));

        println!("{:?}", Calver::parse("2024.02", "YYYY.0M"));
        println!("{:?}", Calver::parse("2024.10.13_1", "YYYY.MM.DD_MICRO"));
        println!("{:?}", Calver::parse("2024.10.13", "BLAH"));
        println!("{:?}", Calver::parse("24.10", "YY.FOO"));
        println!("{:?}", Calver::parse("24.21", "YY.2W"));
        println!(
            "{:?}",
            Calver::parse("2024-10-13-1-2", "YY.MM.DD.MICRO-MINOR")
        );
    }

    #[test]
    fn test_new() {
        println!("{:?}", Calver::new("YYYY.MM.DD".to_string(), None));
        println!("{:?}", Calver::new("YYYY-MM".to_string(), None));
        println!("{:?}", Calver::new("YY.MINOR.MICRO".to_string(), None));
        println!("{:?}", Calver::new("YYYY.0M".to_string(), None));
        println!("{:?}", Calver::new("YYYY.MM.DD_MICRO".to_string(), None));
    }

    #[test]
    fn test_segments_parse() {
        println!("{:?}", Segments::parse("YYYY.MM.DD").unwrap());
        println!("{:?}", Segments::parse("YYYY.MM.DD").unwrap());
    }
}
