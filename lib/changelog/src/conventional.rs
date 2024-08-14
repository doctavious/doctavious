use git_conventional::Commit as GitConventionalCommit;
use scm::commit::ScmCommit;
use serde_derive::Serialize;

/// Conventional Commit
/// Wrap's git_conventional's commit in order to include the raw ScmCommit
#[derive(Debug, Serialize)]
pub struct ConventionalCommit<'a> {
    pub commit: ScmCommit,
    pub conv: GitConventionalCommit<'a>,
}
