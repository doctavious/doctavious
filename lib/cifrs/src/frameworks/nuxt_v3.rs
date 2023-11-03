use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildSettings,
    FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy,
    FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

#[derive(Deserialize)]
struct Nuxt3JSConfig {
    output: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Nuxt3JS {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Nuxt3JS {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "nuxt-v3".to_string(),
                name: "Nuxt 3".to_string(),
                website: "https://nuxtjs.org/".to_string(),
                configs,
                // language: Language::Javascript,
                language: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "nuxt3".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "nuxi generate".to_string(), // same as nuxi build --prerender true
                    command_args: None,
                    output_directory: ".output".to_string(),
                },
            },
        }
    }
}

impl Default for Nuxt3JS {
    fn default() -> Self {
        Nuxt3JS::new(Vec::from([
            "nuxt.config.js".to_string(),
            "nuxt.config.mjs".to_string(),
            "nuxt.config.ts".to_string(),
        ]))
    }
}

impl FrameworkSupport for Nuxt3JS {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<Nuxt3JSConfig>(&self.info.configs) {
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
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nuxt".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Nuxt3JS;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_nuxtjs() {
        for config in [
            "tests/fixtures/framework_configs/nuxt3js/nuxt_nitro.config.ts".to_string(),
            "tests/fixtures/framework_configs/nuxt3js/nuxt_vite.config.ts".to_string(),
        ] {
            let nuxtjs = Nuxt3JS::new(vec![config]);

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }
    }
}
