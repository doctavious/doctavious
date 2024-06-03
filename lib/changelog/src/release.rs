use serde::{Deserialize, Serialize};

use crate::commit::Commit;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
    /// Release version, git tag.
    pub version: Option<String>,
    /// Commits made for the release.
    pub commits: Vec<Commit>,
    /// Commit ID of the tag.
    pub commit_id: Option<String>,
    /// Timestamp of the release in seconds, from epoch.
    pub timestamp: i64,
    /// Previous release.
    pub previous: Option<Box<Release>>,
}

impl Release {
    // TODO: Calculates the next version based on the commits.
}

/// Representation of a list of releases.
#[derive(Serialize)]
pub struct Releases<'a> {
    /// Releases.
    pub releases: &'a Vec<Release>,
}
