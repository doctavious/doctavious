use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::cmd::githooks::Hook;
use crate::file_structure::FileStructure;
use crate::markup_format::MarkupFormat;
use crate::{CliResult, DoctaviousCliError};

// TODO: better way to do this? Do we want to keep a default settings file in doctavious dir?
pub const DEFAULT_CONFIG_NAME: &str = "doctavious.toml";

pub const DOCTAVIOUS_ENV_SETTINGS_PATH: &str = "DOCTAVIOUS_CONFIG_PATH";

pub const DEFAULT_ADR_DIR: &str = "docs/adr";

// TODO: could use const_format formatcp
pub const DEFAULT_TEMPLATE_DIR: &str = "templates";

pub const DEFAULT_ADR_INIT_TEMPLATE_PATH: &str = "templates/adr/init";
pub const DEFAULT_ADR_RECORD_TEMPLATE_PATH: &str = "templates/adr/record";
pub const DEFAULT_ADR_TOC_TEMPLATE_PATH: &str = "templates/adr/toc";

pub const DEFAULT_RFD_DIR: &str = "docs/rfd";
pub const DEFAULT_RFD_RECORD_TEMPLATE_PATH: &str = "templates/rfd/record";
pub const DEFAULT_RFD_TOC_TEMPLATE_PATH: &str = "templates/rfd/toc";
// TODO: do we want this to default to the current directory?
pub const DEFAULT_TIL_DIR: &str = "til";

pub const DEFAULT_TIL_README_TEMPLATE_PATH: &str = "templates/til/readme";
pub const DEFAULT_TIL_POST_TEMPLATE_PATH: &str = "templates/til/post";

use std::sync::OnceLock;

#[cfg(not(test))]
static SETTINGS: OnceLock<Option<Settings>> = OnceLock::new();

// This is primary went to avoid having to re-parse settings but an alternative might be to have
// a context object that gets passed around. We might do down that route anyway when we introduce
// an HTTP client to talk to doctavious API
lazy_static! {
    pub static ref SETTINGS_FILE: PathBuf = PathBuf::from(DEFAULT_CONFIG_NAME);

    // TODO: not sure this buys us anything.
    // just have a parse method on Settings that takes in a string/pathbuf?
    // pub static ref SETTINGS: Settings = {
    //     load_settings().unwrap_or_else(|e| {
    //             if Path::new(SETTINGS_FILE.as_path()).exists() {
    //                 eprintln!(
    //                     "Error when parsing {}, fallback to default settings. Error: {:?}\n",
    //                     SETTINGS_FILE.as_path().display(),
    //                     e
    //                 );
    //             }
    //             Default::default()
    //         })
    // };
}

// TODO: should this include output?
// TODO: should this be aware of CWD?
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[skip_serializing_none]
pub struct Settings {
    // TODO: I dont think this is needed
    pub template_format: Option<MarkupFormat>,

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
    // #[serde(default = "DEFAULT_ADR_DIR")]
    pub dir: Option<String>,

    #[serde(default)]
    pub structure: FileStructure,

    #[serde(default)]
    pub template_format: MarkupFormat,
    // TODO: custom date format
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    pub dir: Option<String>,
    pub structure: Option<FileStructure>,
    pub template_format: Option<MarkupFormat>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    pub dir: Option<String>,
    pub template_format: Option<MarkupFormat>,
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

    pub fn get_adr_record_template(&self) -> PathBuf {
        self.get_adr_template_path("record")
    }

    pub fn get_adr_init_template(&self) -> PathBuf {
        self.get_adr_template_path("init")
    }

    fn get_adr_template_path(&self, template_type: &str) -> PathBuf {
        let dir = self.get_adr_dir();

        // see if directory defines a custom template
        let custom_template = Path::new(dir)
            .join("templates")
            .join(template_type)
            .with_extension(self.get_adr_template_format(None).extension());

        if custom_template.is_file() {
            custom_template
        } else {
            self.get_adr_default_template()
        }
    }

    #[cfg(test)]
    pub fn get_adr_default_template(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}.{}",
            DEFAULT_ADR_RECORD_TEMPLATE_PATH,
            self.get_adr_template_format(None)
        ))
    }

    #[cfg(not(test))]
    pub fn get_adr_default_template(&self) -> PathBuf {
        // TODO: path from <home dir>/doctavious/templates/adr/
        unimplemented!()
    }

    #[cfg(test)]
    pub fn get_adr_default_init_template(&self) -> PathBuf {
        PathBuf::from(format!(
            "{DEFAULT_ADR_INIT_TEMPLATE_PATH}.{}",
            self.get_adr_template_format(None)
        ))
    }

    #[cfg(not(test))]
    pub fn get_adr_default_init_template(&self) -> PathBuf {
        // TODO: path from <home dir>/doctavious/templates/adr/
        unimplemented!()
    }

    pub fn get_adr_structure(&self) -> FileStructure {
        if let Some(settings) = &self.adr_settings {
            settings.structure
        } else {
            FileStructure::default()
        }
    }

    // TODO: Might want to split these some of this function up as we end up passing in None just to
    // get to the middle portion
    pub fn get_adr_template_format(&self, format: Option<MarkupFormat>) -> MarkupFormat {
        if let Some(format) = format {
            return format;
        }

        if let Some(settings) = &self.adr_settings {
            return settings.template_format;
        }

        if let Some(template_format) = self.template_format {
            return template_format;
        }

        MarkupFormat::default()
    }

    pub fn get_rfd_dir(&self) -> &str {
        if let Some(settings) = &self.rfd_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        return DEFAULT_ADR_DIR;
    }

    #[cfg(test)]
    pub fn get_rfd_default_template(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}.{}",
            DEFAULT_RFD_RECORD_TEMPLATE_PATH,
            self.get_rfd_template_extension(None)
        ))
    }

    #[cfg(not(test))]
    pub fn get_rfd_default_template(&self) -> PathBuf {
        // TODO: path from <home dir>/doctavious/templates/adr/
        unimplemented!()
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
            if let Some(template_extension) = settings.template_format {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_format {
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
            if let Some(template_extension) = settings.template_format {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_format {
            return template_extension;
        }

        return MarkupFormat::default();
    }

    // TODO: might need a test / not(test) versions of this
    pub fn get_til_default_template(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}/{}.{}",
            self.get_til_dir(),
            DEFAULT_TIL_POST_TEMPLATE_PATH,
            self.get_til_template_extension(None)
        ))
    }
}

pub(crate) fn get_settings_file() -> PathBuf {
    // I dont love having to use env vars here but dont seems like the most convenient way, for the time being,
    // to get around issues of tests writing to the same settings file. It might be better to just have most of the
    // functions take in a `cwd` arg but not sure that solves all the issues.
    match std::env::var_os(DOCTAVIOUS_ENV_SETTINGS_PATH) {
        None => SETTINGS_FILE.to_path_buf(),
        Some(path) => PathBuf::from(path).join(SETTINGS_FILE.to_path_buf()),
    }
}

#[cfg(not(test))]
pub(crate) fn load_settings<'a>() -> CliResult<Cow<'a, Settings>> {
    let settings = SETTINGS
        .get_or_init(|| {
            let settings_path = get_settings_file();
            if settings_path.is_file() {
                let contents = fs::read_to_string(settings_path).unwrap();
                match toml::from_str(contents.as_str()) {
                    Ok(s) => Some(s),
                    Err(_) => None,
                }
            } else {
                Some(Settings::default())
            }
        })
        .as_ref()
        .ok_or(DoctaviousCliError::InvalidSettingsFile)?;

    Ok(Cow::Borrowed(settings))
}

#[cfg(test)]
pub(crate) fn load_settings<'a>() -> CliResult<Cow<'a, Settings>> {
    let settings_path = get_settings_file();
    let settings = if settings_path.is_file() {
        let contents = fs::read_to_string(settings_path).unwrap();
        toml::from_str(contents.as_str())?
    } else {
        Settings::default()
    };

    Ok(Cow::Owned(settings))
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
// TODO: should this take in a mut writer, i.e., a mutable thing we call “writer”.
// Its type is impl std::io::Write
// so that its a bit easier to test?
pub(crate) fn persist_settings(settings: &Settings) -> CliResult<()> {
    let content = toml::to_string(&settings)?;
    let settings_file = get_settings_file();
    println!(
        "persisting settings file: {:?} with content {content}",
        settings_file
    );
    fs::write(get_settings_file(), content)?;
    Ok(())
}

pub fn init_dir(dir: &Path) -> CliResult<()> {
    let create_dir_result = fs::create_dir_all(dir);
    match create_dir_result {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            eprintln!("the directory {} already exists", dir.to_string_lossy());
            return Err(e.into());
        }
        Err(e) => {
            eprintln!(
                "Error occurred creating directory {}: {}",
                dir.to_string_lossy(),
                e
            );
            return Err(e.into());
        }
    }
}
