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
pub enum Scm {
    Git(GitScmRepository),
    Hg(HgScmRepository),
    Svn(SvnScmRepository),
}

impl Scm {
    pub fn get(cwd: &Path) -> ScmResult<Self> {
        // TODO: it might be better to try and discover directory such as
        // `git rev-parse --show-toplevel` and `git rev-parse --git-dir`
        if fs::metadata(cwd.join(".git")).is_ok() {
            return Ok(Scm::Git(GitScmRepository::new(cwd)?));
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

impl ScmRepository for Scm {
    fn checkout(&self, reference: &str) -> ScmResult<()> {
        match self {
            Scm::Git(r) => r.checkout(reference),
            Scm::Hg(r) => r.checkout(reference),
            Scm::Svn(r) => r.checkout(reference),
        }
    }

    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool> {
        match self {
            Scm::Git(r) => r.branch_exists(branch_name),
            Scm::Hg(r) => r.branch_exists(branch_name),
            Scm::Svn(r) => r.branch_exists(branch_name),
        }
    }

    fn write(&self, path: &Path, message: &str) -> ScmResult<()> {
        match self {
            Scm::Git(r) => r.write(path, message),
            Scm::Hg(r) => r.write(path, message),
            Scm::Svn(r) => r.write(path, message),
        }
    }

    fn last_commit(&self) -> ScmResult<ScmCommit> {
        match self {
            Scm::Git(r) => r.last_commit(),
            Scm::Hg(r) => r.last_commit(),
            Scm::Svn(r) => r.last_commit(),
        }
    }

    fn commits(&self, range: Option<String>) -> ScmResult<Vec<ScmCommit>> {
        match self {
            Scm::Git(r) => r.commits(range),
            Scm::Hg(r) => r.commits(range),
            Scm::Svn(r) => r.commits(range),
        }
    }

    fn tags(&self, pattern: &Option<String>) -> ScmResult<IndexMap<String, String>> {
        match self {
            Scm::Git(r) => r.tags(pattern),
            Scm::Hg(r) => r.tags(pattern),
            Scm::Svn(r) => r.tags(pattern),
        }
    }

    fn is_dirty(&self) -> ScmResult<bool> {
        match self {
            Scm::Git(r) => r.is_dirty(),
            Scm::Hg(r) => r.is_dirty(),
            Scm::Svn(r) => r.is_dirty(),
        }
    }

    fn supported_hooks(&self) -> Vec<&'static str> {
        match self {
            Scm::Git(r) => r.supported_hooks(),
            Scm::Hg(r) => r.supported_hooks(),
            Scm::Svn(r) => r.supported_hooks(),
        }
    }

    fn supports_hook(&self, hook: &str) -> bool {
        match self {
            Scm::Git(r) => r.supports_hook(hook),
            Scm::Hg(r) => r.supports_hook(hook),
            Scm::Svn(r) => r.supports_hook(hook),
        }
    }

    fn hooks_path(&self) -> ScmResult<PathBuf> {
        match self {
            Scm::Git(r) => r.hooks_path(),
            Scm::Hg(r) => r.hooks_path(),
            Scm::Svn(r) => r.hooks_path(),
        }
    }

    fn scm(&self) -> &'static str {
        match self {
            Scm::Git(r) => r.scm(),
            Scm::Hg(r) => r.scm(),
            Scm::Svn(r) => r.scm(),
        }
    }
}
