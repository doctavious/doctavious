// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::{
    get_assignment_function, get_function_return_obj, get_obj_property, get_string_property_value,
};
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
#[derive(Deserialize)]
struct EleventyConfig {
    output: String,
}

pub struct Eleventy {
    info: FrameworkInfo,
}

impl Eleventy {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Eleventy",
                website: Some("https://www.11ty.dev/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency {
                        name: "@11ty/eleventy",
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "eleventy",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--output",
                        }),
                    }),
                    output_directory: "_site",
                },
            },
        }
    }
}

impl Default for Eleventy {
    fn default() -> Self {
        Eleventy::new(Some(Vec::from([
            ".eleventy.js",
            "eleventy.config.js",
            "eleventy.config.cjs",
        ])))
    }
}

impl FrameworkSupport for Eleventy {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<EleventyConfig>(configs) {
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
        let eleventy = Eleventy::new(Some(vec![
            "tests/fixtures/framework_configs/eleventy/.eleventy.js",
        ]));

        let output = eleventy.get_output_dir();
        assert_eq!(output, String::from("dist"))
    }
}
