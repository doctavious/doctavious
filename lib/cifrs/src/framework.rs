use std::fs;
use std::path::Path;
use serde::Deserialize;

use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::{CifrsError, CifrsResult};
use crate::js_module::parse_js_module;
use crate::language::Language;


// FrameworkDefinition
#[derive(Serialize)]
pub struct FrameworkInfo {
    // id: String,

    /// Name of the framework
    ///
    /// # Examples
    /// Next.js
    pub name: &'static str,

    /// A URL to the official website of the framework
    ///
    /// # Examples
    /// https://nextjs.org
    pub website: Option<&'static str>,

    // TODO: does this really need to be an Option? How about just empty?
    /// List of potential config files
    pub configs: Option<Vec<&'static str>>,

    // TODO: this could be SoftwareFramework / SoftwarePlatform.
    // Potentially solves for the scenario of needed multiple languages such as C#/F#
    pub language: Language,

    // /// Detectors used to find out the framework
    pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,

}

impl FrameworkInfo {

    pub fn detected(&self) -> bool {
        let mut results = vec![];
        // let stop_on_first_found = FrameworkMatchingStrategy::Any == &self.detection.matching_strategy;
        for detection in &self.detection.detectors {
            let result = match detection {
                FrameworkDetectionItem::Config { content } => {
                    if let Some(configs) = &self.configs {
                        for config in configs {
                            if let Ok(file_content) = fs::read_to_string(config) {
                                if let Some(content) = content {
                                    if file_content.contains(content) {
                                        return true;
                                    }
                                    continue;
                                }
                                return true;
                            }
                        }
                    }
                    false
                }
                FrameworkDetectionItem::Dependency { name: dependency } => {
                    for project_file in self.language.project_files() {
                        // if project_file.has_dependency(dependency) {
                        //     return true;
                        // }
                    }
                    // for pck_manager in self.language.get_package_managers() {
                    //     if pck_manager.has_dependency(dependency) {
                    //         return true;
                    //     }
                    // }
                    false
                }
                _ => { false }
            };

            match &self.detection.matching_strategy {
                FrameworkMatchingStrategy::All => {
                    results.push(result);
                }
                FrameworkMatchingStrategy::Any => {
                    if result {
                        results.push(result);
                        break;
                    }
                }
            }
        }

        // use std::convert::identity might be more idiomatic here
        results.iter().all(|&r| r)
    }

}

// TODO: rename to FrameworkDetection?
#[derive(Serialize)]
pub struct FrameworkDetector {
    pub matching_strategy: FrameworkMatchingStrategy,
    pub detectors: Vec<FrameworkDetectionItem>
}

#[derive(Debug, Serialize)]
pub enum FrameworkDetectionItem {

    // TODO: see if this can replace Config
    File {
        path: &'static str,
        content: Option<&'static str>
    },

    /// A matcher for a config file
    Config {
        /// Content that must be present in the config file
        content: Option<&'static str>
    },

    /// A matcher for a dependency found in project file
    Dependency { name: &'static str }
}


#[derive(Serialize)]
pub enum RequiredDependencies {
    All(Vec<&'static str>),
    Any(Vec<&'static str>)
}

// TODO: change name?
/// Matching strategies to match on a framework
#[derive(Serialize)]
pub enum FrameworkMatchingStrategy {
    /// Strategy that requires all detectors to match for the framework to be detected
    All,

    /// Strategy where one match causes the framework to be detected
    Any
}


#[derive(Serialize)]
pub struct FrameworkBuildSettings {
    pub command: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_args: Option<FrameworkBuildArgs>,
    pub output_directory: &'static str
}

#[derive(Serialize)]
pub struct FrameworkBuildArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<FrameworkBuildArg>
}

#[derive(Serialize)]
pub enum FrameworkBuildArg {
    /// 0-based index of argument and default value
    Arg { index: i8, default_value: Option<&'static str> },
    // TODO: do we care short or long? how about use vec/array?
    Option { short: &'static str, long: &'static str }
}


pub trait FrameworkSupport {

    fn get_info(&self) -> &FrameworkInfo;

    fn get_output_dir(&self) -> String {
        self.get_info().build.output_directory.to_string()
    }
}

// I tried to use Deserialize however I couldnt think of a good way to implement
// Deserialize trait for Program to associated Config. If there is a way I think that would
// be preferred. This trait still requires config struct implement Deserialize and we forward
// to various serde implementations that support more strait forward deserialization formats
// and provide a custom implementation for cases were we need to get data from JS modules
pub trait ConfigurationFileDeserialization: for<'a> Deserialize<'a> {

    fn from_json(s: &str) -> CifrsResult<Self> {
        Ok(serde_json::from_str(s)?)
    }

    fn from_yaml(s: &str) -> CifrsResult<Self> {
        Ok(serde_yaml::from_str(s)?)
    }

    fn from_toml(s: &str) -> CifrsResult<Self> {
        Ok(toml::from_str(s)?)
    }

    fn from_js_module(_program: &Program) -> CifrsResult<Self> {
        unimplemented!();
    }
}

pub(crate) fn read_config_files<T>(files: &Vec<&'static str>) -> CifrsResult<T>
    where T: ConfigurationFileDeserialization
{
    for file in files {
        let path = Path::new(&file);
        if let Some(extension) = path.extension() {
            if let Ok(content) = fs::read_to_string(file) {
                return match extension.to_str() {
                    Some("json") => T::from_json(content.as_str()),
                    Some("yaml") | Some("yml") => T::from_yaml(content.as_str()),
                    Some("toml") => T::from_toml(content.as_str()),
                    Some("js") | Some("ts") | Some("mjs") | Some("cjs") => {
                        let program = parse_js_module(path.to_owned().into(), content)?;
                        return T::from_js_module(&program);
                    }
                    // TODO (Sean): we should just skip or warn
                    _ => Err(CifrsError::UnknownFrameworkExtension(extension.to_string_lossy().to_string()))
                }
            }
        }
    }

    // TODO (Sean): better error message / handling
    Err(CifrsError::MissingFrameworkConfig())
}
