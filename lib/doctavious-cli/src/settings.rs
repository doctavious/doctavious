use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use changelog::settings::{
    ChangelogBumpSettings, ChangelogCommitSettings, ChangelogOutput, ChangelogRemoteSettings,
    ChangelogSettings, TemplateSettings,
};
use directories::ProjectDirs;
use doctavious_std::command;
use indexmap::IndexMap;
use markup::MarkupFormat;
use regex::Regex;
use scm::drivers::git::TagSort;
use scm::hooks::ScmHook;
use scm::providers::ScmProviders;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use crate::errors::CliResult;
use crate::file_structure::FileStructure;

pub const DEFAULT_CONFIG_DIR: &str = ".doctavious";

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

pub const DEFAULT_TIL_DIR: &str = ".til";

pub const DEFAULT_TIL_TOC_TEMPLATE_PATH: &str = "templates/til/toc";
pub const DEFAULT_TIL_POST_TEMPLATE_PATH: &str = "templates/til/post";

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SettingErrors {
    #[error("{0} setting already initialized")]
    AlreadyInitialized(String),

    #[error("invalid settings file")]
    InvalidFile,

    #[error("{0} section not found")]
    SectionNotFound(String),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub path: PathBuf,
    pub settings: Settings,
    // TODO: I feel like we can get rid of this
    pub is_default_settings: bool,
}

impl Config {
    pub fn from_path(path: &Path) -> CliResult<Self> {
        let settings = get_settings(path)?;
        Ok(Self {
            path: path.join(Self::config_file_path()),
            settings,
            is_default_settings: false,
        })
    }

    pub fn from_path_or_default(path: &Path) -> Self {
        let mut is_default_settings = false;
        let settings = get_settings(path).unwrap_or_else(|_| {
            is_default_settings = true;
            Settings::default()
        });

        Self {
            path: path.join(Self::config_file_path()),
            settings,
            is_default_settings,
        }
    }

    pub fn get_global() -> CliResult<Self> {
        // let path = get_global_settings_dir().join(DEFAULT_CONFIG_NAME);
        let path = get_global_settings_file();
        let settings = get_settings(&path)?;

        Ok(Self {
            path,
            settings,
            is_default_settings: false,
        })
    }

    pub fn get_global_or_default() -> Self {
        let mut is_default_settings = false;
        let path = get_global_settings_file();
        let settings = get_settings(&path).unwrap_or_else(|_| {
            is_default_settings = true;
            Settings::default()
        });

        Self {
            path,
            settings,
            is_default_settings,
        }
    }

    pub fn save(&self) -> CliResult<()> {
        fs::create_dir_all(&self.path.parent().expect("Unable to get config directory"))?;
        fs::write(&self.path, toml::to_string(&self.settings)?)?;
        Ok(())
    }

    pub fn config_file_path() -> PathBuf {
        PathBuf::from(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_NAME)
    }
}

// TODO: should this include output?
// TODO: should this be aware of CWD?
// DoctaviousConfigurationFile
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    // TODO: I dont think this is needed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template_format: Option<MarkupFormat>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "adr"))]
    #[serde(alias = "adr")]
    pub adr_settings: Option<AdrSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "build"))]
    #[serde(alias = "build")]
    pub build_settings: Option<BuildSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<ChangelogSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "scmhook"))]
    #[serde(alias = "scmhook")]
    pub scmhook_settings: Option<ScmHookSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "rfd"))]
    #[serde(alias = "rfd")]
    pub rfd_settings: Option<RFDSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename(serialize = "til"))]
    #[serde(alias = "til")]
    pub til_settings: Option<TilSettings>,
    // TODO: snippets
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdrSettings {
    pub dir: Option<String>,

    #[serde(default)]
    pub structure: FileStructure,

    #[serde(default)]
    pub template_format: MarkupFormat,
    // TODO: custom date format
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChangelogConfiguration {
    // pub range: Option<ChangelogRange>,
    // pub include_paths: Option<Vec<Pattern>>,
    // pub exclude_paths: Option<Vec<Pattern>>,
    // pub commit_sort: ChangelogCommitSort,
    #[serde(flatten)]
    pub output: ChangelogOutput,
    pub templates: TemplateSettings,
    pub scm: ChangelogCommitSettings,
    pub remote: Option<ChangelogRemoteSettings>,
    pub bump: Option<ChangelogBumpSettings>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    pub dir: Option<String>,

    #[serde(default)]
    pub structure: FileStructure,

    #[serde(default)]
    pub template_format: MarkupFormat,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ScmHookSettings {
    pub hooks: IndexMap<String, ScmHook>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<PathBuf>,

    #[serde(default)]
    pub template_format: MarkupFormat,
}

impl Settings {
    pub fn get_adr_dir(&self) -> &str {
        if let Some(settings) = &self.adr_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }

        DEFAULT_ADR_DIR
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

        DEFAULT_RFD_DIR
    }

    #[cfg(test)]
    pub fn get_rfd_default_template(&self) -> PathBuf {
        PathBuf::from(format!(
            "{}.{}",
            DEFAULT_RFD_RECORD_TEMPLATE_PATH,
            self.get_rfd_template_format(None)
        ))
    }

    #[cfg(not(test))]
    pub fn get_rfd_default_template(&self) -> PathBuf {
        // TODO: path from <home dir>/doctavious/templates/adr/
        unimplemented!()
    }

    pub fn get_rfd_structure(&self) -> FileStructure {
        if let Some(settings) = &self.rfd_settings {
            settings.structure
        } else {
            FileStructure::default()
        }
    }

    pub fn get_rfd_template_format(&self, format: Option<MarkupFormat>) -> MarkupFormat {
        if let Some(format) = format {
            return format;
        }

        if let Some(settings) = &self.rfd_settings {
            return settings.template_format;
        }

        if let Some(template_extension) = self.template_format {
            return template_extension;
        }

        MarkupFormat::default()
    }

    pub fn get_til_dir(&self) -> Option<PathBuf> {
        self.til_settings.as_ref().and_then(|s| s.dir.to_owned())
    }

    // TODO: I might revert having this take in an extension and rather just have a function in til
    // that does and defers to settings
    pub fn get_til_template_format(&self, format: Option<MarkupFormat>) -> MarkupFormat {
        if let Some(format) = format {
            return format;
        }

        if let Some(settings) = &self.til_settings {
            return settings.template_format;
        }

        if let Some(template_extension) = self.template_format {
            return template_extension;
        }

        MarkupFormat::default()
    }

    // TODO: fix this
    // TODO: might need a test / not(test) versions of this
    // pub fn get_til_default_template(&self) -> PathBuf {
    //     self.get_til_dir().unwrap_or(&env::current_dir().expect("Unable to get current directory"))
    //         .join(DEFAULT_TIL_POST_TEMPLATE_PATH)
    //         .with_extension(self.get_til_template_format(None).extension())
    // }
}

pub(crate) fn get_settings_file(cwd: &Path) -> PathBuf {
    cwd.join(DEFAULT_CONFIG_DIR).join(DEFAULT_CONFIG_NAME)
}

pub fn load_settings(path: &Path) -> CliResult<Settings> {
    Ok(if path.is_file() {
        get_settings(path)?
    } else {
        let path = get_settings_file(path);
        if path.is_file() {
            get_settings(&path)?
        } else {
            warn!(
                "Unable to find Doctavious config file at {}. Using default settings",
                path.as_path().as_os_str().to_string_lossy()
            );
            Settings::default()
        }
    })
}

pub fn get_settings(path: &Path) -> CliResult<Settings> {
    let contents = fs::read_to_string(path)?;
    Ok(toml::from_str(contents.as_str())?)
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
// TODO: should this take in a mut writer, i.e., a mutable thing we call “writer”.
// Its type is impl std::io::Write
// so that its a bit easier to test?
pub(crate) fn persist_settings(cwd: &Path, settings: &Settings) -> CliResult<()> {
    let content = toml::to_string(&settings)?;
    let settings_file = get_settings_file(cwd);
    if let Some(dir) = settings_file.parent() {
        fs::create_dir_all(dir)?;
    }

    fs::write(settings_file, content)?;
    Ok(())
}

pub fn get_global_settings_dir() -> PathBuf {
    ProjectDirs::from("com", "doctavious", "cli")
        .expect("Unable to get valid Doctavious global config directory")
        .config_dir()
        .to_path_buf()
}

pub(crate) fn get_global_settings_file() -> PathBuf {
    let file_name = if cfg!(test) {
        "doctavious-test.toml"
    } else {
        DEFAULT_CONFIG_NAME
    };
    get_global_settings_dir().join(file_name)
}

pub fn get_global_settings() -> CliResult<Settings> {
    let settings_dir = get_global_settings_dir();
    Ok(get_settings(&settings_dir)?)
}

pub fn init_dir(dir: &Path) -> CliResult<()> {
    let create_dir_result = fs::create_dir_all(dir);
    match create_dir_result {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            eprintln!("the directory {} already exists", dir.to_string_lossy());
            Err(e.into())
        }
        Err(e) => {
            eprintln!(
                "Error occurred creating directory {}: {}",
                dir.to_string_lossy(),
                e
            );
            Err(e.into())
        }
    }
}
