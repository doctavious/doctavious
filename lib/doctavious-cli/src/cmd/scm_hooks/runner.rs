use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::string::FromUtf8Error;
use std::{fs, io};

use regex::RegexBuilder;
use scm::drivers::Scm;
use scm::hooks::{HookCommand, HookScript, ScmHook, ScmHookConditionalExecution, ScmHookExecution};
use scm::{ScmError, ScmRepository};
use thiserror::Error;
use tracing::{debug, info, warn};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ScmHookRunnerError {
    #[error("Skipped: {0}")]
    Skip(String),
}

pub type ScmHookRunnerResult<T> = Result<T, ScmHookRunnerError>;

pub struct ScmHookRunnerOutcome {
    pub name: String,
    pub status: ScmHookRunnerStatus,
    pub text: Option<String>,
}

pub enum ScmHookRunnerStatus {
    Failure,
    Skip,
    Success,
}

impl ScmHookRunnerOutcome {
    pub fn success(&self) -> bool {
        matches!(self.status, ScmHookRunnerStatus::Success)
    }

    pub fn failure(&self) -> bool {
        matches!(self.status, ScmHookRunnerStatus::Failure)
    }

    pub fn skipped(name: String) -> Self {
        Self {
            name,
            status: ScmHookRunnerStatus::Skip,
            text: None,
        }
    }

    pub fn succeeded(name: String) -> Self {
        Self {
            name,
            status: ScmHookRunnerStatus::Success,
            text: None,
        }
    }

    pub fn failed(name: String, text: String) -> Self {
        Self {
            name,
            status: ScmHookRunnerStatus::Failure,
            text: Some(text),
        }
    }
}

pub struct ScmHookRunnerOptions<'a> {
    pub scm: &'a Scm,
    pub hook: &'a ScmHook,
    pub hook_name: String,
    pub files: Vec<PathBuf>,
    pub run_only_executions: Vec<String>,
}

pub struct ScmHookRunner<'a> {
    pub options: ScmHookRunnerOptions<'a>,
}

impl<'a> ScmHookRunner<'a> {
    pub fn new(options: ScmHookRunnerOptions<'a>) -> Self {
        Self { options }
    }

    // TODO: probably need to pass in scripts dir
    pub fn run_all(&self) -> Vec<ScmHookRunnerResult<()>> {
        let mut results = Vec::new();
        if ExecutionChecker::check(&self.options.hook.skip, &self.options.hook.only) {
            // TODO: should logging skips be a flag?
            info!("{} skip hook setting", self.options.hook_name);
            return results;
        }

        results.extend(self.run_executions());

        results
    }

    fn run_executions(&self) -> Vec<ScmHookRunnerResult<()>> {
        let mut runnable_executions = Vec::new();
        for name in self.options.hook.executions.keys() {
            if self.options.run_only_executions.is_empty()
                || self.options.run_only_executions.contains(&name)
            {
                runnable_executions.push(name);
            }
        }

        // TODO: do we want to have a sort priority for commands?

        let mut results = Vec::new();
        for name in runnable_executions {
            if self.options.hook.parallel {
                // TODO: implement parallel processing
                unimplemented!()
            } else {
                match self.options.hook.executions.get(name) {
                    None => {
                        // shouldn't get here...
                        warn!("Unable to find execution {name}");
                    }
                    Some(execution) => {
                        results.push(match execution {
                            ScmHookExecution::Command(command) => self.run_command(command),
                            ScmHookExecution::Script(script) => {
                                let path = PathBuf::from(&self.options.hook_name).join(name);
                                self.run_script(script, &path)
                            }
                        });
                    }
                }
            }
        }

        results
    }

    // pub fn run_scripts(&self, dir: &Path) -> Vec<ScmHookRunnerResult<()>> {
    //     let mut results = Vec::new();
    //     // TODO: I feel like perhaps we should iterate hooks and warn if file doesnt exist
    //     // is there a reason to iterate the directory first?
    //     for entry in fs::read_dir(dir)?.flatten() {
    //         let file_name = entry.file_name().to_string_lossy().to_string();
    //         let Some(script) = self.options.hook.scripts.get(&file_name) else {
    //             // log
    //             continue;
    //         };
    //
    //         results.push(self.run_script(script, &entry.path()));
    //     }
    //
    //     results
    // }

    fn run_script(&self, script: &HookScript, path: &Path) -> ScmHookRunnerResult<()> {
        if let Err(error) = self.should_execute_script(script, path) {
            // TODO: log error
            match error {
                ScmHookRunnerError::Skip(_) => {
                    // return ScmHookRunnerOutcome.skipped(command.name);
                } // marked as failed
                  // return ScmHookRunnerOutcome.failed(command.name, err);
            }
        }

        Ok(())
    }

    fn build_run_script(&self, script: &HookScript) -> Result<(), ScmHookRunnerError> {
        Ok(())
    }

    fn should_execute_script(
        &self,
        script: &HookScript,
        path: &Path,
    ) -> Result<(), ScmHookRunnerError> {
        if ExecutionChecker::check(&script.skip, &script.only) {
            return Err(ScmHookRunnerError::Skip(String::from("settings")));
        }

        // TODO: convert to hashset?
        if let Some(exclude_tags) = &self.options.hook.exclude_tags {
            if exclude_tags.contains(&script.name) {
                return Err(ScmHookRunnerError::Skip(String::from("name")));
            }

            for tag in &script.tags {
                if exclude_tags.contains(tag) {
                    return Err(ScmHookRunnerError::Skip(String::from("tags")));
                }
            }
        }

        if !path.is_file() {
            debug!("Skipping file: {path:?} is not a file");
            return Err(ScmHookRunnerError::Skip(String::from("not a file")));
        }

        // determine if executable -- only for linux
        // will probably need to extract validation into method and have platform specific implementations

        Ok(())
    }

    // pub fn run_commands(&self) -> Vec<ScmHookRunnerResult<()>> {
    //     let mut runnable_commands = Vec::new();
    //     for command in &self.options.hook.commands {
    //         if self.options.run_only_commands.is_empty() ||  self.options.run_only_commands.contains(&command.name) {
    //             runnable_commands.push(command);
    //         }
    //     }
    //
    //     // TODO: do we want to have a sort priority for commands?
    //     let mut results = Vec::new();
    //     for command in runnable_commands {
    //         if self.options.hook.parallel {
    //             // TODO: implement parallel processing
    //             unimplemented!()
    //         } else {
    //             results.push(self.run_command(command));
    //         }
    //     }
    //
    //     results
    // }

    fn run_command(&self, command: &HookCommand) -> ScmHookRunnerResult<()> {
        if let Err(error) = self.should_execute_command(command) {
            // TODO: log error
            match error {
                ScmHookRunnerError::Skip(_) => {
                    // return ScmHookRunnerOutcome.skipped(command.name);
                } // marked as failed
                  // return ScmHookRunnerOutcome.failed(command.name, err);
            }
        }

        // TODO: build run command
        // - get appropriate file template vars
        // - get files and apply any necessary filters
        // - swap template variables with files

        Ok(())
    }

    fn should_execute_command(&self, command: &HookCommand) -> Result<(), ScmHookRunnerError> {
        if ExecutionChecker::check(&command.skip, &command.only) {
            return Err(ScmHookRunnerError::Skip(String::from("settings")));
        }

        // TODO: convert to hashset?
        if let Some(exclude_tags) = &self.options.hook.exclude_tags {
            if exclude_tags.contains(&command.name) {
                return Err(ScmHookRunnerError::Skip(String::from("name")));
            }

            for tag in &command.tags {
                if exclude_tags.contains(tag) {
                    return Err(ScmHookRunnerError::Skip(String::from("tags")));
                }
            }
        }

        Ok(())
    }

    fn build_run_command(&self, command: &HookCommand) -> Result<(), ScmHookRunnerError> {
        Ok(())
    }

    pub fn run(&self) {}
}

struct ExecutionChecker;

impl ExecutionChecker {
    pub fn check(
        // &self,
        skip: &Option<ScmHookConditionalExecution>,
        only: &Option<ScmHookConditionalExecution>,
    ) -> bool {
        if let Some(skip) = skip {
            if Self::matches(skip) {
                return true;
            }
        }

        if let Some(only) = only {
            return !Self::matches(only);
        }

        false
    }

    fn matches(
        // &self,
        condition: &ScmHookConditionalExecution,
    ) -> bool {
        match condition {
            ScmHookConditionalExecution::Bool(b) => *b,
            ScmHookConditionalExecution::Ref(refs) => {
                for r in refs {
                    match RegexBuilder::new(&r).build() {
                        Ok(reg) => {
                            // if reg.is_match()
                        }
                        Err(e) => {
                            warn!("Invalid hook condition {r}: {e}")
                        }
                    }
                }
                todo!()
            }
            ScmHookConditionalExecution::Run(r) => {
                for s in r {
                    match s.split_once(" ") {
                        None => warn!("Invalid hook condition run command {s}"),
                        Some((cmd, args)) => {
                            match Command::new(cmd).args(args.split(" ")).output() {
                                Ok(output) => {
                                    // TODO: how to know if this resulted in true
                                    return true;
                                }
                                Err(e) => {
                                    warn!("Hook condition run command {s} failed: {e}")
                                }
                            }
                        }
                    }
                }
                todo!()
            }
            _ => {
                todo!()
            }
        }
    }
}
