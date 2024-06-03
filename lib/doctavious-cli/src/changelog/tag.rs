use std::cmp::Ordering;

use semver::Version;

#[derive(Debug, Eq, Clone)]
pub struct Tag {
    // pub package: Option<String>,
    // pub prefix: Option<String>,
    pub version: Version,
    // pub oid: Option<Oid>,
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.version.cmp(&other.version)
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}

impl PartialOrd<Tag> for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Default for Tag {
    fn default() -> Self {
        Tag::new(Version::new(0, 0, 0))
    }
}

impl Tag {
    pub(crate) fn new(version: Version) -> Self {
        Tag { version }
    }
}
