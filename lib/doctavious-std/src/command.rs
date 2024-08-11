use std::ffi::OsStr;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Write};
use std::path::Path;
use std::process::{Command, Output};
use std::thread;

use tracing::error;

use crate::error::Result;

/// Runs the given program and returns the output as string.
pub fn run_program(
    program: &str,
    args: &str,
    root: Option<&Path>,
    envs: Vec<(&str, &str)>,
) -> Result<String> {
    let mut command = Command::new(program);
    command.arg(args).envs(envs);
    if let Some(cwd) = root {
        command.current_dir(cwd);
    }

    let output = command.output()?;
    handle_output(output)
}

pub fn run_program_with_args<I, S>(
    program: &str,
    args: I,
    root: Option<&Path>,
    envs: Vec<(&str, &str)>,
) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args).envs(envs);
    if let Some(cwd) = root {
        command.current_dir(cwd);
    }

    let output = command.output()?;
    handle_output(output)
}

/// Runs the given OS command and returns the output as string.
///
/// Use `input` parameter to specify a text to write to stdin.
/// Environment variables are set accordingly to `envs`.
pub fn run(
    cmd: &str,
    input: Option<String>,
    root: &Path,
    envs: Vec<(&str, &str)>,
) -> Result<String> {
    let mut child = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .envs(envs)
            .current_dir(root)
            .args(["/C", cmd])
            .spawn()
    } else {
        Command::new("sh")
            .envs(envs)
            .current_dir(root)
            .args(["-c", cmd])
            .spawn()
    }?;

    if let Some(input) = input {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| IoError::new(IoErrorKind::Other, "stdin is not captured"))?;
        thread::spawn(move || {
            stdin
                .write_all(input.as_bytes())
                .expect("Failed to write to stdin");
        });
    }

    let output = child.wait_with_output()?;
    handle_output(output)
}

fn handle_output(output: Output) -> Result<String> {
    if output.status.success() {
        Ok(std::str::from_utf8(&output.stdout)?.to_string())
    } else {
        for output in [output.stdout, output.stderr] {
            let output = std::str::from_utf8(&output)?.to_string();
            if !output.is_empty() {
                error!("{}", output);
            }
        }

        Err(IoError::new(
            IoErrorKind::Other,
            format!("command exited with {:?}", output.status),
        )
        .into())
    }
}
