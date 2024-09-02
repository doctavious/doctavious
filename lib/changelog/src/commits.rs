use scm::commit::{ScmCommit, ScmTag};

// TODO: might need to handle multiple tags for the same set of commits.
// In a mono-repo and using include/exclude it might skip over commit and as a result might need to
// find closest tag instead of exact
// See https://github.com/orhun/git-cliff/pull/711
/// Commits grouped by tag
///
/// tag will be none for Commits that are untagged
pub struct ScmTaggedCommits {
    pub repository: String,
    pub tag: Option<ScmTag>,
    pub commits: Vec<ScmCommit>,
    pub timestamp: Option<i64>,
}
