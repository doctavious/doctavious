pub mod cmd;
mod edit;
pub mod enums;
pub mod file_structure;
mod files;
mod git;
pub mod markup_format;
pub mod settings;
mod templates;

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DoctaviousCliError {
    #[error("cas error: {0}")]
    CasError(#[from] cas::CasError),

    #[error("cifrs error: {0}")]
    CifrsError(#[from] cifrs::CifrsError),

    #[error("design doc directory already exists")]
    DesignDocDirectoryAlreadyExists,

    #[error("invalid design doc directory. Should be utf-8")]
    DesignDocDirectoryInvalid,

    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),

    #[error("Glob pattern error: `{0}`")]
    GlobPatternError(#[from] glob::PatternError),

    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    // TODO: fix this
    #[error("{0}")]
    NoConfirmation(String),

    #[error("regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Error that may occur while reserving ADR/RFD number.
    #[error("{0} has already been reserved")]
    ReservedNumberError(i32),

    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Enum parsing error: {0}")]
    StrumParseError(#[from] strum::ParseError),

    #[error("Creating a Context from a Value/Serialize requires it being a JSON object")]
    TemplateContextError(),

    /// Error that may occur while template operations such as parse and render.
    #[error("Template error: `{0}`")]
    TemplateError(#[from] minijinja::Error),

    /// Error that may occur while parsing the template.
    #[error("Template parse error:\n{0}")]
    TemplateParseError(String),

    /// Errors that may occur when deserializing types from TOML format.
    #[error("toml deserialize error: {0}")]
    TomlDeserialize(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("toml serialization error: `{0}`")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Unknown design document: {0}")]
    UnknownDesignDocument(String),

    #[error("Unknown markup extension for path: {0}")]
    UnknownMarkupExtension(String),

    #[error("walkdir error")]
    WalkdirError(#[from] walkdir::Error),
}

pub type CliResult<T> = Result<T, DoctaviousCliError>;

// TODO: PUT THESE SOMEWHERE BETTER!

// TODO: This is wrong for ADRs init as it doesnt look for a custom init template
// does it need to take in name?
/// If the ADR directory contains a file `templates/template.<format>`, use it as the template for the new ADR.
/// Otherwise a use the default template.
pub(crate) fn get_template(
    template_path: Option<PathBuf>,
    dir: &str,
    extension: &str,
    default_template: PathBuf,
) -> PathBuf {
    if let Some(template_path) = template_path {
        template_path
    } else {
        // see if direction defines a custom template
        let custom_template = Path::new(dir)
            .join("templates")
            .join("template")
            .with_extension(extension);

        if custom_template.is_file() {
            custom_template
        } else {
            default_template
        }
    }
}

pub(crate) fn get_template_content(
    template_path: PathBuf,
    dir: &str,
    extension: &str,
    default_template: PathBuf,
) -> String {
    let template_path = get_template(Some(template_path), dir, extension, default_template);
    // TODO: we shouldnt panic here
    fs::read_to_string(&template_path).expect(&format!(
        "failed to read file {}.",
        &template_path.to_string_lossy()
    ))
}
