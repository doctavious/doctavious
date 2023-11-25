use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::backends::LanguageBackends;
use crate::framework_detection::Detectable;
use crate::frameworks::{
    antora, astro, docfx, docusaurus_v2, eleventy, gatsby, hexo, hugo, jekyll, mdbook, mkdocs,
    nextjs, nuxt_v3, nuxtjs, sphinx, sveltekit, vitepress, vuepress,
};
use crate::js_module::parse_js_module;
use crate::projects::project_file::ProjectFile;
use crate::{CifrsError, CifrsResult};

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
    pub fn get_output_dir(&self) -> String {
        for config in &self.configs {
            if config.is_file() {
                if let Ok(config_format) = FrameworkConfigurationFormat::from_path(config) {
                    // this seems generally ok but some additional thoughts
                    // could be an configuration enum and then make one call to get_output_dir
                    // could just do a Box<dyn ConfigurationFile> and then do single get_output_dir
                    // could just have each implementation return a common ConfigurationFile struct
                    // which has an output dir.
                    // I like the idea of the last option but not yet sure of the shape
                    let output_dir = match self.id.as_str() {
                        "antora" => antora::get_output_dir(&config_format),
                        "astro" => astro::get_output_dir(&config_format),
                        "docfx" => docfx::get_output_dir(&config_format),
                        "docusaurus-v2" => docusaurus_v2::get_output_dir(&config_format),
                        "eleventy" => eleventy::get_output_dir(&config_format),
                        "gatsby" => gatsby::get_output_dir(&config_format),
                        "hexo" => hexo::get_output_dir(&config_format),
                        "hugo" => hugo::get_output_dir(&config_format),
                        "jekyll" => jekyll::get_output_dir(&config_format),
                        "mdbook" => mdbook::get_output_dir(&config_format),
                        "mkdocs" => mkdocs::get_output_dir(&config_format),
                        "nextjs" => nextjs::get_output_dir(&config_format),
                        "nuxt-v2" => nuxtjs::get_output_dir(&config_format),
                        "nuxt-v3" => nuxt_v3::get_output_dir(&config_format),
                        "sphinx" => sphinx::get_output_dir(&config_format),
                        "sveltekit" => sveltekit::get_output_dir(&config_format),
                        "vitepress" => vitepress::get_output_dir(&config_format),
                        "vuepress" => vuepress::get_output_dir(&config_format),
                        _ => todo!(),
                    };

                    if let Ok(Some(output_dir)) = output_dir {
                        return output_dir;
                    }
                }
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
    Arg {
        index: i8,
        #[serde(skip_serializing_if = "Option::is_none")]
        default_value: Option<String>,
    },
    // TODO: do we care short or long? how about use vec/array? I dont think it really matters
    // I Think we should just have option with a name/value/arg
    #[serde(rename = "option")]
    Option {
        #[serde(default)]
        short: String,
        #[serde(default)]
        long: String,
    },
}

pub trait FrameworkSupport {
    fn get_info(&self) -> &FrameworkInfo;

    fn get_output_dir(&self) -> String {
        self.get_info().build.output_directory.to_string()
    }
}


pub trait FrameworkConfiguration: for<'a> Deserialize<'a> {

    fn from_js_module(_program: &Program) -> CifrsResult<Self> {
        unimplemented!();
    }

    // fn get_output_dir(&self) -> CifrsResult<Option<String>>;
}

pub enum FrameworkConfigurationFormat {
    EcmaScript(Program),
    Json(serde_json::Value),
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
                    Some("js") | Some("ts") | Some("mjs") | Some("cjs") => Ok(Self::EcmaScript(
                        parse_js_module(path.as_ref().to_owned().into(), content)?,
                    )),
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

pub fn deser_config<T>(format: &FrameworkConfigurationFormat) -> CifrsResult<T>
where
    T: FrameworkConfiguration,
{
    match format {
        FrameworkConfigurationFormat::EcmaScript(p) => T::from_js_module(&p),
        FrameworkConfigurationFormat::Json(c) => Ok(serde_json::from_value::<T>(c.to_owned())?),
        FrameworkConfigurationFormat::Toml(c) => Ok(toml::from_str::<T>(c)?),
        FrameworkConfigurationFormat::Yaml(c) => Ok(serde_yaml::from_value::<T>(c.to_owned())?),
        _ => Err(CifrsError::UnknownFrameworkFormat("".to_string())),
    }
}
