use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::cmd::githooks::Hook;
use crate::file_structure::FileStructure;
use crate::markup_format::MarkupFormat;
use crate::CliResult;

// TODO: better way to do this? Do we want to keep a default settings file in doctavious dir?
pub const DEFAULT_CONFIG_NAME: &str = "doctavious.toml";
pub const DEFAULT_ADR_DIR: &str = "docs/adr";
pub const DEFAULT_ADR_TEMPLATE_PATH: &str = "templates/adr/template";
pub const INIT_ADR_TEMPLATE_PATH: &str = "templates/adr/init";
pub const DEFAULT_RFD_DIR: &str = "docs/rfd";
pub const DEFAULT_RFD_TEMPLATE_PATH: &str = "templates/rfd/template";
// TODO: do we want this to default to the current directory?
pub const DEFAULT_TIL_DIR: &str = "til";
pub const DEFAULT_TIL_TEMPLATE_PATH: &str = "templates/til/template";

// This is primary went to avoid having to re-parse settings but an alternative might be to have
// a context object that gets passed around. We might do down that route anyway when we introduce
// an HTTP client to talk to doctavious API
lazy_static! {
    // TODO: doctavious config will live in project directory
    // do we also want a default settings file
    pub static ref SETTINGS_FILE: PathBuf = PathBuf::from(DEFAULT_CONFIG_NAME);

    // TODO: not sure this buys us anything.
    // just have a parse method on Settings that takes in a string/pathbuf?
    pub static ref SETTINGS: Settings = {
        load_settings().unwrap_or_else(|e| {
                if Path::new(SETTINGS_FILE.as_path()).exists() {
                    eprintln!(
                        "Error when parsing {}, fallback to default settings. Error: {:?}\n",
                        SETTINGS_FILE.as_path().display(),
                        e
                    );
                }
                Default::default()
            })
    };
}

// TODO: should this include output?
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub template_extension: Option<MarkupFormat>,

    #[serde(rename(serialize = "adr"))]
    #[serde(alias = "adr")]
    pub adr_settings: Option<AdrSettings>,

    #[serde(rename(serialize = "build"))]
    #[serde(alias = "build")]
    pub build_settings: Option<BuildSettings>,

    #[serde(rename(serialize = "githook"))]
    #[serde(alias = "githook")]
    pub githook_settings: Option<GithookSettings>,

    #[serde(rename(serialize = "rfd"))]
    #[serde(alias = "rfd")]
    pub rfd_settings: Option<RFDSettings>,

    #[serde(rename(serialize = "til"))]
    #[serde(alias = "til")]
    pub til_settings: Option<TilSettings>,
    // TODO: snippets
    // TODO: changelog
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdrSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<MarkupFormat>,
    // TODO: custom date format
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_extension: Option<MarkupFormat>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    pub dir: Option<String>,
    pub template_extension: Option<MarkupFormat>,
    // TODO: custom template either as a string here or file
    // output_directory
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GithookSettings {
    pub hooks: HashMap<String, Hook>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BuildSettings {
    pub command: String,

    // also allow for env var ex: SKIP_INSTALL_DEPS
    pub skip_install: bool,
    // build_command
    // framework - This value overrides the Framework in Project Settings.
    // ignore_build_command
    // install_command
    // output_directory
}

impl Settings {
    pub fn get_adr_dir(&self) -> &str {
        if let Some(settings) = &self.adr_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_ADR_DIR;
    }

    pub fn get_adr_structure(&self) -> FileStructure {
        if let Some(settings) = &self.adr_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }

        return FileStructure::default();
    }

    pub fn get_adr_template_extension(&self, extension: Option<MarkupFormat>) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.adr_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }

    pub fn get_rfd_dir(&self) -> &str {
        if let Some(settings) = &self.rfd_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_ADR_DIR;
    }

    pub fn get_rfd_structure(&self) -> FileStructure {
        if let Some(settings) = &self.rfd_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }

        return FileStructure::default();
    }

    pub fn get_rfd_template_extension(&self, extension: Option<MarkupFormat>) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.rfd_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }

    pub fn get_til_dir(&self) -> &str {
        if let Some(settings) = &self.til_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_TIL_DIR;
    }

    // TODO: I might revert having this take in an extension and rather just have a function in til
    // that does and defers to settings
    pub fn get_til_template_extension(&self, extension: Option<MarkupFormat>) -> MarkupFormat {
        if extension.is_some() {
            return extension.unwrap();
        }

        if let Some(settings) = &self.til_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }

        return MarkupFormat::default();
    }
}

pub(crate) fn load_settings() -> CliResult<Settings> {
    let contents = fs::read_to_string(SETTINGS_FILE.as_path())?;
    let settings: Settings = toml::from_str(contents.as_str())?;
    Ok(settings)
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
// TODO: should this take in a mut writer, i.e., a mutable thing we call “writer”.
// Its type is impl std::io::Write
// so that its a bit easier to test?
pub(crate) fn persist_settings(settings: Settings) -> CliResult<()> {
    let content = toml::to_string(&settings)?;
    fs::write(SETTINGS_FILE.as_path(), content)?;
    Ok(())
}

// TODO: where to put this?
pub fn init_dir(dir: &str) -> CliResult<()> {
    // TODO: create_dir_all doesnt appear to throw AlreadyExists. Confirm this
    // I think this is fine just need to make sure that we dont overwrite initial file
    println!("{}", format!("creating dir {}", dir));
    let create_dir_result = fs::create_dir_all(dir);
    match create_dir_result {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            eprintln!("the directory {} already exists", dir);
            return Err(e.into());
        }
        Err(e) => {
            eprintln!("Error occurred creating directory {}: {}", dir, e);
            return Err(e.into());
        }
    }
}
