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

use chrono::{Datelike, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{SomeverError, SomeverResult};

const DOT: &'static str = ".";
const HYPHEN: &'static str = "-";

// TODO: Good validations - https://github.com/k1LoW/calver/blob/main/token.go#L182

#[remain::sorted]
#[derive(Debug, Error, PartialEq)]
pub enum CalverError {
    #[error("Mismatch separator. {0} provided but format defined {1}")]
    MismatchSeparator(String, String),
}

pub struct CalverModifier {
    pub value: String,
    pub separator: String,
}

impl CalverModifier {
    pub fn new(value: String) -> Self {
        Self::new_with_separator(value, HYPHEN.to_string())
    }

    pub fn new_with_separator(value: String, separator: String) -> Self {
        Self { value, separator }
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Calver {
    pub prefixed: bool,
    pub major: u16,
    pub minor: u8,
    pub patch: Option<u16>,
    // generally discouraged by Calver however potentially useful in rare scenarios such as
    // fusefs-ntfs which uses a format of YYYY.MM.DD_MICRO
    pub micro: Option<u16>,
    pub modifier: Option<String>,
    pub format: TokenizedFormat,
}

#[derive(Debug, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum VersionToken {
    Prefix,
    Major(u16, Conventions),
    Minor(u8, Conventions),
    Patch(u16, Conventions),
    Micro(u16, Conventions),
    Modifier(String),
    Separator(String),
}

impl Display for VersionToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionToken::Prefix => write!(f, "v"),
            VersionToken::Major(v, c) => write!(f, "{}", c.format(*v)),
            VersionToken::Minor(v, c) => write!(f, "{}", c.format(*v as u16)),
            VersionToken::Patch(v, c) => write!(f, "{}", c.format(*v)),
            VersionToken::Micro(v, c) => write!(f, "{}", c.format(*v)),
            VersionToken::Modifier(v) => write!(f, "{}", v),
            VersionToken::Separator(v) => write!(f, "{}", v),
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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
            _ => None,
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
    pub fn new(format: String, modifier: Option<CalverModifier>) -> SomeverResult<Self> {
        Self::internal_new(Utc::now().date_naive(), false, format, modifier)
    }

    pub fn new_prefixed(format: String, modifier: Option<CalverModifier>) -> SomeverResult<Self> {
        Self::internal_new(Utc::now().date_naive(), true, format, modifier)
    }

    pub(crate) fn internal_new(
        date: NaiveDate,
        prefixed: bool,
        format: String,
        modifier: Option<CalverModifier>,
    ) -> SomeverResult<Self> {
        let mut tokenized = TokenizedFormat::tokenize(&format)?;

        let major = tokenized
            .get_token(Position::Major)
            .ok_or(SomeverError::Invalid)?
            .convention
            .conv(date);
        let minor = tokenized
            .get_token(Position::Minor)
            .ok_or(SomeverError::Invalid)?
            .convention
            .conv(date);
        let patch = tokenized
            .get_token(Position::Patch)
            .and_then(|s| Some(s.convention.conv(date)));
        let micro = tokenized
            .get_token(Position::Micro)
            .and_then(|s| Some(s.convention.conv(date)));

        let mut modifier_str = None;
        if let Some(modifier) = modifier {
            tokenized.separators.push(modifier.separator);
            modifier_str = Some(modifier.value);
        }

        Ok(Self {
            prefixed,
            major: major.parse::<u16>()?,
            minor: minor.parse::<u8>()?,
            patch: patch.map(|s| s.parse::<u16>()).transpose()?,
            micro: micro.map(|s| s.parse::<u16>()).transpose()?,
            modifier: modifier_str,
            format: tokenized,
        })
    }

    pub fn parse(text: &str, format: &str) -> SomeverResult<Self> {
        let mut tokenized = TokenizedFormat::tokenize(&format)?;
        let (prefixed, text_segments, text_separators) = Calver::parse_text(&text)?;

        // TODO: handle validation when modifier segment is provided
        // if format_segments.len() != text_segments.len() {
        //     // TODO: number of segments on value does not match pattern
        //     return Err(SomeverError::Invalid);
        // }

        // TODO: validate format/separator
        // Text separators may have more separators than tokenized as there may be one for the modifier
        if !tokenized
            .separators
            .eq(&text_separators[..tokenized.separators.len()])
        {
            return Err(SomeverError::Invalid);
        }

        let mut major = 0;
        let mut minor = 0;
        let mut patch = None;
        let mut micro = None;
        let mut modifier = None;

        for (pos, segment) in tokenized.tokens.iter().enumerate() {
            // TODO: better error
            let v = text_segments
                .get(pos)
                .ok_or(SomeverError::Invalid)?
                .parse::<u16>()?;

            if !segment.convention.validate(v) {
                // TODO: better error
                return Err(SomeverError::Invalid);
            }

            match segment.position {
                Position::Major => {
                    major = v;
                }
                Position::Minor => {
                    minor = u8::try_from(v)?;
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
        // The provided text has a modifier defined that isn't specified in the format
        if segment_difference == 1 {
            if let Some(modifier_text) = text_segments.get(tokenized.tokens.len()) {
                modifier = Some(modifier_text.to_string());
                let modifier_separator = text_separators
                    //.get(text_segments.len() - 1)
                    .last()
                    .unwrap_or(&"-".to_string())
                    .to_string();
                tokenized.separators.push(modifier_separator.clone());
            }
        } else if segment_difference > 1 {
            // TODO: better error
            return Err(SomeverError::Invalid);
        }

        Ok(Self {
            prefixed,
            major,
            minor,
            patch,
            micro,
            modifier,
            format: tokenized,
        })
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
        while !t.is_empty() && segments.len() < 4 {
            let (ident, separator, text) = format_token(t.as_str())?;
            segments.push(Conventions::new(ident)?);
            separators.push(separator.to_string());
            t = text.to_string();
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

    pub(crate) fn parse_text(input: &str) -> SomeverResult<(bool, Vec<String>, Vec<String>)> {
        if input.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut text = input.clone();
        let mut prefixed = false;
        let mut segments = Vec::new();
        let mut separators = Vec::new();

        if let Some(c) = text.get(0..1) {
            if c.to_lowercase() == "v" {
                prefixed = true;
                text = text.strip_prefix(c).unwrap();
            }
        }

        while !text.is_empty() {
            let (ident, separator, rest) = identifier(text)?;
            segments.push(ident.to_string());

            if !separator.is_empty() {
                separators.push(separator.to_string());
            }

            text = rest;
        }

        if segments.len() < 2 || segments.len() > 5 {
            return Err(SomeverError::Invalid);
        }

        Ok((prefixed, segments, separators))
    }


    // fn get_date_format_segments(self) -> Vec<String> {
    //     let mut date_segments = Vec::new();
    //     for s in self.format.tokens {
    //         if s.convention.is_date_related() {
    //             date_segments.push(self.)
    //         }
    //     }
    //
    //     date_segments
    // }


    // TODO: do we want a prerelease / release?
    //  - both of these could have an optional modifier


    pub fn bump_minor(self) -> SomeverResult<Self> {
        // TODO: fail if MINOR format not provided
        todo!()
    }

    pub fn bump_micro(self) -> SomeverResult<Self> {
        // TODO: fail if MINOR format not provided
        todo!()
    }

    pub fn bump_prerelease(self, modifier: CalverModifier) -> SomeverResult<Self> {
        todo!()
    }

    pub fn bump(self) -> SomeverResult<Self> {
        // TODO: if current date matches Calver date we want to bump minor/micro if present
        //  - what to do if both minor and micro are both present? Would need a way to specify which
        //  - how do we want to do comparison? We dont want to include minor/micro in comparison

        let d = Utc::now().date_naive();

        // TODO: make modifier easier to grab
        let modifier = if let Some(modifier) = self.modifier {
            Some(CalverModifier::new_with_separator(modifier, self.format.separators.last().unwrap_or(&HYPHEN.into()).to_string()))
        } else {
            None
        };

        if self.prefixed {
            Self::new_prefixed(self.format.raw, modifier)
        } else {
            Self::new(self.format.raw, modifier)
        }
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct TokenizedFormat {
    raw: String,
    prefix: Option<String>,
    tokens: Vec<FormatSegment>,
    separators: Vec<String>,
}

impl TokenizedFormat {
    fn new(
        raw: String,
        prefix: Option<String>,
        tokens: Vec<FormatSegment>,
        separators: Vec<String>,
    ) -> Self {
        Self {
            raw,
            prefix,
            tokens,
            separators,
        }
    }

    fn tokenize(text: &str) -> SomeverResult<Self> {
        if text.is_empty() {
            return Err(SomeverError::Empty);
        }

        let mut t = text.to_string();
        let mut pos = Some(Position::Major);
        let mut prefix = None;
        let mut tokens = Vec::new();
        let mut separators = Vec::new();

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
        // TODO: validate - "0Y.WW.DD" should result in error as it doesnt make sense

        TokenizedFormat::validate_order(&tokens)?;

        Ok(Self {
            raw: text.to_string(),
            prefix,
            tokens,
            separators,
        })
    }

    fn validate_order(tokens: &Vec<FormatSegment>) -> SomeverResult<()> {
        let mut previous_ordinal = 0;
        for token in tokens {
            if token.convention.ordinal() < previous_ordinal {
                // TODO: better error message
                return Err(SomeverError::InvalidFormat(format!(
                    "{} in the wrong position",
                    token.convention.representation()
                )));
            }
            previous_ordinal = token.convention.ordinal();
        }

        Ok(())
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

// FormatToken vs FormatTokens vs FormatSegment vs FormatSegments
// need both format tokens vs version tokens
// #[remain::sorted]
// #[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
// pub enum FormatToken {
//     Segment(FormatSegment),
//     Separator(String),
// }
//
// #[derive(Debug)]
// struct Segments {
//     tokens: Vec<Token>,
//     // values: Vec<FormatSegment>,
//     // separators: Vec<String>,
// }

// impl Segments {
//     fn parse(text: &str) -> SomeverResult<Self> {
//         if text.is_empty() {
//             return Err(SomeverError::Empty);
//         }
//
//         let mut t = text.to_string();
//         let mut pos = Some(Position::Major);
//         let mut segments = Vec::new();
//         let mut separators = Vec::new();
//
//         while !t.is_empty() && pos.is_some() {
//             let current_pos = pos.unwrap();
//             let (ident, has_nondigit, separator, text) = identifier(t.as_str())?;
//             segments.push(FormatSegment {
//                 position: current_pos,
//                 convention: Conventions::new(ident)?,
//             });
//             separators.push(separator.to_string());
//             pos = current_pos.next(true);
//             t = text.to_string();
//         }
//
//         if segments.len() < 2 {
//             return Err(SomeverError::Invalid);
//         }
//
//         Ok(Self {
//             values: segments,
//             separators
//         })
//     }
//
//     fn numeric_segments(&self) -> usize {
//         if self.values.is_empty() {
//             return 0;
//         }
//
//         let total_segments = self.values.len();
//         if self.has_modifier() {
//             total_segments - 1
//         } else {
//             total_segments
//         }
//     }
//
//     fn has_modifier(&self) -> bool {
//         self.values
//             .iter()
//             .any(|v| matches!(v.position, Position::Modifier))
//     }
//
// }
//
// impl Default for Segments {
//     fn default() -> Self {
//         Self {
//             values: Vec::new(),
//             separators: Vec::new(),
//         }
//     }
// }

// TODO: FormatTokens better name?
#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
struct FormatSegment {
    position: Position,
    // TODO: Should this be Optional to support Modifier?
    convention: Conventions,
}

impl FormatSegment {
    fn new(position: Position, convention: Conventions) -> Self {
        Self {
            position,
            convention,
        }
    }
}

struct TextSegment {
    position: Position,
    value: String,
}

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

fn format_token(input: &str) -> SomeverResult<(&str, &str, &str)> {
    let mut segment_len = 0;
    loop {
        match input.as_bytes().get(segment_len) {
            Some(b'0'..=b'9') | Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
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

fn identifier(input: &str) -> SomeverResult<(&str, &str, &str)> {
    let mut segment_len = 0;
    loop {
        match input.as_bytes().get(segment_len) {
            Some(b'0'..=b'9') => {
                segment_len += 1;
            }
            Some(b'A'..=b'Z') | Some(b'a'..=b'z') => {
                segment_len += 1;
                // assume this is the modifier portion
                return Ok((input, "", ""));
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

impl Display for Calver {

    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.prefixed {
            write!(f, "v")?;
        }

        for (index, token) in self.format.tokens.iter().enumerate() {
            match token.position {
                Position::Major => write!(f, "{}", token.convention.format(self.major))?,
                Position::Minor => write!(f, "{}", token.convention.format(self.minor as u16))?,
                Position::Patch => {
                    if let Some(patch) = self.patch {
                        write!(f, "{}", token.convention.format(patch))?;
                    }
                }
                Position::Micro => {
                    if let Some(micro) = self.micro {
                        write!(f, "{}", token.convention.format(micro))?;
                    }
                }
                Position::Modifier => {
                    if let Some(modifier) = &self.modifier {
                        write!(f, "{}", modifier)?
                    }
                }
            }

            if let Some(sep) = self.format.separators.get(index) {
                write!(f, "{}", sep)?;
            }
        }

        if let Some(modifier) = &self.modifier {
            write!(f, "{}", modifier)?
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize, Clone)]
pub(crate) enum Conventions {
    /// Full year notation for CalVer - 2006, 2016, 2106
    FullYear,
    /// Short year notation for CalVer - 6, 16, 106
    ShortYear,
    /// Padded year notation for CalVer - 06, 16, 106
    PaddedYear,
    /// Short month notation for CalVer - 1, 2 ... 11, 12
    ShortMonth,
    /// Padded month notation for CalVer - 01, 02 ... 11, 12
    PaddedMonth,
    /// Short week notation for CalVer - 1, 2, 33, 52
    ShortWeek,
    /// Padded week notation for CalVer - 01, 02, 33, 52
    PaddedWeek,
    /// Short day notation for CalVer - 1, 2 ... 30, 31
    ShortDay,
    /// Padded day notation for CalVer - 01, 02 ... 30, 31
    PaddedDay,
    // support minor and micro (ex: pip and pycharm) where these are just integers
    /// Minor notation 0 - 65,536 which is incrementing
    Minor,
    /// Micro notation 0 - 65,536 which is incrementing
    Micro,
}

// https://docs.rs/chrono/latest/chrono/format/strftime/index.html
impl Conventions {
    pub(crate) fn new(representation: &str) -> SomeverResult<Self> {
        match representation {
            "YYYY" => Ok(Conventions::FullYear),
            "YY" => Ok(Conventions::ShortYear),
            "0Y" => Ok(Conventions::PaddedYear),
            "MM" => Ok(Conventions::ShortMonth),
            "0M" => Ok(Conventions::PaddedMonth),
            "WW" => Ok(Conventions::ShortWeek),
            "0W" => Ok(Conventions::PaddedWeek),
            "DD" => Ok(Conventions::ShortDay),
            "0D" => Ok(Conventions::PaddedDay),
            "MINOR" => Ok(Conventions::Minor),
            "MICRO" => Ok(Conventions::Micro),
            _ => Err(SomeverError::Invalid),
        }
    }

    pub(crate) fn format(&self, value: u16) -> String {
        match self {
            Conventions::FullYear => format!("{:04}", value),
            Conventions::ShortYear
            | Conventions::ShortMonth
            | Conventions::ShortWeek
            | Conventions::ShortDay
            | Conventions::Minor
            | Conventions::Micro => format!("{}", value),
            Conventions::PaddedYear
            | Conventions::PaddedMonth
            | Conventions::PaddedWeek
            | Conventions::PaddedDay => format!("{:02}", value),
        }
    }

    pub(crate) fn representation(&self) -> &'static str {
        match &self {
            Conventions::FullYear => "YYYY",
            Conventions::ShortYear => "YY",
            Conventions::PaddedYear => "0Y",
            Conventions::ShortMonth => "MM",
            Conventions::PaddedMonth => "0M",
            Conventions::ShortWeek => "WW",
            Conventions::PaddedWeek => "0W",
            Conventions::ShortDay => "DD",
            Conventions::PaddedDay => "0D",
            Conventions::Minor => "MINOR",
            Conventions::Micro => "MICRO",
        }
    }

    pub(crate) fn validate(&self, value: u16) -> bool {
        match &self {
            Conventions::FullYear => validate_in_range(value, 1900, 2500),
            Conventions::ShortYear => validate_in_range(value, 0, 99),
            Conventions::PaddedYear => validate_in_range(value, 0, 99),
            Conventions::ShortMonth => validate_in_range(value, 1, 12),
            Conventions::PaddedMonth => validate_in_range(value, 1, 12),
            Conventions::ShortWeek => validate_in_range(value, 1, 52),
            Conventions::PaddedWeek => validate_in_range(value, 1, 52),
            Conventions::ShortDay => validate_in_range(value, 1, 31),
            Conventions::PaddedDay => validate_in_range(value, 1, 31),
            Conventions::Minor | Conventions::Micro => true,
        }
    }

    pub(crate) fn conv(&self, value: NaiveDate) -> String {
        // TODO: do we want to be ISO compliant or based values off of Jan 1st being week 1?
        // ISO week compliant
        // Conventions::FullYear => format!("{}", value.format("%G")),
        // Conventions::ShortYear => format!("{}", value.iso_week().year() % 1000)
        // Conventions::PaddedYear => format!("{:02}", value.iso_week().year() % 1000),
        // Conventions::ShortWeek => format!("{}", value.iso_week().week()),
        // Conventions::PaddedWeek => format!("{}", value.format("%V")),

        // Jan 1 week compliant
        // Conventions::FullYear => format!("{}", value.format("%Y")),
        // Conventions::ShortYear => format!("{}", value.year() % 1000),
        // Conventions::PaddedYear => format!("{:02}", value.year() % 1000),
        // Conventions::ShortWeek => format!("{}", week_starting_jan_1(value)),
        // Conventions::PaddedWeek => format!("{:02}", week_starting_jan_1(value)),
        match &self {
            Conventions::FullYear => format!("{}", value.format("%Y")),
            Conventions::ShortYear => format!("{}", value.year() % 1000),
            Conventions::PaddedYear => format!("{:02}", value.year() % 1000),
            Conventions::ShortMonth => format!("{}", value.month()),
            Conventions::PaddedMonth => format!("{}", value.format("%m")),
            Conventions::ShortWeek => format!("{}", week_starting_jan_1(value)),
            Conventions::PaddedWeek => format!("{:02}", week_starting_jan_1(value)),
            Conventions::ShortDay => format!("{}", value.day()),
            Conventions::PaddedDay => format!("{}", value.format("%d")),
            Conventions::Minor | Conventions::Micro => String::from("0"),
        }
    }

    /// Returns an ordinal value which is primarily used in verifying correct order of conventions
    /// within a specific format.
    fn ordinal(&self) -> u8 {
        match self {
            Conventions::FullYear | Conventions::ShortYear | Conventions::PaddedYear => 1,
            Conventions::ShortMonth | Conventions::PaddedMonth => 2,
            Conventions::ShortWeek | Conventions::PaddedWeek => 3,
            Conventions::ShortDay | Conventions::PaddedDay => 4,
            Conventions::Minor | Conventions::Micro => 5,
        }
    }

    fn is_date_related(&self) -> bool {
        match self {
            Conventions::FullYear
            | Conventions::ShortYear
            | Conventions::ShortMonth
            | Conventions::ShortWeek
            | Conventions::ShortDay
            | Conventions::PaddedYear
            | Conventions::PaddedMonth
            | Conventions::PaddedWeek
            | Conventions::PaddedDay => true,
            Conventions::Minor
            | Conventions::Micro => false
        }
    }
}

fn week_starting_jan_1(value: NaiveDate) -> u32 {
    let start_of_year = NaiveDate::from_ymd_opt(value.year(), 1, 1).unwrap();
    let days_difference = (value - start_of_year).num_days() as u32;
    days_difference / 7 + 1
}

pub(crate) fn validate_in_range(val: u16, min: u16, max: u16) -> bool {
    val >= min && val <= max
}

pub(crate) fn validate_positive(val: u16) -> bool {
    val > 0
}

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
    use test_case::test_case;
    use testing::set_snapshot_suffix;

    use super::{CalverModifier, Conventions, FormatSegment, Position, TokenizedFormat};
    use crate::semantic::Semver;
    use crate::{Calver, SomeverError, SomeverResult};

    #[test_case("2024.10.13", "YYYY.MM.DD")]
    #[test_case("2024.10.6", "YYYY.MM.DD")]
    #[test_case("2024.10.6", "YYYY.MM.0D")]
    #[test_case("2024.2.1-rc1", "YYYY.MINOR.MICRO")]
    #[test_case("2024-10", "YYYY-MM")]
    #[test_case("2024-09-alpha", "YYYY-MM")]
    #[test_case("2024-09", "YYYY-0M")]
    #[test_case("2024-46", "YYYY-WW")]
    #[test_case("2024-05", "YYYY-0W")]
    #[test_case("4-09", "YY-MM")]
    #[test_case("04.10.13", "0Y.MM.DD")]
    #[test_case("24.2.1", "YY.MINOR.MICRO")]
    #[test_case("2024.1.suffix", "YYYY.MM")]
    #[test_case("2024.10.13_1", "YYYY.MM.DD_MICRO")]
    #[test_case("v2024.10.13_1", "YYYY.MM.DD_MICRO")]
    fn should_successfully_parse(text: &str, format: &str) {
        set_snapshot_suffix!("{}_{}", text, format);

        insta::assert_snapshot!(
            serde_json::to_string(&Calver::parse(text, format).unwrap()).unwrap()
        );
    }

    #[test_case("2024-11-16", "YYYY.MM.DD", false, None)]
    #[test_case("2024-11-6", "YYYY.MM.DD", false, None)]
    #[test_case("2024-11-6", "YYYY.MM.0D", false, None)]
    #[test_case("2024-09-16", "YYYY-MM-MINOR", false, None)]
    #[test_case("2024-09-16", "YYYY.0M", false, None)]
    #[test_case("2024-11-16", "YYYY.WW", false, None)]
    #[test_case("2024-02-01", "YYYY.0W", false, None)]
    #[test_case("2024-09-16", "YY-MM", false, None)]
    #[test_case("2004-09-16", "YY-MM", false, None)]
    #[test_case("2004-11-16", "0Y.MM.DD", false, None)]
    #[test_case("2024-11-16", "YYYY.MINOR.MICRO", false, None)]
    #[test_case("2024-11-16", "YYYY.MM.DD_MICRO", false, None)]
    #[test_case("2024-11-16", "YYYY.MM", true, None)]
    #[test_case("2024-11-16", "YYYY.MM", false, Some("suffix"))]
    #[test_case("2024-11-16", "YYYY.MM", true, Some("suffix"))]
    fn should_successfully_create(
        date_str: &str,
        format: &str,
        prefixed: bool,
        modifier_str: Option<&str>,
    ) {
        set_snapshot_suffix!("{}_{}_{}_{:?}", date_str, format, prefixed, modifier_str);

        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let modifier = if let Some(modifier) = modifier_str {
            Some(CalverModifier::new(modifier.to_string()))
        } else {
            None
        };

        insta::assert_snapshot!(serde_json::to_string(
            &Calver::internal_new(date, prefixed, format.to_string(), modifier,).unwrap()
        )
        .unwrap());
    }

    #[test_case("2024-11-16", "YYYY.MM.DD", false, None, "2024.11.16")]
    #[test_case("2024-11-6", "YYYY.MM.DD", false, None, "2024.11.6")]
    #[test_case("2024-11-6", "YYYY.MM.0D", false, None, "2024.11.06")]
    #[test_case("2024-09-16", "YYYY-MM-MINOR", false, None, "2024-9-0")]
    #[test_case("2024-09-16", "YY-MM", false, None, "24-9")]
    #[test_case("2004-09-16", "YY-MM", false, None, "4-9")]
    #[test_case("2004-11-16", "0Y.MM.DD", false, None, "04.11.16")]
    #[test_case("2024-09-16", "YYYY.0M", false, None, "2024.09")]
    #[test_case("2024-11-16", "YYYY.MM.DD_MICRO", false, None, "2024.11.16_0")]
    #[test_case("2024-11-16", "YYYY.WW", false, None, "2024.46")]
    #[test_case("2024-02-01", "YYYY.0W", false, None, "2024.05")]
    #[test_case("2024-11-16", "YYYY.MM", true, None, "v2024.11")]
    #[test_case("2024-11-16", "YYYY.MM", false, Some("suffix"), "2024.11-suffix")]
    #[test_case("2024-11-16", "YYYY.MM", true, Some("suffix"), "v2024.11-suffix")]
    fn should_correctly_format(
        date_str: &str,
        format: &str,
        prefixed: bool,
        modifier_str: Option<&str>,
        expected: &str,
    ) {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let modifier = if let Some(modifier) = modifier_str {
            Some(CalverModifier::new(modifier.to_string()))
        } else {
            None
        };

        let calver = Calver::internal_new(date, prefixed, format.to_string(), modifier).unwrap();

        assert_eq!(calver.to_string(), expected);
    }

    #[test_case("2024-02-01", "Y.MM")]
    #[test_case("2024-02-01", "Y.M")]
    #[test_case("2024-02-01", "YYYY-MM-D")]
    #[test_case("2024-02-01", "YYYY-MM-FOO")]
    #[test_case("2024-02-01", "FOO-MM")]
    fn should_fail_parse_with_invalid_formats(text: &str, format: &str) {
        let result = Calver::parse(text, format);
        assert_eq!(SomeverResult::Err(SomeverError::Invalid), result);
    }

    #[test_case("2024-02-01", "Y.MM")]
    #[test_case("2024-02-01", "Y.M")]
    #[test_case("2024-02-01", "YYYY-MM-D")]
    #[test_case("2024-02-01", "YYYY-MM-FOO")]
    #[test_case("2024-02-01", "FOO-MM")]
    fn should_fail_create_with_invalid_format(date_str: &str, format: &str) {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let result = Calver::internal_new(date, false, format.to_string(), None);

        assert_eq!(SomeverResult::Err(SomeverError::Invalid), result);
    }

    #[test_case("2024.10.13", "YYYY.DD.MM", "MM")]
    #[test_case("2024.10.13", "YYYY.MICRO.MM", "MM")]
    #[test_case("2024.10.13", "YYYY.MINOR.0M", "0M")]
    #[test_case("2024.10.13", "YYYY.MICRO.DD", "DD")]
    #[test_case("2024.10.13", "YYYY.MINOR.0D", "0D")]
    #[test_case("2024.10.13", "MM.YYYY", "YYYY")]
    #[test_case("2024.10.13", "MM.YY", "YY")]
    #[test_case("2024.10.13", "MM.0Y", "0Y")]
    #[test_case("2024.10.13", "DD.YYYY", "YYYY")]
    #[test_case("2024.10.13", "0D.YY", "YY")]
    #[test_case("2024.10.13", "DD.0Y", "0Y")]
    #[test_case("2024.10.13", "0D.MM", "MM")]
    #[test_case("2024.10.13", "DD.0M", "0M")]
    #[test_case("2024.10.13", "MINOR.YYYY", "YYYY")]
    #[test_case("2024.10.13", "MICRO.YYYY", "YYYY")]
    #[test_case("2024.10.13", "MINOR.MM", "MM")]
    #[test_case("2024.10.13", "MICRO.0M", "0M")]
    #[test_case("2024.10.13", "MINOR.DD", "DD")]
    #[test_case("2024.10.13", "MICRO.0D", "0D")]
    fn should_fail_parse_with_invalid_format_order(text: &str, format: &str, wrong_position: &str) {
        let result = Calver::parse(text, format);
        assert_eq!(
            SomeverResult::Err(SomeverError::InvalidFormat(format!(
                "{wrong_position} in the wrong position"
            ))),
            result
        );
    }

    #[test_case("2024-10-13", "YYYY.DD.MM", "MM")]
    #[test_case("2024-10-13", "YYYY.MICRO.MM", "MM")]
    #[test_case("2024-10-13", "YYYY.MINOR.0M", "0M")]
    #[test_case("2024-10-13", "YYYY.MICRO.DD", "DD")]
    #[test_case("2024-10-13", "YYYY.MINOR.0D", "0D")]
    #[test_case("2024-10-13", "MM.YYYY", "YYYY")]
    #[test_case("2024-10-13", "MM.YY", "YY")]
    #[test_case("2024-10-13", "MM.0Y", "0Y")]
    #[test_case("2024-10-13", "DD.YYYY", "YYYY")]
    #[test_case("2024-10-13", "0D.YY", "YY")]
    #[test_case("2024-10-13", "DD.0Y", "0Y")]
    #[test_case("2024-10-13", "0D.MM", "MM")]
    #[test_case("2024-10-13", "DD.0M", "0M")]
    #[test_case("2024-10-13", "MINOR.YYYY", "YYYY")]
    #[test_case("2024-10-13", "MICRO.YYYY", "YYYY")]
    #[test_case("2024-10-13", "MINOR.MM", "MM")]
    #[test_case("2024-10-13", "MICRO.0M", "0M")]
    #[test_case("2024-10-13", "MINOR.DD", "DD")]
    #[test_case("2024-10-13", "MICRO.0D", "0D")]
    fn should_fail_create_with_invalid_format_order(
        date_str: &str,
        format: &str,
        wrong_position: &str,
    ) {
        let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
        let result = Calver::internal_new(date, false, format.to_string(), None);
        assert_eq!(
            SomeverResult::Err(SomeverError::InvalidFormat(format!(
                "{wrong_position} in the wrong position"
            ))),
            result
        );
    }

    #[test_case("2024.05", "YYYY-MM")]
    #[test_case("2024.05.1", "YYYY-MM_MINOR")]
    #[test_case("2024-11", "YYYY.MM")]
    #[test_case("2024.11-1", "YYYY.MM_MICRO")]
    fn should_fail_parse_with_mismatched_separators(text: &str, format: &str) {
        let result = Calver::parse(text, format);
        assert_eq!(SomeverResult::Err(SomeverError::Invalid), result);
    }

    #[test]
    fn should_correctly_sort() {
        let mut versions = vec![];
        for (text, format) in [
            ("2024.01.28", "YYYY.0M.DD"),
            ("2024.01.28-final", "YYYY.0M.DD"),
            ("2024.01.28.final", "YYYY.0M.DD"),
            ("2024.01", "YYYY.0M"),
            ("2024.01-suffix", "YYYY.0M"),
            ("2024.01.suffix", "YYYY.0M"),
            ("2024.1", "YYYY.MM"),
            ("2024.1-suffix", "YYYY.MM"),
            ("2024.1.suffix", "YYYY.MM"),
            ("2024.1.28.final", "YYYY.MM.DD"),
            ("2024.1.28-suffix", "YYYY.MM.DD"),
            ("2024.1.28-final", "YYYY.MM.DD"),
            ("2024.1.28", "YYYY.MM.DD"),
            ("2024-06-28", "YYYY-0M-DD"),
            ("2024-06-28-final", "YYYY-0M-DD"),
            ("2024-06-28.final", "YYYY-0M-DD"),
            ("2024-06", "YYYY-0M"),
            ("2024-06-suffix", "YYYY-0M"),
            ("2024-06.suffix", "YYYY-0M"),
            ("2024-6", "YYYY-MM"),
            ("2024-6-suffix", "YYYY-MM"),
            ("2024-6.suffix", "YYYY-MM"),
            ("2024-06-28", "YYYY-MM-DD"),
            ("24.01.28", "YY.MM.DD"),
            ("24.01", "YY.0M"),
            ("24.01.28-final", "YY.MM.DD"),
            ("06.01.28", "0Y.0M.DD"),
            ("06.52.01", "0Y.WW.DD"),
        ] {
            versions.push(Calver::parse(text, format).unwrap());
        }

        versions.sort();

        assert_eq!(
            vec![
                "06.01.28",
                "06.52.1",
                "24.01",
                "24.1.28",
                "24.1.28-final",
                "2024.01",
                "2024.1",
                "2024.01-suffix",
                "2024.01.suffix",
                "2024.1-suffix",
                "2024.1.suffix",
                "2024.01.28",
                "2024.1.28",
                "2024.01.28-final",
                "2024.01.28.final",
                "2024.1.28-final",
                "2024.1.28.final",
                "2024.1.28-suffix",
                "2024-06",
                "2024-6",
                "2024-06-suffix",
                "2024-06.suffix",
                "2024-6-suffix",
                "2024-6.suffix",
                "2024-06-28",
                "2024-6-28",
                "2024-06-28-final",
                "2024-06-28.final",
            ],
            versions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
        );
    }
}
