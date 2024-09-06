use std::path::{Path, PathBuf};

use glob::Pattern;
use indexmap::IndexMap;
use regex::Regex;

use crate::commit::{ScmCommit, ScmCommitRange, ScmTag};
use crate::drivers::git::TagSort;
use crate::drivers::ScmRepository;
use crate::errors::ScmResult;

const HOOK_NAMES: [&str; 13] = [
    "changegroup",
    "commit",
    "incoming",
    "outgoing",
    "prechangegroup",
    "precommit",
    "preoutgoing",
    "pretag",
    "pretxnchangegroup",
    "pretxncommit",
    "preupdate",
    "tag",
    "update",
];

pub struct HgScmRepository;

impl ScmRepository for HgScmRepository {
    fn checkout(&self, reference: &str) -> ScmResult<()> {
        todo!()
    }

    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool> {
        todo!()
    }

    fn write(&self, path: &Path, message: &str) -> ScmResult<()> {
        todo!()
    }

    fn last_commit(&self) -> ScmResult<ScmCommit> {
        todo!()
    }

    fn commits(
        &self,
        range: Option<&ScmCommitRange>,
        include_paths: Option<&Vec<Pattern>>,
        exclude_paths: Option<&Vec<Pattern>>,
        limit_commits: Option<usize>,
    ) -> ScmResult<Vec<ScmCommit>> {
        todo!()
    }

    fn tags(
        &self,
        includes: Option<&Vec<Regex>>,
        excludes: Option<&Vec<Regex>>,
        sort: TagSort,
        suffix_order: Option<&Vec<String>>,
    ) -> ScmResult<IndexMap<String, ScmTag>> {
        todo!()
    }

    fn current_tag(&self) -> Option<ScmTag> {
        todo!()
    }

    fn latest_tag(&self) -> ScmResult<Option<ScmTag>> {
        todo!()
    }

    fn get_tag(&self, name: &str) -> ScmTag {
        todo!()
    }

    fn is_dirty(&self) -> ScmResult<bool> {
        todo!()
    }

    fn supported_hooks(&self) -> Vec<&'static str> {
        todo!()
    }

    fn supports_hook(&self, hook: &str) -> bool {
        todo!()
    }

    fn hooks_path(&self) -> ScmResult<PathBuf> {
        todo!()
    }

    fn is_hook_file_sample(&self, path: &Path) -> bool {
        todo!()
    }

    fn info_path(&self) -> ScmResult<PathBuf> {
        todo!()
    }

    fn all_files(&self) -> ScmResult<Vec<PathBuf>> {
        todo!()
    }

    fn staged_files(&self) -> ScmResult<Vec<PathBuf>> {
        todo!()
    }

    fn push_files(&self) -> ScmResult<Vec<PathBuf>> {
        todo!()
    }

    fn files_by_command(&self, cmd: &String) -> ScmResult<Vec<PathBuf>> {
        todo!()
    }

    fn scm(&self) -> &'static str {
        todo!()
    }
}
