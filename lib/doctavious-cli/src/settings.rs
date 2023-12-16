use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::cmd::githooks::Hook;
use crate::file_structure::FileStructure;
use crate::markup_format::MarkupFormat;

// TODO: better way to do this? Do we want to keep a default settings file in doctavious dir?
pub static DEFAULT_CONFIG_NAME: &str = "doctavious.toml";
pub static DEFAULT_ADR_DIR: &str = "docs/adr";
pub static DEFAULT_ADR_TEMPLATE_PATH: &str = "templates/adr/template";
pub static INIT_ADR_TEMPLATE_PATH: &str = "templates/adr/init";
pub static DEFAULT_RFD_DIR: &str = "docs/rfd";
pub static DEFAULT_RFD_TEMPLATE_PATH: &str = "templates/rfd/template";
// TODO: do we want this to default to the current directory?
pub static DEFAULT_TIL_DIR: &str = "til";
pub static DEFAULT_TIL_TEMPLATE_PATH: &str = "templates/til/template";

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
