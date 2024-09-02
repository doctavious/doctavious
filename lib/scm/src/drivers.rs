use std::fs;
use std::path::{Path, PathBuf};

use glob::Pattern;
use indexmap::IndexMap;
use regex::Regex;

use crate::commit::{ScmCommit, ScmCommitRange, ScmTag};
use crate::drivers::git::{GitScmRepository, TagSort};
use crate::drivers::hg::HgScmRepository;
use crate::drivers::svn::SvnScmRepository;
use crate::errors::{ScmError, ScmResult};

pub mod git;
pub mod hg;
pub mod svn;

#[remain::sorted]
pub enum Scm {
    Git(GitScmRepository),
    Hg(HgScmRepository),
    Svn(SvnScmRepository),
}

pub trait ScmRepository {
    fn checkout(&self, reference: &str) -> ScmResult<()>;

    // TODO: change to return branch get_branch / resolve_branch
    // TODO: For our use cases (ADR/RFD reservation) branches aren't always useful so it might be
    // better to have a specific trait for those use cases
    fn branch_exists(&self, branch_name: &str) -> ScmResult<bool>;

    fn write(&self, path: &Path, message: &str) -> ScmResult<()>;

    // get_commit -> ScmCommit

    // TODO: this probably doesn't belong on this trait.
    // Maybe should be be able to use get_commit with some option
    fn last_commit(&self) -> ScmResult<ScmCommit>;

    // options
    fn commits(
        &self,
        range: Option<&ScmCommitRange>,
        include_paths: Option<&Vec<Pattern>>,
        exclude_paths: Option<&Vec<Pattern>>,
        limit_commits: Option<usize>,
    ) -> ScmResult<Vec<ScmCommit>>;

    // fn tagged_commits(&self) -> ScmResult<Vec<TaggedCommits>>;

    // TODO: I wonder if we would be ok with include / exclude being globs rather than regex?
    fn tags(
        &self,
        include: Option<&Regex>,
        exclude: Option<&Regex>,
        sort: TagSort,
        suffix_order: Option<&Vec<String>>,
    ) -> ScmResult<IndexMap<String, ScmTag>>;

    fn current_tag(&self) -> Option<ScmTag>;

    fn latest_tag(&self) -> ScmResult<Option<ScmTag>>;

    fn get_tag(&self, name: &str) -> ScmTag;

    /// Determines if the working directory has changes
    fn is_dirty(&self) -> ScmResult<bool>;

    // head return commit/revision

    fn supported_hooks(&self) -> Vec<&'static str>;

    fn supports_hook(&self, hook: &str) -> bool;

    fn hooks_path(&self) -> ScmResult<PathBuf>;

    fn is_hook_file_sample(&self, path: &Path) -> bool;

    fn info_path(&self) -> ScmResult<PathBuf>;

    fn all_files(&self) -> ScmResult<Vec<PathBuf>>;

    // TODO: better name than staged. What do you call files in SVN that are added but not committed?
    fn staged_files(&self) -> ScmResult<Vec<PathBuf>>;

    // TODO: push files?
    // for SVN files that are added would be staged and files that are added and have modifications would be pushed?
    // or we just say this isnt used for all SCMs
    fn push_files(&self) -> ScmResult<Vec<PathBuf>>;

    fn files_by_command(&self, cmd: &String) -> ScmResult<Vec<PathBuf>>;

    fn scm(&self) -> &'static str;
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

    pub fn ensure_hooks_directory(&self) -> ScmResult<PathBuf> {
        let path = self.hooks_path()?;
        if !path.exists() {
            fs::create_dir_all(&path)?;
        }

        Ok(path)
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

    /// Retrieve Commits
    ///
    /// Commits sorted newest to oldest
    fn commits(
        &self,
        range: Option<&ScmCommitRange>,
        include_paths: Option<&Vec<Pattern>>,
        exclude_paths: Option<&Vec<Pattern>>,
        limit_commits: Option<usize>,
    ) -> ScmResult<Vec<ScmCommit>> {
        match self {
            Scm::Git(r) => r.commits(range, include_paths, exclude_paths, limit_commits),
            Scm::Hg(r) => r.commits(range, include_paths, exclude_paths, limit_commits),
            Scm::Svn(r) => r.commits(range, include_paths, exclude_paths, limit_commits),
        }
    }

    fn tags(
        &self,
        include: Option<&Regex>,
        exclude: Option<&Regex>,
        sort: TagSort,
        suffix_order: Option<&Vec<String>>,
    ) -> ScmResult<IndexMap<String, ScmTag>> {
        match self {
            Scm::Git(r) => r.tags(include, exclude, sort, suffix_order),
            Scm::Hg(r) => r.tags(include, exclude, sort, suffix_order),
            Scm::Svn(r) => r.tags(include, exclude, sort, suffix_order),
        }
    }

    fn current_tag(&self) -> Option<ScmTag> {
        match self {
            Scm::Git(r) => r.current_tag(),
            Scm::Hg(r) => r.current_tag(),
            Scm::Svn(r) => r.current_tag(),
        }
    }

    fn latest_tag(&self) -> ScmResult<Option<ScmTag>> {
        match self {
            Scm::Git(r) => r.latest_tag(),
            Scm::Hg(r) => r.latest_tag(),
            Scm::Svn(r) => r.latest_tag(),
        }
    }

    fn get_tag(&self, name: &str) -> ScmTag {
        match self {
            Scm::Git(r) => r.get_tag(name),
            Scm::Hg(r) => r.get_tag(name),
            Scm::Svn(r) => r.get_tag(name),
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

    fn is_hook_file_sample(&self, path: &Path) -> bool {
        match self {
            Scm::Git(r) => r.is_hook_file_sample(path),
            Scm::Hg(r) => r.is_hook_file_sample(path),
            Scm::Svn(r) => r.is_hook_file_sample(path),
        }
    }

    fn info_path(&self) -> ScmResult<PathBuf> {
        match self {
            Scm::Git(r) => r.info_path(),
            Scm::Hg(r) => r.info_path(),
            Scm::Svn(r) => r.info_path(),
        }
    }

    fn all_files(&self) -> ScmResult<Vec<PathBuf>> {
        match self {
            Scm::Git(r) => r.all_files(),
            Scm::Hg(r) => r.all_files(),
            Scm::Svn(r) => r.all_files(),
        }
    }

    fn staged_files(&self) -> ScmResult<Vec<PathBuf>> {
        match self {
            Scm::Git(r) => r.staged_files(),
            Scm::Hg(r) => r.staged_files(),
            Scm::Svn(r) => r.staged_files(),
        }
    }

    fn push_files(&self) -> ScmResult<Vec<PathBuf>> {
        match self {
            Scm::Git(r) => r.push_files(),
            Scm::Hg(r) => r.push_files(),
            Scm::Svn(r) => r.push_files(),
        }
    }

    fn files_by_command(&self, cmd: &String) -> ScmResult<Vec<PathBuf>> {
        match self {
            Scm::Git(r) => r.files_by_command(cmd),
            Scm::Hg(r) => r.files_by_command(cmd),
            Scm::Svn(r) => r.files_by_command(cmd),
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
