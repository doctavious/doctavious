use serde::Serialize;

use crate::entries::ChangelogEntry;

#[derive(Default, Debug, Serialize)]
pub struct Release {
    /// Release version, git tag.
    pub version: Option<String>,

    /// Commit ID of the tag.
    pub tag_id: Option<String>,

    /// Entries made for the release
    pub commits: Vec<ChangelogEntry>,

    /// Timestamp of the release in seconds, from epoch.
    pub timestamp: i64,
}

/// Representation of a list of releases.
#[derive(Serialize)]
pub struct Releases {
    /// Releases.
    pub releases: Vec<Release>,
}
