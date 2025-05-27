use std::process::Command;

use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum CliDocsError {

    #[error(transparent)]
    IoError(#[from] io::Error),

    #[error(transparent)]
    Utf8Error(#[from] FromUtf8Error),
}

pub type CliDocsResult<T> = Result<T, CliDocsError>;

// TODO: would again like to pick a different name
pub enum Framework {
    Clap,
    Click,
    CliKit,
    Docopt,
    Fire,
    Go,
    OClif,
    Unknown
}


fn generate() {

}

fn parse() {

}

// fn get_program_commands(program: &str) -> CliDocsResult<Vec<String>> {
//     let mut command = Command::new(program);
//     let output = command.args(["--help"]).output()?.stdout;
//     let help: Vec<String> = String::from_utf8(output)?;
//
//     // TODO: parse
//
//     Ok(())
// }

fn get_command_help(program: &str, cmd: &str) -> CliDocsResult<String> {
    let mut command = Command::new(program);
    let output = command.args([cmd, "--help"]).output()?.stdout;
    let help_text = String::from_utf8(output)?;
    Ok(help_text)
}