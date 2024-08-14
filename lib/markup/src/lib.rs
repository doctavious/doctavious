use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator, VariantNames};
use thiserror::Error;

lazy_static! {
    pub static ref MARKUP_FORMAT_EXTENSIONS: HashMap<&'static str, MarkupFormat> = {
        let mut map = HashMap::new();
        for markup_format in MarkupFormat::iter() {
            map.insert(markup_format.extension(), markup_format.to_owned());
        }
        map
    };
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MarkupError {
    #[error("Enum parsing error: {0}")]
    StrumParseError(#[from] strum::ParseError),

    #[error("Unknown markup extension for path: {0}")]
    UnknownMarkupExtension(String),
}

pub type MarkupResult<T> = Result<T, MarkupError>;

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, VariantNames, PartialEq)]
#[non_exhaustive]
pub enum MarkupFormat {
    #[strum(serialize = "adoc")]
    Asciidoc,
    #[strum(serialize = "md")]
    Markdown,
    // TODO: Other(str)?
}

impl MarkupFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Asciidoc => "adoc",
            Self::Markdown => "md",
        }
    }

    pub fn leading_header_character(&self) -> char {
        match self {
            Self::Asciidoc => '=',
            Self::Markdown => '#',
        }
    }

    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> MarkupResult<Self> {
        let path = path.as_ref();
        // has to be a better way to do this
        let extension = path
            .extension()
            .ok_or(MarkupError::UnknownMarkupExtension(
                path.to_string_lossy().to_string(),
            ))?
            .to_str()
            .ok_or(MarkupError::UnknownMarkupExtension(
                path.to_string_lossy().to_string(),
            ))?;
        Ok(Self::from_str(extension)?)
    }
}

impl Default for MarkupFormat {
    fn default() -> Self {
        Self::Markdown
    }
}

impl Serialize for MarkupFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.extension();
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MarkupFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let markup_format = <MarkupFormat as FromStr>::from_str(&s).unwrap_or_else(|e| {
            eprintln!(
                "Error when parsing {}, fallback to default settings. Error: {:?}\n",
                s, e
            );
            MarkupFormat::default()
        });
        Ok(markup_format)
    }
}
