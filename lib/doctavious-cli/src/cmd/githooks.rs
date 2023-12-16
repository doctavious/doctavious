mod init;
mod install;
mod list;
mod run;
mod uninstall;

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

use serde::{Deserialize, Deserializer, Serialize, Serializer};

// idea from rusty-hook and left-hook
// TODO: flush this out more

// add hook
// execute hook

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHooks {
    hooks: Vec<Hook>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hook {
    name: String,
    parallel: bool,
    piped: bool, // If any command in the sequence fails, the other will not be executed.
    glob: String,
    exclude: String, //regex
    root: String,    // execute in a sub directory "api/" # Careful to have only trailing slash
    commands: Vec<HookCommand>,
}

// If one line commands are not enough, you can execute files..
// https://github.com/evilmartians/lefthook/blob/master/docs/full_guide.md#bash-script-example
struct Script {
    // used as file name
    name: String,
    runner: String,
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
    name: String,
    tags: Vec<String>,
    glob: String, //Use glob patterns to choose what files you want to check
    run: String,
}

fn init() {}
