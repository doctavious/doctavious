use std::borrow::Cow;
use std::fmt::Debug;
use std::fs;
use std::path::{Path, PathBuf};

use lazy_static::lazy_static;
use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::backends::LanguageBackends;
use crate::framework_detection::Detectable;
use crate::frameworks::antora::AntoraConfig;
use crate::frameworks::astro::AstroConfig;
use crate::frameworks::docfx::DocFxConfig;
use crate::frameworks::docusaurus_v2::DocusaurusConfig;
use crate::frameworks::eleventy::EleventyConfig;
use crate::frameworks::gatsby::GatsbyConfig;
use crate::frameworks::hexo::HexoConfig;
use crate::frameworks::hugo::HugoConfig;
use crate::frameworks::jekyll::JekyllConfig;
use crate::frameworks::mdbook::MDBookConfig;
use crate::frameworks::mkdocs::MKDocsConfig;
use crate::frameworks::nextjs::NextJSConfig;
use crate::frameworks::nuxt_v3::Nuxt3JSConfig;
use crate::frameworks::nuxtjs::NuxtJSConfig;
use crate::frameworks::sphinx::SphinxConfig;
use crate::frameworks::sveltekit::SvelteKitConfig;
use crate::frameworks::vitepress::VitePressConfig;
use crate::frameworks::vuepress::VuePressConfig;
use crate::js_module::parse_js_module;
use crate::projects::project_file::ProjectFile;
use crate::{CifrsError, CifrsResult};

pub mod antora;
pub mod astro;
pub mod docfx;
pub mod docusaurus_v2;
pub mod eleventy;
pub mod gatsby;
pub mod hexo;
pub mod hugo;
pub mod jekyll;
pub mod mdbook;
pub mod mkdocs;
pub mod nextjs;
pub mod nuxt_v3;
pub mod nuxtjs;
pub mod sphinx;
pub mod sveltekit;
pub mod vitepress;
pub mod vuepress;

pub const FRAMEWORKS_STR: &str = include_str!("frameworks.yaml");

lazy_static! {

    // TODO: probably doesnt need to be an owned type
    static ref FRAMEWORKS_LIST: Vec<FrameworkInfo> = serde_yaml::from_str::<Vec<FrameworkInfo>>(FRAMEWORKS_STR)
        .expect("frameworks.yaml should be deserializable");
}

pub fn get_all() -> Vec<FrameworkInfo> {
    FRAMEWORKS_LIST.to_vec()
}

// FrameworkDefinition
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkInfo {
    pub id: String,

    /// Name of the framework
    ///
    /// # Examples
    /// Next.js
    pub name: String,

    /// A URL to the official website of the framework
    ///
    /// # Examples
    /// https://nextjs.org
    pub website: String,

    /// List of potential config files
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub configs: Vec<PathBuf>,

    // TODO: this could be SoftwareFramework / SoftwarePlatform.
    // Potentially solves for the scenario of needed multiple languages such as C#/F#
    // I kinda like Replit's UPM LanguageBackend. Or Could use LanguageRuntime
    pub backend: LanguageBackends,

    // /// Detectors used to find out the framework
    pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,
}

impl FrameworkInfo {
    pub fn get_configuration(&self) -> Option<FrameworkConfigFile> {
        for config in &self.configs {
            if config.is_file() {
                // this seems generally ok but some additional thoughts
                // could be an configuration enum and then make one call to get_output_dir
                // could just do a Box<dyn ConfigurationFile> and then do single get_output_dir
                // could just have each implementation return a common ConfigurationFile struct
                // which has an output dir.
                // I like the idea of the last option but not yet sure of the shape
                let config = match self.id.as_str() {
                    "antora" => AntoraConfig::get_config(config),
                    "astro" => AstroConfig::get_config(config),
                    "docfx" => DocFxConfig::get_config(config),
                    "docusaurus-v2" => DocusaurusConfig::get_config(config),
                    "eleventy" => EleventyConfig::get_config(config),
                    "gatsby" => GatsbyConfig::get_config(config),
                    "hexo" => HexoConfig::get_config(config),
                    "hugo" => HugoConfig::get_config(config),
                    "jekyll" => JekyllConfig::get_config(config),
                    "mdbook" => MDBookConfig::get_config(config),
                    "mkdocs" => MKDocsConfig::get_config(config),
                    "nextjs" => NextJSConfig::get_config(config),
                    "nuxt-v2" => NuxtJSConfig::get_config(config),
                    "nuxt-v3" => Nuxt3JSConfig::get_config(config),
                    "sphinx" => SphinxConfig::get_config(config),
                    "sveltekit" => SvelteKitConfig::get_config(config),
                    "vitepress" => VitePressConfig::get_config(config),
                    "vuepress" => VuePressConfig::get_config(config),
                    _ => todo!(),
                };

                if config.is_ok() {
                    return config.ok();
                }
            }
        }

        None
    }

    pub fn get_output_dir<P: AsRef<Path>>(&self, _cwd: P) -> String {
        if let Some(config) = self.get_configuration() {
            if let Some(output_dir) = config.settings.output_dir {
                return output_dir;
            }
        }

        self.build.output_directory.to_owned()
    }
}

impl Detectable for FrameworkInfo {
    fn get_matching_strategy(&self) -> &FrameworkMatchingStrategy {
        &self.detection.matching_strategy
    }

    fn get_detectors(&self) -> &Vec<FrameworkDetectionItem> {
        &self.detection.detectors
    }

    fn get_project_files(&self) -> Cow<Vec<ProjectFile>> {
        Cow::Owned(self.backend.project_files().to_vec())
    }

    fn get_configuration_files(&self) -> &Vec<PathBuf> {
        &self.configs
    }
}

// TODO: rename to FrameworkDetection?
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkDetector {
    pub matching_strategy: FrameworkMatchingStrategy,
    pub detectors: Vec<FrameworkDetectionItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum FrameworkDetectionItem {
    // TODO: see if this can replace Config
    #[serde(rename = "file")]
    File {
        path: String,
        content: Option<String>,
    },

    /// A matcher for a config file
    #[serde(rename = "config")]
    Config {
        /// Content that must be present in the config file
        content: Option<String>,
    },

    /// A matcher for a dependency found in project file
    #[serde(rename = "dependency")]
    Dependency { name: String },
}

// TODO: change name?
/// Matching strategies to match on a framework
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkMatchingStrategy {
    /// Strategy that requires all detectors to match for the framework to be detected
    All,

    /// Strategy where one match causes the framework to be detected
    Any,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkBuildSettings {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_args: Option<FrameworkBuildArgs>,
    pub output_directory: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkBuildArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<FrameworkBuildArg>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum FrameworkBuildArg {
    /// 0-based index of argument and default value
    #[serde(rename = "arg")]
    Arg { index: i8 },
    #[serde(rename = "option")]
    Option {
        #[serde(default)]
        name: String,
    },
}

#[derive(Debug)]
pub struct FrameworkConfigFile {
    pub path: PathBuf,
    pub settings: FrameworkConfigFileSettings,
}

#[derive(Debug)]
pub struct FrameworkConfigFileSettings {
    pub output_dir: Option<String>,
}


pub trait FrameworkConfiguration: for<'a> Deserialize<'a> + Debug {
    // TODO(Sean): Can default to Self when associated type defaults is a stable feature.
    // Perahps this is a bad idea?
    type Config: FrameworkConfiguration + Debug;

    fn from_js_module(_program: &Program) -> CifrsResult<Self> {
        unimplemented!();
    }

    fn from_python(_content: &str) -> CifrsResult<Self> {
        unimplemented!();
    }

    fn get_config<P: AsRef<Path>>(path: P) -> CifrsResult<FrameworkConfigFile> {
        let path = path.as_ref();
        let format = FrameworkConfigurationFormat::from_path(path)?;
        let config = <Self as FrameworkConfiguration>::read_config(&format)?;
        let settings = <Self as FrameworkConfiguration>::get_config_file_settings(&config);
        println!(
            "path: [{:?}] format: [{:?}] config: [{:?}] settings: [{:?}]",
            path, format, config, &settings
        );
        Ok(FrameworkConfigFile {
            path: path.to_path_buf(),
            settings,
        })
    }

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings;

    fn read_config(format: &FrameworkConfigurationFormat) -> CifrsResult<Self::Config> {
        match format {
            FrameworkConfigurationFormat::EcmaScript(p) => Self::Config::from_js_module(&p),
            FrameworkConfigurationFormat::Json(c) => {
                Ok(serde_json::from_value::<Self::Config>(c.to_owned())?)
            }
            FrameworkConfigurationFormat::Python(c) => Self::Config::from_python(c),
            FrameworkConfigurationFormat::Toml(c) => Ok(toml::from_str::<Self::Config>(c)?),
            FrameworkConfigurationFormat::Yaml(c) => {
                Ok(serde_yaml::from_value::<Self::Config>(c.to_owned())?)
            }
            _ => Err(CifrsError::UnknownFrameworkFormat("".to_string())),
        }
    }
}

#[derive(Debug)]
pub enum FrameworkConfigurationFormat {
    EcmaScript(Program),
    Json(serde_json::Value),
    // dont yet have a need to try and convert to AST
    Python(String),
    // tried toml::Value but could not figure out how to go from Value to struct
    Toml(String),
    Yaml(serde_yaml::Value),
}

impl FrameworkConfigurationFormat {
    pub fn from_path<P: AsRef<Path>>(path: P) -> CifrsResult<Self> {
        if let Some(extension) = path.as_ref().extension() {
            if let Ok(content) = fs::read_to_string(&path) {
                return match extension.to_str() {
                    Some("json") => Ok(Self::Json(serde_json::from_str(&content)?)),
                    Some("yaml") | Some("yml") => Ok(Self::Yaml(serde_yaml::from_str(&content)?)),
                    Some("toml") => Ok(Self::Toml(content)),
                    Some("js") | Some("ts") | Some("mjs") | Some("cjs") => {
                        Ok(Self::EcmaScript(parse_js_module(path, content)?))
                    }
                    Some("py") => Ok(Self::Python(content)),
                    // TODO (Sean): we should just skip or warn
                    _ => Err(CifrsError::UnknownFrameworkFormat(
                        extension.to_string_lossy().to_string(),
                    )),
                };
            }
        }

        Err(CifrsError::UnknownFrameworkFormat(String::new()))
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::get_all;

    #[test]
    fn test_deserialize_frameworks_yaml() {
        println!("{}", serde_json::to_string(&get_all()).unwrap());
    }
}
