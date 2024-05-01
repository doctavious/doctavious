use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::DateTime;
use git2::{
    BranchType, Commit as Git2Commit, Direction, IndexAddOption, Repository as Git2Repository,
    Signature as Git2Signature, Sort, StatusOptions,
};
use indexmap::IndexMap;
use regex::Regex;

use crate::drivers::Scm;
use crate::{ScmCommit, ScmRepository, ScmResult, ScmSignature, GIT};

// TODO: Oid strut

// TODO: get_branches
// TODO: branch_exists
// TODO: checkout
// TODO: add_and_commit
// TODO: push - i think this is ok here as it probably(?) doesnt care about provider
// TODO: get_commits
// TODO: find_last_commit
// TODO: tags

const HOOK_NAMES: [&str; 21] = [
    "applypatch-msg",
    "pre-applypatch",
    "post-applypatch",
    "pre-commit",
    "pre-merge-commit",
    "prepare-commit-msg",
    "commit-msg",
    "post-commit",
    "pre-rebase",
    "post-checkout",
    "post-merge",
    "pre-push",
    "pre-receive",
    "update",
    "post-receive",
    "post-update",
    "push-to-checkout",
    "pre-auto-gc",
    "post-rewrite",
    "sendemail-validate",
    "post-index-change",
];

pub struct Oid {
    pub bytes: Vec<u8>,
}

impl From<Git2Commit<'_>> for ScmCommit {
    fn from(value: Git2Commit) -> Self {
        ScmCommit {
            id: value.id().to_string(),
            message: value.message().map(String::from),
            author: value.author().into(),
            timestamp: DateTime::from_timestamp_millis(value.time().seconds()),
        }
    }
}

impl From<Git2Signature<'_>> for ScmSignature {
    fn from(signature: Git2Signature) -> Self {
        Self {
            name: signature.name().map(String::from),
            email: signature.email().map(String::from),
            timestamp: DateTime::from_timestamp_millis(signature.when().seconds()),
        }
    }
}

// TODO: should we instead have a GitScm rather than a GitScmRepository?
// pub struct GitScm {
//     // TODO: change to repository
//     pub(crate) inner: Git2Repository,
// }
//
// impl GitScm {
//     pub fn new() -> ScmResult<Self> {
//         Ok(Self {
//             inner: Git2Repository::open(".")?,
//         })
//     }
//
//     fn find_last_commit(&self) -> ScmResult<Git2Commit> {
//         // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
//         Ok(self.inner.head()?.resolve()?.peel_to_commit()?)
//     }
//
//     fn commit(&self, message: &str) -> ScmResult<crate::drivers::git::Oid> {
//         let parent_commit = self.find_last_commit()?;
//         let tree = self.inner.find_tree(parent_commit.tree_id())?;
//         let signature = self.inner.signature()?;
//         let commit = self.inner.commit(
//             Some("HEAD"),
//             &signature,
//             &signature,
//             message,
//             &tree,
//             &[&parent_commit],
//         )?;
//
//         Ok(crate::drivers::git::Oid {
//             bytes: commit.as_bytes().to_vec(),
//         })
//     }
//
//     fn push(&self) -> ScmResult<()> {
//         let mut remote = self.inner.find_remote("origin")?;
//         remote.connect(Direction::Push)?;
//         // TODO: what should this be?
//         Ok(remote.push(&["refs/heads/master:refs/heads/master"], None)?)
//     }
// }

// impl ScmRepository for GitScm {
//     // pub fn open<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
//     //     Ok(Self {
//     //         inner: Git2Repository::open(path)?,
//     //     })
//     // }
//
//
//
//     fn checkout(&self, reference: &str) -> ScmResult<()> {
//         // does this need to be `&("refs/heads/".to_owned() + reference)`
//         let (object, reference) = self.inner.revparse_ext(reference)?;
//         self.inner.checkout_tree(&object, None)?;
//         match reference {
//             // gref is an actual reference like branches or tags
//             Some(gref) => self.inner.set_head(gref.name().unwrap()),
//             // this is a commit, not a reference
//             None => self.inner.set_head_detached(object.id()),
//         }?;
//
//         Ok(())
//
//     }
//
//     fn branch_exists(&self, branch_name: &str) -> ScmResult<bool> {
//         let re = Regex::new(branch_name)?;
//
//         let branches = self.inner.branches(Some(BranchType::Remote))?;
//         // tried using iter but was getting "referencing data owned by the current function" errors
//         for branch in branches {
//             if let Ok((branch, _)) = branch {
//                 if let Some(branch_name) = branch.name()? {
//                     if re.is_match(branch_name) {
//                         return Ok(true);
//                     }
//                 }
//             }
//         }
//
//         Ok(false)
//     }
//
//     // fn add_and_commit(&self, path: &Path, message: &str) -> ScmResult<crate::drivers::git::Oid> {
//     //     let mut index = self.inner.index()?;
//     //     index.add_path(path)?;
//     //     Ok(self.commit(message)?)
//     // }
//
//     fn write(&self, path: &Path, message: &str) -> ScmResult<()> {
//         let mut index = self.inner.index()?;
//         index.add_path(path)?;
//         self.commit(message)?;
//         self.push()
//     }
//
//     fn last_commit(&self) -> ScmResult<ScmCommit> {
//         // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
//         Ok(self.find_last_commit()?.into())
//     }
//
//     /// Parses and returns the commits.
//     ///
//     /// Sorts the commits by their time.
//     fn commits(&self, range: Option<String>) -> ScmResult<Vec<ScmCommit>> {
//         let mut revwalk = self.inner.revwalk()?;
//         revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
//         if let Some(range) = range {
//             revwalk.push_range(&range)?;
//         } else {
//             revwalk.push_head()?;
//         }
//         Ok(revwalk
//             .filter_map(|id| id.ok())
//             .filter_map(|id| self.inner.find_commit(id).ok())
//             .map(|c| c.into())
//             .collect())
//     }
//
//     /// Parses and returns a commit-tag map.
//     ///
//     /// It collects lightweight and annotated tags.
//     fn tags(&self, pattern: &Option<String>) -> ScmResult<IndexMap<String, String>> {
//         let mut tags: Vec<(ScmCommit, String)> = Vec::new();
//
//         // from https://github.com/rust-lang/git2-rs/blob/master/examples/tag.rs
//         // also check https://github.com/orhun/git-cliff/blob/main/git-cliff-core/src/repo.rs tags
//         for name in self
//             .inner
//             .tag_names(pattern.as_deref())?
//             .iter()
//             .flatten()
//             .map(String::from)
//         {
//             let obj = self.inner.revparse_single(name.as_str())?;
//             if let Some(tag) = obj.as_tag() {
//                 if let Some(commit) = tag
//                     .target()
//                     .ok()
//                     .map(|target| target.into_commit().ok())
//                     .flatten()
//                 {
//                     tags.push((commit.into(), name));
//                 }
//             } else if let Ok(commit) = obj.into_commit() {
//                 tags.push((commit.into(), name));
//             }
//         }
//
//         tags.sort_by(|a, b| a.0.timestamp.cmp(&b.0.timestamp));
//         Ok(tags.into_iter().map(|(a, b)| (a.id, b)).collect())
//     }
//
//     /// Determines if there are any current changes in the working directory / staging area
//     fn is_dirty(&self) -> ScmResult<bool> {
//         let mut opts = StatusOptions::new();
//         opts.include_ignored(false);
//         opts.include_untracked(false);
//
//         let statuses = self.inner.statuses(Some(&mut opts))?;
//         Ok(!statuses.is_empty())
//     }
//
//     fn scm(&self) -> &'static str {
//         GIT
//     }
// }

pub struct GitScmRepository {
    inner: Git2Repository,
}

impl GitScmRepository {
    pub fn init<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
        Git2Repository::init(&path)?;
        GitScmRepository::new(path)
    }

    pub fn new<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
        Ok(Self {
            inner: Git2Repository::open(&path)?,
        })
    }

    fn find_last_commit(&self) -> ScmResult<Git2Commit> {
        // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
        Ok(self.inner.head()?.resolve()?.peel_to_commit()?)
    }

    fn commit(&self, message: &str) -> ScmResult<crate::drivers::git::Oid> {
        let parent_commit = self.find_last_commit()?;
        let tree = self.inner.find_tree(parent_commit.tree_id())?;
        let signature = self.inner.signature()?;
        let commit = self.inner.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(Oid {
            bytes: commit.as_bytes().to_vec(),
        })
    }

    pub fn add_all(&self) {
        let mut index = self.inner.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
    }

    fn push(&self) -> ScmResult<()> {
        let mut remote = self.inner.find_remote("origin")?;
        remote.connect(Direction::Push)?;
        // TODO: what should this be?
        Ok(remote.push(&["refs/heads/master:refs/heads/master"], None)?)
    }

    /// accepts git command args and returns its result as a list of filepaths.
    fn get_files<I, S>(&self, args: I) -> ScmResult<Vec<PathBuf>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("git");
        if let Some(parent) = self.inner.path().parent() {
            command.args(["-C", &parent.to_string_lossy()]);
        }

        let output = command.args(args).output()?.stdout;

        let files: Vec<_> = output
            .split(|&b| b == b'\n')
            .filter(|&x| !x.is_empty())
            .filter_map(|line| std::str::from_utf8(line).ok())
            .map(|s| PathBuf::from(s.trim_end()))
            .collect();

        Ok(files)
    }
}

impl ScmRepository for GitScmRepository {
    fn checkout(&self, reference: &str) -> ScmResult<()> {
        // does this need to be `&("refs/heads/".to_owned() + reference)`
        let (object, reference) = self.inner.revparse_ext(reference)?;
        self.inner.checkout_tree(&object, None)?;
        match reference {
            // gref is an actual reference like branches or tags
            Some(gref) => self.inner.set_head(gref.name().unwrap()),
            // this is a commit, not a reference
            None => self.inner.set_head_detached(object.id()),
        }?;

        Ok(())
    }

    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool> {
        let re = Regex::new(branch_name)?;

        let branches = self.inner.branches(Some(BranchType::Remote))?;
        // tried using iter but was getting "referencing data owned by the current function" errors
        for branch in branches {
            if let Ok((branch, _)) = branch {
                if let Some(branch_name) = branch.name()? {
                    if re.is_match(branch_name) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    // fn add_and_commit(&self, path: &Path, message: &str) -> ScmResult<crate::drivers::git::Oid> {
    //     let mut index = self.inner.index()?;
    //     index.add_path(path)?;
    //     Ok(self.commit(message)?)
    // }

    fn write(&self, path: &Path, message: &str) -> ScmResult<()> {
        let mut index = self.inner.index()?;
        index.add_path(path)?;
        self.commit(message)?;
        self.push()
    }

    fn last_commit(&self) -> ScmResult<ScmCommit> {
        // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
        Ok(self.find_last_commit()?.into())
    }

    /// Parses and returns the commits.
    ///
    /// Sorts the commits by their time.
    fn commits(&self, range: Option<String>) -> ScmResult<Vec<ScmCommit>> {
        let mut revwalk = self.inner.revwalk()?;
        revwalk.set_sorting(Sort::TIME | Sort::TOPOLOGICAL)?;
        if let Some(range) = range {
            revwalk.push_range(&range)?;
        } else {
            revwalk.push_head()?;
        }
        Ok(revwalk
            .filter_map(|id| id.ok())
            .filter_map(|id| self.inner.find_commit(id).ok())
            .map(|c| c.into())
            .collect())
    }

    /// Parses and returns a commit-tag map.
    ///
    /// It collects lightweight and annotated tags.
    fn tags(&self, pattern: &Option<String>) -> ScmResult<IndexMap<String, String>> {
        let mut tags: Vec<(ScmCommit, String)> = Vec::new();

        // from https://github.com/rust-lang/git2-rs/blob/master/examples/tag.rs
        // also check https://github.com/orhun/git-cliff/blob/main/git-cliff-core/src/repo.rs tags
        for name in self
            .inner
            .tag_names(pattern.as_deref())?
            .iter()
            .flatten()
            .map(String::from)
        {
            let obj = self.inner.revparse_single(name.as_str())?;
            if let Some(tag) = obj.as_tag() {
                if let Some(commit) = tag
                    .target()
                    .ok()
                    .map(|target| target.into_commit().ok())
                    .flatten()
                {
                    tags.push((commit.into(), name));
                }
            } else if let Ok(commit) = obj.into_commit() {
                tags.push((commit.into(), name));
            }
        }

        tags.sort_by(|a, b| a.0.timestamp.cmp(&b.0.timestamp));
        Ok(tags.into_iter().map(|(a, b)| (a.id, b)).collect())
    }

    /// Determines if there are any current changes in the working directory / staging area
    fn is_dirty(&self) -> ScmResult<bool> {
        let mut opts = StatusOptions::new();
        opts.include_ignored(false);
        opts.include_untracked(false);

        let statuses = self.inner.statuses(Some(&mut opts))?;
        Ok(!statuses.is_empty())
    }

    fn supported_hooks(&self) -> Vec<&'static str> {
        HOOK_NAMES.to_vec()
    }

    fn supports_hook(&self, hook: &str) -> bool {
        HOOK_NAMES.contains(&hook.as_ref())
    }

    fn hooks_path(&self) -> ScmResult<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--git-path", "hooks"])
            .output()?
            .stdout;

        Ok(PathBuf::from(String::from_utf8(output)?.trim_end()))
    }

    fn all_files(&self) -> ScmResult<Vec<PathBuf>> {
        self.get_files(["ls-files", "--cached"])
    }

    fn staged_files(&self) -> ScmResult<Vec<PathBuf>> {
        // TODO: see how diff-filter AM / ACMR is different than d
        self.get_files(["--name-only", "--staged", "--diff-filter=d"])
    }

    fn push_files(&self) -> ScmResult<Vec<PathBuf>> {
        self.get_files(["diff", "--name-only", "HEAD", "@{push}"])
    }

    // TODO: verify this
    fn files_by_command(&self, cmd: &String) -> ScmResult<Vec<PathBuf>> {
        self.get_files([cmd])
    }

    fn scm(&self) -> &'static str {
        GIT
    }
}
