use std::collections::HashMap;
use std::str::FromStr;

use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use strum::{Display, EnumIter, EnumString, EnumVariantNames, IntoEnumIterator};

lazy_static! {
    pub static ref FILE_STRUCTURES: HashMap<&'static str, FileStructure> = {
        let mut map = HashMap::new();
        for file_structure in FileStructure::iter() {
            map.insert(file_structure.value(), file_structure.to_owned());
        }
        map
    };
}

#[derive(Clone, Copy, Debug, Display, EnumIter, EnumString, EnumVariantNames, PartialEq)]
pub enum FileStructure {
    #[strum(serialize = "flat")]
    Flat,
    #[strum(serialize = "nested")]
    Nested,
}

impl FileStructure {
    pub(crate) fn value(&self) -> &'static str {
        match self {
            Self::Flat => "flat",
            Self::Nested => "nested",
        }
    }

    #[must_use]
    pub const fn variants() -> &'static [&'static str] {
        <Self as strum::VariantNames>::VARIANTS
    }
}

impl Default for FileStructure {
    fn default() -> Self {
        Self::Flat
    }
}

impl Serialize for FileStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            Self::Flat => "flat",
            Self::Nested => "nested",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileStructure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        // let structure = parse_file_structure(&s).unwrap_or_else(|e| {
        //     eprintln!("Error when parsing {}, fallback to default settings. Error: {:?}\n", s, e);
        //     FileStructure::default()
        // });
        let structure = <FileStructure as FromStr>::from_str(&s).unwrap_or_else(|e| {
            eprintln!(
                "Error when parsing {}, fallback to default settings. Error: {:?}\n",
                s, e
            );
            FileStructure::default()
        });
        Ok(structure)
    }
}

// pub(crate) fn parse_file_structure(
//     src: &str,
// ) -> Result<FileStructure, EnumError> {
//     parse_enum(&FILE_STRUCTURES, src)
// }
