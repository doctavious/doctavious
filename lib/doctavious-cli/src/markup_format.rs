use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumString, EnumVariantNames};

use crate::markup_format::MarkupFormat::{Asciidoc, Markdown};

// lazy_static! {
//     pub static ref MARKUP_FORMAT_EXTENSIONS: HashMap<&'static str, MarkupFormat> = {
//         let mut map = HashMap::new();
//         for markup_format in MarkupFormat::iterator() {
//             map.insert(markup_format.extension(), markup_format.to_owned());
//         }
//         map
//     };
// }

#[derive(Clone, Copy, Debug, Display, EnumString, EnumVariantNames, PartialEq)]
#[non_exhaustive]
pub enum MarkupFormat {
    #[strum(serialize = "adoc")]
    Asciidoc,
    #[strum(serialize = "md")]
    Markdown,
    // TODO: Other(str)?
}

impl MarkupFormat {
    // pub(crate) fn iterator() -> Iter<'static, MarkupFormat> {
    //     return [Asciidoc, Markdown].iter();
    // }

    pub(crate) fn extension(&self) -> &'static str {
        return match self {
            Asciidoc => "adoc",
            Markdown => "md",
        };
    }

    pub(crate) fn leading_header_character(&self) -> char {
        return match self {
            Asciidoc => '=',
            Markdown => '#',
        };
    }
}

// TODO: whats a replacement for this that doesnt use clap?
// impl ValueEnum for MarkupFormat {
//     fn value_variants<'a>() -> &'a [Self] {
//         &[Asciidoc, Markdown]
//     }
//
//     fn to_possible_value(&self) -> Option<PossibleValue> {
//         Some(match self {
//             Asciidoc => PossibleValue::new("adoc"),
//             Markdown => PossibleValue::new("md"),
//         })
//     }
// }

// impl FromStr for MarkupFormat {
//     type Err = DoctaviousError;
//
//     fn from_str(input: &str) -> DoctaviousResult<MarkupFormat> {
//         parse_enum(&MARKUP_FORMAT_EXTENSIONS, input)
//     }
// }

// part of valueEnum
// impl FromStr for MarkupFormat {
//     type Err = EnumError;
//
//     fn from_str(input: &str) -> Result<MarkupFormat, Self::Err> {
//         parse_enum(&MARKUP_FORMAT_EXTENSIONS, input)
//     }
// }

impl Default for MarkupFormat {
    fn default() -> Self {
        Markdown
    }
}

// impl Display for MarkupFormat {
//     fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
//         write!(f, "{}", self.extension())
//     }
// }

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
