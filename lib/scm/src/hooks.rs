use std::collections::HashMap;

use serde::de::{Error, MapAccess, Visitor};
use serde::{Deserialize as serde_deser, Deserializer};
use serde_derive::{Deserialize, Serialize};

// list of prior art
// - https://pre-commit.com/
// - https://www.npmjs.com/package/node-hooks
// - https://github.com/evilmartians/lefthook
// - https://github.com/sds/overcommit

// idea from rusty-hook and left-hook
// TODO: flush this out more
// add hook
// execute hook

// TODO: most of these things look like CLI things vs core SCM hooks

pub const FILE_MODE: &'static str = "755";
pub const OLD_HOOK_POSTFIX: &'static str = ".old";

#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ScmHookExecution {
    /// Commands to be executed for the hook. Each command has a name and associated run options.
    Command(HookCommand),

    /// Scripts to be executed for the hook.
    /// Each script has a name (filename in scripts dir) and associated run options.
    Script(HookScript),
}

// If one line commands are not enough, you can execute files..
// https://github.com/evilmartians/lefthook/blob/master/docs/full_guide.md#bash-script-example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookScript {
    pub file_name: String,
    pub runner: String,

    /// You can skip all or specific commands and scripts using skip option.
    /// You can also skip when merging, rebasing, or being on a specific branch.
    /// Globs are available for branches.
    pub skip: Option<ScmHookConditionalExecution>,

    /// You can force a command, script, or the whole hook to execute only in certain conditions.
    /// This option acts like the opposite of skip. It accepts the same values but skips execution
    /// only if the condition is not satisfied.
    pub only: Option<ScmHookConditionalExecution>,

    // TODO: could this also go in ScmHookConditionalExecution?
    /// You can specify tags for commands and scripts.
    /// This is useful for excluding. You can specify more than one tag using comma or space.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom text to show when the script fails.
    pub fail_text: Option<String>,
}

/// Holds the conditions used to determine if a hook or execution should be executed.
///
/// We want to support the following use cases for conditional executions:
/// 1. Provide a boolean
/// skip = true
///
/// 2. Provide a list of either refs or commands to run
/// ```toml
/// [skip]
/// ref = ["main"]
/// ```
///
/// Serde does not allow you to mix different tag representations for the same enum so in order to
/// achieve the above we needed to split the enum into two. This is the primary enum which supports
/// the boolean use case which leverages serde's `untagged` support.
/// The second, `ScmHookConditionalExecutionTagged`, nested in the `Tagged` variant allows us to
/// support the second use case leveraging serde's default representation called externally tagged
#[non_exhaustive]
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ScmHookConditionalExecution {
    Bool(bool),
    Tagged(ScmHookConditionalExecutionTagged),
}

#[non_exhaustive]
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmHookConditionalExecutionTagged {
    // TODO: Rebase / Merge -- for git others?
    Ref(Vec<String>), // scm refs / tags / etc
    Run(Vec<String>),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScmHook {
    /// A custom SCM command for files to be referenced in {files} template. See run and files.
    pub files: Option<String>,

    /// Run commands and scripts concurrently.
    #[serde(default)]
    pub parallel: bool,

    // If any command in the sequence fails, the other will not be executed.
    // should return an error if both piped: true and parallel: true are set.
    /// Stop running commands and scripts if one of them fail.
    #[serde(default)]
    pub stop_on_failure: bool,

    // could also be set as an env var
    /// Tags or command names that you want to exclude.
    pub exclude_tags: Option<Vec<String>>,

    pub skip: Option<ScmHookConditionalExecution>,

    pub only: Option<ScmHookConditionalExecution>,

    pub executions: HashMap<String, ScmHookExecution>,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HookCommand {
    /// This is a mandatory option for a command. This is actually a command that is executed for the hook.
    /// You can use files templates that will be substituted with the appropriate files on execution:
    /// {files} - custom files command result.
    /// {staged_files} - staged files which you try to commit.
    /// {push_files} - files that are committed but not pushed.
    /// {all_files} - all files tracked by git.
    pub run: String,

    // TODO: what type should this be? Probably enum if we are expecting different types
    /// You can skip all or specific commands and scripts using skip option.
    /// You can also skip when merging, rebasing, or being on a specific branch.
    /// Globs are available for branches.
    pub skip: Option<ScmHookConditionalExecution>,

    // TODO: optional fields
    /// You can force a command, script, or the whole hook to execute only in certain conditions.
    /// This option acts like the opposite of skip. It accepts the same values but skips execution
    /// only if the condition is not satisfied.
    pub only: Option<ScmHookConditionalExecution>,

    // TODO: could this also go in ScmHookConditionalExecution?
    /// You can specify tags for commands and scripts.
    /// This is useful for excluding. You can specify more than one tag using comma or space.
    #[serde(default)]
    pub tags: Vec<String>,

    // Use glob patterns to choose what files you want to check
    /// You can set a glob to filter files for your command.
    /// This is only used if you use a file template in run option or provide your custom files command.
    pub glob: Option<String>,

    /// A custom scm command for files or directories to be referenced in {files} template for run setting.
    /// If the result of this command is empty, the execution of commands will be skipped.
    /// This option overwrites the hook-level files option.
    pub files: Option<String>,

    /// You can specify some ENV variables for the command or script.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// You can change the CWD for the command you execute using root option.
    /// This is useful when you execute some npm or yarn command but the package.json is in another directory.
    /// For pre-push and pre-commit hooks and for the custom files command root option is used to
    /// filter file paths. If all files are filtered the command will be skipped.
    pub root: Option<String>,

    /// You can provide a regular expression to exclude some files from being passed to run command.
    /// The regular expression is matched against full paths to files in the repo, relative to the
    /// repo root, using / as the directory separator on all platforms. File paths do not begin
    /// with the separator or any other prefix.
    pub exclude: Option<String>,

    /// You can specify a text to show when the command or script fails.
    pub fail_text: Option<String>,
}
