use std::path::PathBuf;
use std::process::Command;

use regex::{Error, Regex, RegexBuilder};
use scm::drivers::Scm;
use scm::hooks::{ScmHook, ScmHookConditionalExecution};
use tracing::warn;

// TODO: this should move to CLI

struct ScmRunnerOptions<'a> {
    hook: ScmHook,
    hook_name: String,
    force: bool,
    files: Vec<PathBuf>,
    run_only_commands: Vec<String>,
    scm: &'a Scm,
}

struct ScmRunner<'a> {
    pub options: ScmRunnerOptions<'a>,
}

impl<'a> ScmRunner<'a> {
    pub fn new(options: ScmRunnerOptions<'a>) -> Self {
        Self { options }
    }

    pub fn run_all(&self) {
        // this probably should look at run_only_commands as well?
        if self.should_skip(&self.options.hook) {
            return;
        }

        // run scripts
        // run commands
    }

    fn should_skip(&self, hook: &ScmHook) -> bool {
        ExecutionChecker::check(&hook.skip, &hook.only)
    }
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
