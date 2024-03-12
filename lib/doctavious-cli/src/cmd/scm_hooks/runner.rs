use std::fs;
use std::path::Path;

use scm::drivers::Scm;
use scm::hooks::{HookCommand, ScmHook, Script};
use scm::{ScmRepository, ScmResult};

pub struct ScmHookRunnerOptions<'a> {
    pub scm: &'a Scm,
    pub hook: &'a ScmHook,
    pub hook_name: String,
    pub files: Vec<String>,
    pub run_only_commands: Vec<String>,
}

pub struct ScmHookRunner<'a> {
    pub options: ScmHookRunnerOptions<'a>,
}

impl<'a> ScmHookRunner<'a> {
    pub fn new(options: ScmHookRunnerOptions<'a>) -> Self {
        Self { options }
    }

    pub fn run_all(&self) {
        // TODO: check to see if we should skip hook

        // TODO: get script directories from source directories

        // TODO: for each script dir run script

        self.run_commands();
    }

    pub fn run_scripts(&self, dir: &Path) -> ScmResult<()> {
        for entry in fs::read_dir(dir)?.flatten() {
            let file_name = entry.file_name().to_string_lossy().to_string();
            let Some(script) = self.options.hook.scripts.get(&file_name) else {
                // log
                continue;
            };

            self.run_script(script, &entry.path());
        }

        Ok(())
    }

    pub fn run_script(&self, script: &Script, path: &Path) -> ScmResult<()> {
        // TODO: determine if should skip
        // TODO: determine if appropriate tag

        if !path.is_file() {
            // log
            // return error / return to skip
        }

        // determine if executable -- only for linux

        Ok(())
    }

    pub fn run_commands(&self) {
        for command in &self.options.hook.commands {
            self.run_command(command)
        }
    }

    pub fn run_command(&self, command: &HookCommand) {
        // TODO: determine if should skip
        // TODO: determine if appropriate tag
        // TODO: validate command
        // TODO: build run command
    }

    pub fn run(&self) {}
}
