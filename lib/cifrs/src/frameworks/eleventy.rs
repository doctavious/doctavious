// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site

use std::path::PathBuf;

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::backends::LanguageBackends;
use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::{
    get_assignment_function, get_function_return_obj, get_obj_property, get_string_property_value,
};
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct EleventyConfig {
    output: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Eleventy {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Eleventy {
    fn new(configs: Vec<PathBuf>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "eleventy".to_string(),
                name: "Eleventy".to_string(),
                website: "https://www.11ty.dev/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency {
                        name: "@11ty/eleventy".to_string(),
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "eleventy".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--output".to_string(),
                        }),
                    }),
                    output_directory: "_site".to_string(),
                },
            },
        }
    }
}

impl Default for Eleventy {
    fn default() -> Self {
        Eleventy::new(Vec::from([
            ".eleventy.js".into(),
            "eleventy.config.js".into(),
            "eleventy.config.cjs".into(),
        ]))
    }
}

impl FrameworkSupport for Eleventy {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<EleventyConfig>(&self.info.configs) {
                Ok(c) => {
                    return c.output;
                }
                Err(e) => {
                    // log warning/error
                    println!("{}", e);
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for EleventyConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(func) = get_assignment_function(program) {
            if let Some(return_obj) = get_function_return_obj(func) {
                if let Some(dir_prop) = get_obj_property(return_obj, "dir") {
                    if let Some(output) = get_string_property_value(&dir_prop.props, "output") {
                        return Ok(Self { output });
                    }
                }
            }
        }

        Err(CifrsError::InvalidConfig("eleventy".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Eleventy;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_eleventy() {
        let eleventy = Eleventy::new(vec![
            "tests/fixtures/framework_configs/eleventy/.eleventy.js".into(),
        ]);

        let output = eleventy.get_output_dir();
        assert_eq!(output, String::from("dist"))
    }
}
