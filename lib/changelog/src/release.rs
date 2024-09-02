use serde::Serialize;
use somever::Somever;

use crate::entries::ChangelogEntry;

#[derive(Default, Debug, Serialize)]
pub struct Release {
    /// Release version
    pub version: Option<Somever>,

    // TODO: maybe its worth including the ScmTag
    /// Commit ID of the tag.
    pub tag_id: Option<String>,

    pub repository: String,

    /// Entries made for the release
    pub commits: Vec<ChangelogEntry>,

    /// Timestamp of the release in seconds, from epoch.
    pub timestamp: Option<i64>,
}

/// Representation of a list of releases.
#[derive(Serialize)]
pub struct Releases {
    /// Releases.
    pub releases: Vec<Release>,
}
