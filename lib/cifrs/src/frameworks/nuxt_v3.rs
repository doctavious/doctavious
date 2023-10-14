use serde::{Deserialize};
use swc_ecma_ast::{Program};
use crate::{CifrsError, CifrsResult};

use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::js_module::{PropertyAccessor};
use crate::language::Language;
#[derive(Deserialize)]
struct Nuxt3JSConfig { output: Option<String> }

pub struct Nuxt3JS { info: FrameworkInfo }

impl Nuxt3JS {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Nuxt 3",
                website: Some("https://nuxtjs.org/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency { name: "nuxt3"},
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "nuxi generate", // same as nuxi build --prerender true
                    command_args: None,
                    output_directory: ".output",
                },
            }
        }
    }
}

impl Default for Nuxt3JS {
    fn default() -> Self {
        Nuxt3JS::new(Some(Vec::from(["nuxt.config.js", "nuxt.config.mjs", "nuxt.config.ts"])))
    }
}


impl FrameworkSupport for Nuxt3JS {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<Nuxt3JSConfig>(configs) {
                Ok(c) => {
                    if let Some(dest) = c.output {
                        return dest;
                    }
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

impl ConfigurationFileDeserialization for Nuxt3JSConfig {

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("publicDir");
            if output.is_some() {
                return Ok(Self {
                    output
                })
            }
        }
        Err(CifrsError::InvalidConfig("nuxt".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::framework::FrameworkSupport;
    use super::Nuxt3JS;

    #[test]
    fn test_nuxtjs() {
        for config in [
            "tests/fixtures/framework_configs/nuxt3js/nuxt_nitro.config.ts",
            "tests/fixtures/framework_configs/nuxt3js/nuxt_vite.config.ts"
        ] {
            let nuxtjs = Nuxt3JS::new(Some(vec![config]));

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }

    }
}
