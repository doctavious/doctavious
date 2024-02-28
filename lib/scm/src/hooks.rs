use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

use crate::ScmResult;

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

// idea from rusty-hook and left-hook
// TODO: flush this out more
// add hook
// execute hook

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScmHook {
    pub name: String,
    pub parallel: bool,
    pub piped: bool, // If any command in the sequence fails, the other will not be executed.
    pub glob: String,
    pub exclude: String, //regex
    pub root: String,    // execute in a sub directory "api/" # Careful to have only trailing slash
    pub commands: Vec<HookCommand>,
}

// If one line commands are not enough, you can execute files..
// https://github.com/evilmartians/lefthook/blob/master/docs/full_guide.md#bash-script-example
struct Script {
    // used as file name
    pub name: String,
    pub runner: String,
}

// select specific file groups
// There are two shorthands for such situations: {staged_files} - staged git files which you try to commit
// {all_files} - all tracked files by git

// custom file list
// files: git diff --name-only master # custom list of files

// Git hook argument shorthands in commands
// If you want to use the original Git hook arguments in a command you can do it using the indexed shorthands:
// commit-msg:
//  commands:
//      multiple-sign-off:
//      run: 'test $(grep -c "^Signed-off-by: " {1}) -lt 2'
// {0} - shorthand for the single space-joint string of Git hook arguments
// {i} - shorthand for the i-th Git hook argument

// You can skip commands by skip option:
// skip: true
// Skipping commands during rebase or merge
// skip: merge
// skip:
//   - merge
//   - rebase
// You can skip commands by tags:
// pre-push:
//   exclude_tags:
//     - frontend

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HookCommand {
    pub name: String,
    pub tags: Vec<String>,
    pub glob: String, // Use glob patterns to choose what files you want to check
    pub run: String,
}

pub const FILE_MODE: &'static str = "755";
pub const OLD_HOOK_POSTFIX: &'static str = ".old";

// TODO: Some of these things probably belong in the CLI.
// Example the template might be good to put there as it needs to know the various ways doctavious can be installed

pub trait ScmHookSupport {
    fn get_supported(&self) -> Vec<&'static str>;

    fn supports<S: AsRef<str>>(&self, hook: S) -> bool;

    fn hook_path(&self) -> ScmResult<PathBuf>;

    // not sure if we should have an install method or just a way to get a specific file for the hook
    // which the CLI can write to
}

pub fn install() -> ScmResult<()> {
    // get config or return error
    // ensure hooks directory exists (get from SCM)
    // iterate over hooks in configuration
    //

    Ok(())
}

pub fn add_hook() -> ScmResult<()> {
    // write
    //      fs::set_permissions("/path", fs::Permissions::from_mode(0o655)).unwrap();

    // fs::OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .mode(0o770)
    //     .open("somefile")
    //     .unwrap();

    Ok(())
}

pub fn synchronize() -> ScmResult<()> {
    Ok(())
}
