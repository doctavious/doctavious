// from https://siciarz.net/24-days-rust-git2/

use git2::{BranchType, Commit, Direction, Oid, Repository, Sort};
use indexmap::IndexMap;
use regex::Regex;
use std::path::Path;
use crate::CliResult;

// https://github.com/simeg/eureka/blob/master/src/git.rs

// Latest semver tag. Need to verify as this probably doesnt take into account pre-release or build_mod
// git tag | sort -r --version-sort | head -n1


// get default branch options
// git remote show REMOTE_REPO_NAME | grep 'HEAD branch' | cut -d' ' -f5
// git remote show [your_remote] | sed -n '/HEAD branch/s/.*: //p'
// git symbolic-ref refs/remotes/origin/HEAD | sed 's@^refs/remotes/origin/@@'
// git rev-parse --abbrev-ref origin/HEAD
// git branch --remotes --list '*/HEAD'

// TODO: return a Result<bool, Error> instead?
pub(crate) fn branch_exists(repo: &Repository, reserve_number: i32) -> bool {
    let pattern = format!("*{}", reserve_number);
    let re = Regex::new(pattern.as_str()).unwrap();
    let c = repo
        .branches(Some(BranchType::Remote))
        .unwrap()
        .find(|b| re.is_match(b.as_ref().unwrap().0.name().unwrap().unwrap()));

    return c.is_some();
}

pub(crate) fn checkout_branch(repo: &Repository, branch_name: &str) -> CliResult<()> {
    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid).unwrap();

    let branch = repo.branch(branch_name, &commit, false)?;

    let obj = repo
        .revparse_single(&("refs/heads/".to_owned() + branch_name))
        .unwrap();

    repo.checkout_tree(&obj, None)?;

    Ok(())
}

pub(crate) fn add_and_commit(
    repo: &Repository,
    path: &Path,
    message: &str,
) -> CliResult<Oid> {
    let mut index = repo.index()?;
    index.add_path(path)?;

    // TODO: is this required?
    // index.write()?;

    let oid = index.write_tree()?;
    let parent_commit = find_last_commit(&repo)?;
    let tree = repo.find_tree(oid)?;
    let signature = repo.signature()?;

    Ok(repo.commit(
        Some("HEAD"), //  point HEAD to our new commit
        &signature,   // author
        &signature,   // committer
        message,      // commit message
        &tree,        // tree
        &[&parent_commit],
    )?) // parents
}

pub(crate) fn push(repo: &Repository) -> CliResult<()> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Push)?;
    Ok(remote.push(&["refs/heads/master:refs/heads/master"], None)?)
}

fn find_last_commit(repo: &Repository) -> CliResult<Commit> {
    // let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    let obj = repo.head()?.resolve()?.peel_to_commit();
    Ok(obj.map_err(|_| git2::Error::from_str("Couldn't find commit"))?)
}

/// Parses and returns the commits.
///
/// Sorts the commits by their time.
pub fn commits(
    repo: &Repository,
    range: Option<String>,
) -> CliResult<Vec<Commit>> {
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
    if let Some(range) = range {
        revwalk.push_range(&range)?;
    } else {
        revwalk.push_head()?;
    }
    Ok(revwalk
        .filter_map(|id| id.ok())
        .filter_map(|id| repo.find_commit(id).ok())
        .collect())
}

/// Parses and returns a commit-tag map.
///
/// It collects lightweight and annotated tags.
pub fn tags(
    repo: &Repository,
    pattern: &Option<String>,
) -> CliResult<IndexMap<String, String>> {
    let mut tags: Vec<(Commit, String)> = Vec::new();

    // from https://github.com/rust-lang/git2-rs/blob/master/examples/tag.rs
    // also check https://github.com/orhun/git-cliff/blob/main/git-cliff-core/src/repo.rs tags
    for name in
    repo.tag_names(pattern.as_deref())?.iter().flatten().map(String::from)
    {
        let obj = repo.revparse_single(name.as_str())?;
        if let Some(tag) = obj.as_tag() {
            if let Some(commit) = tag
                .target()
                .ok()
                .map(|target| target.into_commit().ok())
                .flatten()
            {
                tags.push((commit, name));
            }
        } else if let Ok(commit) = obj.into_commit() {
            tags.push((commit, name));
        }
    }

    tags.sort_by(|a, b| a.0.time().seconds().cmp(&b.0.time().seconds()));
    Ok(tags.into_iter().map(|(a, b)| (a.id().to_string(), b)).collect())
}
