use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use git2::{
    BranchType, Commit as Git2Commit, Config, DescribeFormatOptions, DescribeOptions, Direction,
    IndexAddOption, Oid as Git2Oid, Repository as Git2Repository, Signature as Git2Signature,
    Signature, StatusOptions,
};
use glob::Pattern;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use regex::Regex;
use serde_derive::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString, VariantNames};

use crate::GIT;
use crate::commit::{ScmCommit, ScmCommitRange, ScmSignature, ScmTag};
use crate::drivers::ScmRepository;
use crate::errors::ScmResult;

// TODO: Oid strut

// TODO: get_branches
// TODO: branch_exists
// TODO: checkout
// TODO: add_and_commit
// TODO: push - i think this is ok here as it probably(?) doesnt care about provider
// TODO: get_commits
// TODO: find_last_commit
// TODO: tags

// TODO: should we make this an enum?
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

lazy_static! {
    // TODO: probably doesnt need to be an owned type
    static ref TAG_SIGNATURE_REGEX: Regex = Regex::new(
        r"(?s)-----BEGIN PGP SIGNATURE-----(.*?)-----END PGP SIGNATURE-----"
    ).unwrap();
}

pub struct Oid {
    pub bytes: Vec<u8>,
}

pub struct PathSpec {}

impl From<Git2Commit<'_>> for ScmCommit {
    fn from(value: Git2Commit) -> Self {
        ScmCommit {
            id: value.id().to_string(),
            message: value.message().unwrap_or_default().to_string(),
            description: value.summary().unwrap_or_default().to_string(),
            body: value.body().unwrap_or_default().to_string(),
            author: value.author().into(),
            committer: value.committer().into(),
            timestamp: value.time().seconds(),
        }
    }
}

impl From<Git2Signature<'_>> for ScmSignature {
    fn from(signature: Git2Signature) -> Self {
        Self {
            name: signature.name().map(String::from),
            email: signature.email().map(String::from),
            timestamp: signature.when().seconds(),
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

#[remain::sorted]
#[derive(
    Clone, Copy, Debug, Display, EnumIter, EnumString, VariantNames, Default, Deserialize, Serialize,
)]
pub enum TagSort {
    Alphabetical,
    // this could be potentially several different fields
    // authordate, committerdate, creatordate, taggerdate
    Chronological,
    #[default]
    Version,
}

pub struct GitScmRepository {
    inner: Git2Repository,
}

impl GitScmRepository {
    pub fn init<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
        Git2Repository::init(&path)?;
        GitScmRepository::new(path)
    }

    pub fn discover<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
        Ok(Self {
            inner: Git2Repository::discover(path)?,
        })
    }

    pub fn new<P: AsRef<Path>>(path: P) -> ScmResult<Self> {
        Ok(Self {
            inner: Git2Repository::open(&path)?,
        })
    }

    fn find_last_commit(&self) -> ScmResult<Option<Git2Commit>> {
        let head = self.inner.head();
        let parent_commit = match head {
            Ok(head_ref) => {
                let resolved = head_ref.resolve();
                if let Ok(commit) = resolved.and_then(|r| r.peel_to_commit()) {
                    Some(commit)
                } else {
                    None
                }
            }
            Err(_) => None, // Handle cases where HEAD does not exist (first commit)
        };

        Ok(parent_commit)
        // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
        // Ok(self.inner.head()?.resolve()?.peel_to_commit()?)
    }

    fn commit(
        &self,
        message: &str,
        signature: Option<&Signature>,
    ) -> ScmResult<crate::drivers::git::Oid> {
        let oid = self.inner.index()?.write_tree()?;
        let tree = self.inner.find_tree(oid)?;
        let signature = match signature {
            Some(sig) => sig,
            None => &self.inner.signature()?,
        };
        let parent_commit = self.find_last_commit()?;
        let commit_oid = if let Some(parent_commit) = parent_commit {
            self.inner.commit(
                Some("HEAD"),
                signature,
                signature,
                message,
                &tree,
                &[&parent_commit],
            )?
        } else {
            self.inner
                .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?
        };

        Ok(Oid {
            bytes: commit_oid.as_bytes().to_vec(),
        })
    }

    pub fn add_all(&self) -> ScmResult<()> {
        let mut index = self.inner.index()?;
        index.add_all(["."].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
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
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
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

    fn last_tag_commit(&self) -> ScmResult<Option<String>> {
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        let last_tag_commit_output = command
            .args(["rev-list", "--tags", "--max-count=1"])
            .output()?;

        let commit = String::from_utf8(last_tag_commit_output.stdout)?
            .trim()
            .to_string();
        if commit.is_empty() {
            Ok(None)
        } else {
            Ok(Some(commit))
        }
    }

    pub fn get_commit_hash(&self, revision: &str) -> ScmResult<String> {
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        let output = command
            .args(["rev-parse", "--short", revision])
            .output()?
            .stdout;
        let commit_hash = String::from_utf8(output)?.trim().to_string();
        Ok(commit_hash)
    }

    /// Add and commit files
    pub fn am(&self, message: &str, signature: Option<&Signature>) -> ScmResult<()> {
        self.add_all()?;
        self.commit(message, signature)?;
        Ok(())
    }

    pub fn get_config(&self) -> ScmResult<Config> {
        let config = self.inner.config()?;
        Ok(config)
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

    fn write(&self, path: &Path, message: &str, signature: Option<&Signature>) -> ScmResult<()> {
        let mut index = self.inner.index()?;
        index.add_path(path)?;
        self.commit(message, signature)?;
        self.push()
    }

    fn commit(&self, message: &str, signature: Option<&Signature>) -> ScmResult<()> {
        self.commit(message, signature)?;
        Ok(())
    }

    fn last_commit(&self) -> ScmResult<Option<ScmCommit>> {
        // TODO: does this need to call `resolve` or can we just call `peel_to_commit`?
        Ok(self.find_last_commit()?.and_then(|c| Some(c.into())))
    }

    /// Parses and returns the commits.
    ///
    /// Sorts the commits by their time.
    fn commits(
        &self,
        range: Option<&ScmCommitRange>,
        include_paths: Option<&Vec<Pattern>>,
        exclude_paths: Option<&Vec<Pattern>>,
        limit_commits: Option<usize>,
    ) -> ScmResult<Vec<ScmCommit>> {
        // libgit2 and as a result git2's revwalk doesnt support filtering by paths touched by commits
        // see https://github.com/libgit2/libgit2/issues/3041
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        // only output commit hash for now as we re-fetch commits to avoid parsing git log
        let mut args = vec!["log".to_string(), "--pretty=%H".to_string()];

        if let Some(num) = limit_commits {
            args.extend(["-n".to_string(), num.to_string()])
        }

        if let Some(range) = range {
            let start = &range.0;
            let end = match &range.1 {
                None => "HEAD",
                Some(e) => e,
            };

            args.push(format!("{start}..{end}"));
        }

        let mut path_specs = vec![];
        if let Some(include_paths) = include_paths {
            for include_path in include_paths {
                path_specs.push(include_path.to_string());
            }
        }

        if let Some(exclude_paths) = exclude_paths {
            for exclude_path in exclude_paths {
                path_specs.push(format!(":(exclude){}", exclude_path));
            }
        }

        if !path_specs.is_empty() {
            args.push("-- .".to_string());
            args.extend(path_specs);
        }

        let output = command.args(args).output()?.stdout;
        let commits: Vec<ScmCommit> = String::from_utf8(output)?
            .lines()
            .filter_map(|id| Git2Oid::from_str(id).ok())
            .filter_map(|oid| self.inner.find_commit(oid).ok())
            .map(|c| c.into())
            .collect();

        Ok(commits)
    }

    /// Parses and returns a commit-tag map.
    ///
    /// It collects lightweight and annotated tags.
    fn tags(
        &self,
        includes: Option<&Vec<Regex>>,
        excludes: Option<&Vec<Regex>>,
        sort: TagSort,
        suffix_order: Option<&Vec<String>>,
    ) -> ScmResult<IndexMap<String, ScmTag>> {
        // initially used git2's `tag_names` to get tags however I can't find a good way to mimic
        // git's `--sort=v:refname` support via git2
        // TODO: might have to expose `versionsort.suffix` configuration
        // how do we want to generally handle suffix
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        let mut args = Vec::new();
        if let Some(suffix_order) = suffix_order {
            for suffix in suffix_order {
                args.extend(["-c".to_string(), format!("versionsort.suffix={suffix}")]);
            }
        }

        args.extend(["tag".to_string(), "-l".to_string()]);
        match sort {
            TagSort::Chronological => args.push("--sort=creatordate".to_string()),
            TagSort::Version => args.push("--sort=v:refname".to_string()),
            _ => {}
        }

        let output = command.args(args).output()?;

        let tag_names = output
            .stdout
            .split(|&b| b == b'\n')
            .filter(|&x| !x.is_empty())
            .filter_map(|line| std::str::from_utf8(line).ok())
            .filter(|tag_name| {
                if let Some(includes) = includes {
                    for include in includes {
                        if include.is_match(tag_name) {
                            return true;
                        }
                    }
                }

                false
            })
            .filter(|tag_name| {
                if let Some(excludes) = excludes {
                    for exclude in excludes {
                        if exclude.is_match(tag_name) {
                            return false;
                        }
                    }
                }

                true
            })
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let mut tags: Vec<(ScmCommit, ScmTag)> = Vec::new();
        for name in tag_names {
            let obj = self.inner.revparse_single(name.as_str())?;
            if let Some(tag) = obj.as_tag() {
                if let Some(commit) = tag
                    .target()
                    .ok()
                    .map(|target| target.into_commit().ok())
                    .flatten()
                {
                    let timestamp = commit.time().seconds();
                    tags.push((
                        commit.into(),
                        ScmTag {
                            id: Some(tag.id().to_string()),
                            name: tag.name().unwrap_or(&name).to_string(),
                            message: tag
                                .message()
                                .map(|msg| TAG_SIGNATURE_REGEX.replace(msg, "").trim().to_owned()),
                            timestamp,
                        },
                    ));
                }
            } else if let Ok(commit) = obj.into_commit() {
                let commit_id = commit.id().to_string();
                let timestamp = commit.time().seconds();
                tags.push((
                    commit.into(),
                    ScmTag {
                        id: Some(commit_id),
                        name,
                        message: None,
                        timestamp,
                    },
                ));
            }
        }

        Ok(tags.into_iter().map(|(a, b)| (a.id, b)).collect())
    }

    /// Returns the current tag.
    ///
    /// It is the same as running `git describe --tags --abbrev=0`
    fn current_tag(&self) -> Option<ScmTag> {
        self.inner
            .describe(DescribeOptions::new().describe_tags())
            .ok()
            .and_then(|describe| {
                describe
                    .format(Some(DescribeFormatOptions::new().abbreviated_size(0)))
                    .ok()
                    .map(|name| self.get_tag(&name))
            })
    }

    fn latest_tag(&self) -> ScmResult<Option<ScmTag>> {
        let last_tag_commit = self.last_tag_commit()?;
        if let Some(last_tag_commit) = last_tag_commit {
            let mut command = Command::new("git");
            if let Some(git_workdir) = self.inner.workdir() {
                command.current_dir(git_workdir);
            }

            // TODO: confirm this gets lightweight and annotated tags
            // if using `--all` need to remove prefix e.g. `tags/` / `head/`
            let output = command
                .args(["describe", "--tags", &last_tag_commit])
                .output()?;

            let tag = String::from_utf8(output.stdout)?.trim_end().to_string();
            if tag.is_empty() {
                Ok(None)
            } else {
                Ok(Some(self.get_tag(&tag)))
            }
        } else {
            Ok(None)
        }
    }

    fn get_tag(&self, name: &str) -> ScmTag {
        match self
            .inner
            .resolve_reference_from_short_name(name)
            .and_then(|r| r.peel_to_tag())
        {
            Ok(tag) => ScmTag {
                id: Some(tag.id().to_string()),
                name: tag.name().unwrap_or(name).to_string(),
                message: tag
                    .message()
                    .map(|msg| TAG_SIGNATURE_REGEX.replace(msg, "").trim().to_owned()),
                timestamp: self
                    .inner
                    .find_commit(tag.id())
                    .map(|c| c.time().seconds())
                    .unwrap_or_default(),
            },
            // TODO: should probably return None
            _ => ScmTag {
                id: None,
                name: name.to_owned(),
                message: None,
                timestamp: 0,
            },
        }
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
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        let output = command
            .args(["rev-parse", "--git-path", "hooks"])
            .output()?
            .stdout;

        Ok(self
            .inner
            .workdir()
            .ok_or(git2::Error::from_str("Cant get hooks path from repository"))?
            .join(String::from_utf8(output)?.trim_end()))
    }

    fn is_hook_file_sample(&self, path: &Path) -> bool {
        path.ends_with(".sample")
    }

    fn info_path(&self) -> ScmResult<PathBuf> {
        let mut command = Command::new("git");
        if let Some(git_workdir) = self.inner.workdir() {
            command.current_dir(git_workdir);
        }

        let output = command
            .args(["rev-parse", "--git-path", "info"])
            .output()?
            .stdout;

        Ok(self
            .inner
            .workdir()
            .ok_or(git2::Error::from_str("Cant get info path from repository"))?
            .join(String::from_utf8(output)?.trim_end()))
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

    // TODO: given return of Vec<PathBuf> we should change the name to something like `diff_paths`
    // and make names only required. Given we aren't actually parsing diff no reason to make it
    // generic

    fn diff_paths(&self, range: Option<&ScmCommitRange>) -> ScmResult<Vec<PathBuf>> {
        let mut args = vec!["diff".to_string(), "--name-only".to_string()];

        if let Some(range) = range {
            // TODO: move logic to a common area
            let start = &range.0;
            let end = match &range.1 {
                None => "HEAD",
                Some(e) => e,
            };

            args.push(format!("{start}..{end}"));
        }

        self.get_files(args)
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use glob::Pattern;

    use crate::drivers::ScmRepository;
    use crate::drivers::git::{GitScmRepository, TagSort};

    #[test]
    fn commits() {
        println!("{:?}", env::current_dir().unwrap());
        let scm = GitScmRepository::new("../../").unwrap();

        let include = vec![Pattern::new("README.md").unwrap()];
        let exclude = vec![
            Pattern::new("bin/cli/").unwrap(),
            Pattern::new("lib/").unwrap(),
        ];
        let commits = scm
            .commits(
                None,
                None, //Some(&include),
                Some(&exclude),
                None,
            )
            .unwrap();
        for c in commits {
            println!("{:?}", &c);
        }
    }

    // TODO: need to implement something that works in CI
    // #[test]
    // fn git_cliff_tags() {
    //     println!("{:?}", env::current_dir().unwrap());
    //     let scm = GitScmRepository::new("../../../../git-cliff").unwrap();
    //     let commits = scm.tags(None, None, TagSort::Version, None).unwrap();
    //     for c in commits {
    //         println!("{:?}", &c);
    //     }
    // }

    // TODO: need to implement something that works in CI
    // #[test]
    // fn latest_tags() {
    //     println!("{:?}", env::current_dir().unwrap());
    //     let scm = GitScmRepository::new("../../../../gitlab-ce").unwrap();
    //     let latest = scm.latest_tag().unwrap();
    //     println!("{:?}", &latest);
    // }
}
