use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::backends::LanguageBackends;
use crate::framework_detection::Detectable;
use crate::frameworks::antora::Antora;
use crate::frameworks::astro::Astro;
use crate::frameworks::docfx::DocFx;
use crate::frameworks::docusaurus_v2::DocusaurusV2;
use crate::frameworks::eleventy::Eleventy;
use crate::frameworks::gatsby::Gatsby;
use crate::frameworks::hexo::Hexo;
use crate::frameworks::hugo::Hugo;
use crate::frameworks::jekyll::Jekyll;
use crate::frameworks::mdbook::MDBook;
use crate::frameworks::mkdocs::MKDocs;
use crate::frameworks::nextjs::NextJS;
use crate::frameworks::nuxtjs::NuxtJS;
use crate::frameworks::sphinx::Sphinx;
use crate::frameworks::sveltekit::SvelteKit;
use crate::frameworks::vitepress::VitePress;
use crate::frameworks::vuepress::VuePress;
use crate::js_module::parse_js_module;
use crate::projects::project_file::ProjectFile;
use crate::{CifrsError, CifrsResult};

// FrameworkDefinition
#[derive(Debug, Deserialize, PartialEq, Serialize)]
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
    pub configs: Vec<PathBuf>,

    // TODO: this could be SoftwareFramework / SoftwarePlatform.
    // Potentially solves for the scenario of needed multiple languages such as C#/F#
    // I kinda like Replit's UPM LanguageBackend. Or Could use LanguageRuntime
    pub backend: LanguageBackends,

    // /// Detectors used to find out the framework
    pub detection: FrameworkDetector,

    pub build: FrameworkBuildSettings,
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

impl FrameworkInfo {
    pub fn detected(&self) -> bool {
        let mut results = vec![];
        // let stop_on_first_found = FrameworkMatchingStrategy::Any == &self.detection.matching_strategy;
        for detection in &self.detection.detectors {
            let result = match detection {
                FrameworkDetectionItem::Config { content } => {
                    for config in &self.configs {
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

                    false
                }
                FrameworkDetectionItem::Dependency { name: dependency } => {
                    // for project_file in self.language.project_files() {
                    //     if project_file.has_dependency(dependency) {
                    //         return true;
                    //     }
                    // }
                    // for pck_manager in self.language.get_package_managers() {
                    //     if pck_manager.has_dependency(dependency) {
                    //         return true;
                    //     }
                    // }
                    false
                }
                _ => false,
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
#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkDetector {
    pub matching_strategy: FrameworkMatchingStrategy,
    pub detectors: Vec<FrameworkDetectionItem>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkMatchingStrategy {
    /// Strategy that requires all detectors to match for the framework to be detected
    All,

    /// Strategy where one match causes the framework to be detected
    Any,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkBuildSettings {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_args: Option<FrameworkBuildArgs>,
    pub output_directory: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct FrameworkBuildArgs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<FrameworkBuildArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<FrameworkBuildArg>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
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

pub(crate) fn read_config_files<T>(files: &Vec<PathBuf>) -> CifrsResult<T>
where
    T: ConfigurationFileDeserialization,
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
                    _ => Err(CifrsError::UnknownFrameworkExtension(
                        extension.to_string_lossy().to_string(),
                    )),
                };
            }
        }
    }

    // TODO (Sean): better error message / handling
    Err(CifrsError::MissingFrameworkConfig())
}

// I wish Box<dyn> hasnt necessary and maybe its not with a different structure
// but I'm at a loss for how how to structure these frameworks and allow fn overrides,
// so I suppose this will have to work until I or someone else comes up with something better
pub fn get_frameworks() -> Vec<Box<dyn FrameworkSupport>> {
    let mut frameworks = Vec::<Box<dyn FrameworkSupport>>::new();
    frameworks.push(Box::new(Antora::default()));
    frameworks.push(Box::new(Astro::default()));
    frameworks.push(Box::new(DocFx::default()));
    frameworks.push(Box::new(DocusaurusV2::default()));
    frameworks.push(Box::new(Eleventy::default()));
    frameworks.push(Box::new(Gatsby::default()));
    frameworks.push(Box::new(Hexo::default()));
    frameworks.push(Box::new(Hugo::default()));
    frameworks.push(Box::new(Jekyll::default()));
    frameworks.push(Box::new(MDBook::default()));
    frameworks.push(Box::new(MKDocs::default()));
    frameworks.push(Box::new(NextJS::default()));
    frameworks.push(Box::new(NuxtJS::default()));
    frameworks.push(Box::new(Sphinx::default()));
    frameworks.push(Box::new(SvelteKit::default()));
    frameworks.push(Box::new(VitePress::default()));
    frameworks.push(Box::new(VuePress::default()));
    frameworks
}
