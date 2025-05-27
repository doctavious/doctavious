// implementation base on https://github.com/systeminit/si/blob/main/lib/object-tree/src/lib.rs

//! A cryptographic hashing strategy which can help to determine if two arbitrary objects are
//! identical, assuming they can be both deterministically serialized into bytes.
//!
//! # Implementation Notes
//!
//! The current implementation uses the [BLAKE3] hashing function, but this strategy is designed to
//! be opaque, meaning that it might be changed in the future.
//!
//! [BLAKE3]: https://github.com/BLAKE3-team/BLAKE3

// https://github.com/systeminit/si/blob/main/lib/object-tree/src/hash.rs
use std::default::Default;
use std::fmt;
use std::str::FromStr;

use serde::de::Visitor;
use serde::{Deserialize, Serialize, de};
use thiserror::Error;

pub struct Hasher(blake3::Hasher);

impl Hasher {
    pub fn new() -> Self {
        Self(blake3::Hasher::new())
    }

    pub fn update(&mut self, input: &[u8]) -> &mut Self {
        self.0.update(input);
        self
    }

    pub fn finalize(&mut self) -> Hash {
        Hash {
            0: self.0.finalize(),
        }
    }
}

/// A cryptographic hash value, computed over an input of bytes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Hash(blake3::Hash);

impl Hash {
    /// Creates and returns a new [Hash] value, computed from an input of bytes.
    #[must_use]
    pub fn new(input: &[u8]) -> Self {
        Self(blake3::hash(input))
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash::new("".as_bytes())
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a blake3 hash string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Hash::from_str(v).map_err(|e| E::custom(e.to_string()))
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}

/// An error when parsing a String representation of a [`Hash`].
#[derive(Debug, Error)]
#[error("failed to parse hash hex string")]
pub struct HashParseError(#[from] blake3::HexError);

impl FromStr for Hash {
    type Err = HashParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(blake3::Hash::from_str(s)?))
    }
}

#[cfg(test)]
mod tests {
    use serde::de::{self, Deserializer, IntoDeserializer};

    use super::*;

    #[test]
    fn test_deserialize() {
        let hash = Hash::new(b"doctavious docs");
        let hash_string = hash.to_string();
        let deserializer: de::value::StrDeserializer<de::value::Error> =
            hash_string.as_str().into_deserializer();
        let hash_deserialized: Hash = deserializer
            .deserialize_any(HashVisitor)
            .expect("able to deserialize");

        assert_eq!(hash, hash_deserialized);
    }
}
