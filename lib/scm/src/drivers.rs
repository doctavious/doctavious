use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;

use crate::drivers::git::GitScmRepository;
use crate::drivers::hg::HgScmRepository;
use crate::drivers::svn::SvnScmRepository;
use crate::{ScmCommit, ScmError, ScmRepository, ScmResult};

pub mod git;
pub mod hg;
pub mod svn;

#[remain::sorted]
pub enum ScmKind {
    Git(GitScmRepository),
    Hg(HgScmRepository),
    Svn(SvnScmRepository),
}

impl ScmKind {
    pub fn get(cwd: &Path) -> ScmResult<Self> {
        // TODO: it might be better to try and discover directory such as
        // `git rev-parse --show-toplevel` and `git rev-parse --git-dir`
        if fs::metadata(cwd.join(".git")).is_ok() {
            return Ok(ScmKind::Git(GitScmRepository::new(cwd)?));
        }

        if fs::metadata(cwd.join(".svn")).is_ok() {
            // TODO: implement
            unimplemented!()
        }

        if fs::metadata(cwd.join(".hg")).is_ok() {
            // TODO: implement
            unimplemented!()
        }

        Err(ScmError::Unsupported)
    }
}

impl ScmRepository for ScmKind {
    fn checkout(&self, reference: &str) -> ScmResult<()> {
        match self {
            ScmKind::Git(r) => r.checkout(reference),
            ScmKind::Hg(r) => r.checkout(reference),
            ScmKind::Svn(r) => r.checkout(reference),
        }
    }

    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool> {
        match self {
            ScmKind::Git(r) => r.branch_exists(branch_name),
            ScmKind::Hg(r) => r.branch_exists(branch_name),
            ScmKind::Svn(r) => r.branch_exists(branch_name),
        }
    }

    fn write(&self, path: &Path, message: &str) -> ScmResult<()> {
        match self {
            ScmKind::Git(r) => r.write(path, message),
            ScmKind::Hg(r) => r.write(path, message),
            ScmKind::Svn(r) => r.write(path, message),
        }
    }

    fn last_commit(&self) -> ScmResult<ScmCommit> {
        match self {
            ScmKind::Git(r) => r.last_commit(),
            ScmKind::Hg(r) => r.last_commit(),
            ScmKind::Svn(r) => r.last_commit(),
        }
    }

    fn commits(&self, range: Option<String>) -> ScmResult<Vec<ScmCommit>> {
        match self {
            ScmKind::Git(r) => r.commits(range),
            ScmKind::Hg(r) => r.commits(range),
            ScmKind::Svn(r) => r.commits(range),
        }
    }

    fn tags(&self, pattern: &Option<String>) -> ScmResult<IndexMap<String, String>> {
        match self {
            ScmKind::Git(r) => r.tags(pattern),
            ScmKind::Hg(r) => r.tags(pattern),
            ScmKind::Svn(r) => r.tags(pattern),
        }
    }

    fn is_dirty(&self) -> ScmResult<bool> {
        match self {
            ScmKind::Git(r) => r.is_dirty(),
            ScmKind::Hg(r) => r.is_dirty(),
            ScmKind::Svn(r) => r.is_dirty(),
        }
    }

    fn supported_hooks(&self) -> Vec<&'static str> {
        match self {
            ScmKind::Git(r) => r.supported_hooks(),
            ScmKind::Hg(r) => r.supported_hooks(),
            ScmKind::Svn(r) => r.supported_hooks(),
        }
    }

    fn supports_hook(&self, hook: &str) -> bool {
        match self {
            ScmKind::Git(r) => r.supports_hook(hook),
            ScmKind::Hg(r) => r.supports_hook(hook),
            ScmKind::Svn(r) => r.supports_hook(hook),
        }
    }

    fn hook_path(&self) -> ScmResult<PathBuf> {
        match self {
            ScmKind::Git(r) => r.hook_path(),
            ScmKind::Hg(r) => r.hook_path(),
            ScmKind::Svn(r) => r.hook_path(),
        }
    }

    fn scm(&self) -> &'static str {
        match self {
            ScmKind::Git(r) => r.scm(),
            ScmKind::Hg(r) => r.scm(),
            ScmKind::Svn(r) => r.scm(),
        }
    }
}
