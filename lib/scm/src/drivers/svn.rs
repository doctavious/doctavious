use std::path::{Path, PathBuf};

use glob::Pattern;
use indexmap::IndexMap;

use crate::{ScmCommit, ScmRepository, ScmResult};

const HOOK_NAMES: [&str; 9] = [
    "start-commit",
    "pre-commit",
    "post-commit",
    "pre-revprop-change",
    "post-revprop-change",
    "pre-lock",
    "post-lock",
    "pre-unlock",
    "post-unlock",
];

pub struct SvnScmRepository;

impl ScmRepository for SvnScmRepository {
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
        range: &Option<String>,
        include_paths: Option<&Vec<Pattern>>,
        exclude_paths: Option<&Vec<Pattern>>,
    ) -> ScmResult<Vec<ScmCommit>> {
        todo!()
    }

    fn tags(
        &self,
        pattern: &Option<String>,
        topo_order: bool,
    ) -> ScmResult<IndexMap<String, String>> {
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
        path.ends_with(".tmpl")
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
