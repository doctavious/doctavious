use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use glob_match::glob_match;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use regex::{Regex, RegexBuilder};
use scm::drivers::{Scm, ScmRepository};
use scm::errors::ScmError;
use scm::hooks::{
    HookCommand, HookScript, ScmHook, ScmHookConditionalExecution,
    ScmHookConditionalExecutionTagged, ScmHookExecution,
};
use thiserror::Error;
use tracing::{debug, info, warn};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ScmHookRunnerError {
    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    ScmError(#[from] ScmError),

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

    pub fn failed(name: String, text: Option<String>) -> Self {
        Self {
            name,
            status: ScmHookRunnerStatus::Failure,
            text,
        }
    }
}

pub struct ScmHookRunnerOptions<'a> {
    pub cwd: &'a Path,
    // would like to pass this in however underlying git_repository is not marked as `Send` so unable
    // to given we support parallel execution.
    // pub scm: &'a Scm,
    pub hook: &'a ScmHook,
    pub hook_name: String,
    pub files: Vec<PathBuf>,
    pub run_only_executions: Vec<String>,
    pub force: bool,
}

pub struct ScmHookRunner<'a> {
    pub options: ScmHookRunnerOptions<'a>,
}

struct TemplateFiles {
    templates: HashMap<&'static str, String>,
}
impl<'a> TemplateFiles {
    fn from_files(command: &HookCommand, files: Vec<PathBuf>) -> ScmHookRunnerResult<Self> {
        let file_templates = Self::filter_map_files(command, files);
        // TODO: would be nice to avoid all these clones but dont know a better way at the moment
        Ok(Self::new(
            file_templates.clone(),
            file_templates.clone(),
            file_templates.clone(),
            file_templates.clone(),
        ))
    }

    fn from_scm(
        command: &HookCommand,
        scm: &Scm,
        custom_files_cmd: Option<String>,
    ) -> ScmHookRunnerResult<Self> {
        let staged = Self::filter_map_files(command, scm.staged_files()?);
        let push = Self::filter_map_files(command, scm.push_files()?);
        let all = Self::filter_map_files(command, scm.all_files()?);
        let files = if let Some(custom_files_cmd) = custom_files_cmd {
            Self::filter_map_files(command, scm.files_by_command(&custom_files_cmd)?)
        } else {
            String::new()
        };

        Ok(Self::new(staged, push, all, files))
    }

    fn new(staged: String, push: String, all: String, files: String) -> Self {
        Self {
            templates: HashMap::from([
                ("{staged_files}", staged),
                ("{push_files}", push),
                ("{all_files}", all),
                ("{files}", files),
            ]),
        }
    }

    fn filter_map_files(command: &HookCommand, files: Vec<PathBuf>) -> String {
        if files.is_empty() {
            return String::new();
        }

        // TODO: filter by_type

        files
            .into_iter()
            .filter(|file| {
                // filter by root
                if let Some(root) = &command.root {
                    if root.is_empty() {
                        return true;
                    }

                    return file.starts_with(root);
                }

                true
            })
            .filter(|file| {
                // filter by glob
                if let Some(glob) = &command.glob {
                    if glob.is_empty() {
                        return true;
                    }

                    return glob_match(glob, file.to_string_lossy().as_ref());
                }

                true
            })
            .filter(|file| {
                // filter by exclude
                if let Some(exclude) = &command.exclude {
                    if exclude.is_empty() {
                        return true;
                    }

                    return match Regex::new(exclude) {
                        Ok(regex) => regex.is_match(exclude),
                        Err(e) => {
                            // TODO: Log
                            false
                        }
                    };
                }

                true
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(" ")
    }
}

impl<'a> ScmHookRunner<'a> {
    pub fn new(options: ScmHookRunnerOptions<'a>) -> Self {
        Self { options }
    }

    // TODO: probably need to pass in scripts dir
    pub fn run_all(&self) -> Vec<ScmHookRunnerResult<ScmHookRunnerOutcome>> {
        return if self.options.force
            || ExecutionChecker::check(
                self.options.hook.skip.as_ref(),
                self.options.hook.only.as_ref(),
            ) {
            self.run_executions()
        } else {
            // TODO: should logging skips be a flag?
            info!("{} skip hook setting", self.options.hook_name);
            Vec::new()
        };
    }

    fn run_executions(&self) -> Vec<ScmHookRunnerResult<ScmHookRunnerOutcome>> {
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
        if self.options.hook.parallel {
            results.extend(
                runnable_executions
                    .into_par_iter()
                    .filter_map(|e| self.handle_execution(e))
                    .collect::<Vec<ScmHookRunnerResult<ScmHookRunnerOutcome>>>(),
            );
        } else {
            for name in runnable_executions {
                if let Some(result) = self.handle_execution(name) {
                    results.push(result);
                }
            }
        }

        results
    }

    fn handle_execution(&self, name: &String) -> Option<ScmHookRunnerResult<ScmHookRunnerOutcome>> {
        return match self.options.hook.executions.get(name) {
            None => {
                // shouldn't get here...
                warn!("Unable to find execution {name}");
                None
            }
            Some(execution) => {
                match execution {
                    ScmHookExecution::Command(command) => Some(self.run_command(name, command)),
                    ScmHookExecution::Script(script) => {
                        // TODO: constant for hooks directory
                        let path = PathBuf::from(self.options.cwd)
                            .join(".doctavious/scm_hooks")
                            .join(&self.options.hook_name)
                            .join(&script.file_name);
                        Some(self.run_script(name.as_str(), script, &path))
                    }
                }
            }
        };
    }

    fn run_script(
        &self,
        name: &str,
        script: &HookScript,
        path: &Path,
    ) -> ScmHookRunnerResult<ScmHookRunnerOutcome> {
        if let Err(error) = self.should_execute_script(name, script, path) {
            // TODO: log error
            return match error {
                ScmHookRunnerError::Skip(reason) => {
                    Ok(ScmHookRunnerOutcome::skipped(name.to_owned()))
                }
                _ => Ok(ScmHookRunnerOutcome::failed(
                    name.to_owned(),
                    Some(error.to_string()),
                )),
            };
        }

        let ok = self.run(
            format!("{} {}", script.runner, path.to_string_lossy()).as_str(),
            self.options.cwd,
        );

        if ok {
            Ok(ScmHookRunnerOutcome::succeeded(name.to_owned()))
        } else {
            Ok(ScmHookRunnerOutcome::failed(
                name.to_owned(),
                script.fail_text.to_owned(),
            ))
        }
    }

    fn should_execute_script(
        &self,
        name: &str,
        script: &HookScript,
        path: &Path,
    ) -> Result<(), ScmHookRunnerError> {
        if self.options.force || ExecutionChecker::check(script.skip.as_ref(), script.only.as_ref())
        {
            // TODO: convert to hashset?
            // TODO: is there a way to avoid the &Vec<String> and the &String for contains?
            if let Some(exclude_tags) = &self.options.hook.exclude_tags {
                if exclude_tags.contains(&name.to_string()) {
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

            return Ok(());
        }

        Err(ScmHookRunnerError::Skip(String::from("settings")))
    }

    fn run_command(
        &self,
        name: &str,
        command: &HookCommand,
    ) -> ScmHookRunnerResult<ScmHookRunnerOutcome> {
        if let Err(error) = self.should_execute_command(name, command) {
            // TODO: log error
            return match error {
                ScmHookRunnerError::Skip(reason) => {
                    Ok(ScmHookRunnerOutcome::skipped(name.to_owned()))
                }
                _ => Ok(ScmHookRunnerOutcome::failed(
                    name.to_owned(),
                    Some(error.to_string()),
                )),
            };
        }

        let runnable = self.build_run_command(command)?;
        let mut working_dir = self.options.cwd.to_path_buf();
        if let Some(root) = &command.root {
            working_dir = working_dir.join(root);
        }

        let ok = self.run(&runnable, &working_dir);
        if ok {
            Ok(ScmHookRunnerOutcome::succeeded(name.to_owned()))
        } else {
            Ok(ScmHookRunnerOutcome::failed(
                name.to_owned(),
                command.fail_text.to_owned(),
            ))
        }
    }

    fn should_execute_command(
        &self,
        name: &str,
        command: &HookCommand,
    ) -> Result<(), ScmHookRunnerError> {
        if self.options.force
            || ExecutionChecker::check(command.skip.as_ref(), command.only.as_ref())
        {
            // TODO: convert to hashset?
            if let Some(exclude_tags) = &self.options.hook.exclude_tags {
                if exclude_tags.contains(&name.to_string()) {
                    return Err(ScmHookRunnerError::Skip(String::from("name")));
                }

                for tag in &command.tags {
                    if exclude_tags.contains(tag) {
                        return Err(ScmHookRunnerError::Skip(String::from("tags")));
                    }
                }
            }

            return Ok(());
        }

        Err(ScmHookRunnerError::Skip(String::from("settings")))
    }

    fn build_run_command(&self, command: &HookCommand) -> Result<String, ScmHookRunnerError> {
        let template_files = if !self.options.files.is_empty() {
            TemplateFiles::from_files(command, self.options.files.to_owned())?
        } else {
            let mut files_cmd = command.files.clone().or(self.options.hook.files.clone());
            if let Some(cmd) = files_cmd {
                // replace positional args
                files_cmd = Some("".to_string());
            }
            let scm = Scm::get(self.options.cwd)?;
            TemplateFiles::from_scm(command, &scm, files_cmd)?
        };

        let mut run_string = command.run.clone();
        for (key, tmpl) in template_files.templates {
            if let Some(index) = run_string.find(key) {
                if tmpl.is_empty() {
                    return Err(ScmHookRunnerError::Skip(format!(
                        "no files found to substitute for {}",
                        key
                    )));
                } else {
                    run_string.replace_range(index..index + key.len(), &tmpl);
                }
            }
        }

        println!("run string: [{}]", run_string);

        Ok(run_string)
    }

    pub fn run(&self, runnable: &str, root: &Path) -> bool {
        let output = doctavious_std::command::run(runnable, None, root, vec![]);
        output.is_ok()
    }
}

struct RunnableCommand {
    program: String,
    args: Vec<String>,
}

struct ExecutionChecker;

impl ExecutionChecker {
    /// Check to see if execution should run
    ///
    /// return true if execution should run and false if execution should be skipped
    pub fn check(
        skip: Option<&ScmHookConditionalExecution>,
        only: Option<&ScmHookConditionalExecution>,
    ) -> bool {
        if let Some(skip) = skip {
            if Self::matches(skip) {
                return false;
            }
        }

        if let Some(only) = only {
            return Self::matches(only);
        }

        true
    }

    fn matches(condition: &ScmHookConditionalExecution) -> bool {
        match condition {
            ScmHookConditionalExecution::Bool(b) => *b,
            ScmHookConditionalExecution::Tagged(t) => {
                match t {
                    ScmHookConditionalExecutionTagged::Ref(refs) => {
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
                    ScmHookConditionalExecutionTagged::Run(r) => {
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
            _ => {
                todo!()
            }
        }
    }
}
