use std::collections::HashMap;

use anyhow::Result;
use doctavious_cli::enums::{EnumError, parse_enum};
use lazy_static::lazy_static;
use serde::Serialize;

lazy_static! {
    static ref OUTPUT_TYPES: HashMap<&'static str, Output> = {
        let mut map = HashMap::new();
        map.insert("json", Output::Json);
        map.insert("table", Output::Table);
        map.insert("text", Output::Text);
        map
    };
}

// TODO:
// should text be the following?
// The text format organizes the CLI output into tab-delimited lines.
// It works well with traditional Unix text tools such as grep, sed, and awk, and the text processing performed by PowerShell.
// The text output format follows the basic structure shown below.
// The columns are sorted alphabetically by the corresponding key names of the underlying JSON object.
// What about table?
// The table format produces human-readable representations of complex CLI output in a tabular form.
#[remain::sorted]
#[derive(Debug, Copy, Clone)]
pub enum Output {
    /// JSON is the default output format of the Doctavious CLI.
    Json,

    /// Produces human-readable representations of complex Doctavious CLI output in a tabular form.
    Table,

    /// The text format organizes the Doctavious CLI output into tab-delimited lines.
    /// It works well with traditional Unix text tools such as grep, sed, and awk, and PowerShell.
    Text,
}

impl Default for Output {
    fn default() -> Self {
        Output::Json
    }
}

pub(crate) fn parse_output(src: &str) -> Result<Output, EnumError> {
    parse_enum(&OUTPUT_TYPES, src)
}

// TODO: I dont thin this will work for generating Tables as I dont see a way to make the implementation
// generic over any struct. Could use `tabled` instead if we really wanted to go this direction
pub(crate) fn print_output<A: std::fmt::Display + Serialize>(
    output: Output,
    value: A,
) -> Result<()> {
    match output {
        Output::Json => {
            serde_json::to_writer_pretty(std::io::stdout(), &value)?;
            Ok(())
        }
        Output::Text => {
            println!("{value}");
            Ok(())
        }
        Output::Table => {
            // TODO: comfy-table could be used here
            todo!()
        }
    }
}

// /// get output based on following order of precednece
// /// output argument (--output)
// /// env var DOCTAVIOUS_DEFAULT_OUTPUT
// /// config file overrides output default -- TODO: implement
// /// Output default
// pub(crate) fn get_output(opt_output: Option<Output>) -> Output {
//     match opt_output {
//         Some(o) => o,
//         None => {
//             match env::var("DOCTAVIOUS_DEFAULT_OUTPUT") {
//                 Ok(val) => parse_output(&val).unwrap(), // TODO: is unwrap ok here?
//                 Err(_) => Output::default(), // TODO: implement output via settings/config file
//             }
//         }
//     }
// }
