// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

// nuxt v2 for static pre-rendered
// nuxt generate
// dist/

use std::path::PathBuf;

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::backends::LanguageBackends;
use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildSettings,
    FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy,
    FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct NuxtJSConfig {
    output: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct NuxtJS {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl NuxtJS {
    fn new(configs: Vec<PathBuf>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "nuxt".to_string(),
                name: "Nuxt".to_string(),
                website: "https://nuxtjs.org/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency {
                            name: "nuxt".to_string(),
                        },
                        FrameworkDetectionItem::Dependency {
                            name: "nuxt-edge".to_string(),
                        },
                    ],
                },
                build: FrameworkBuildSettings {
                    command: "nuxt build".to_string(),
                    command_args: None,
                    output_directory: ".nuxt".to_string(),
                },
            },
        }
    }
}

impl Default for NuxtJS {
    fn default() -> Self {
        NuxtJS::new(Vec::from(["nuxt.config.js".into()]))
    }
}

impl FrameworkSupport for NuxtJS {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<NuxtJSConfig>(&self.info.configs) {
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

impl ConfigurationFileDeserialization for NuxtJSConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("buildDir");
            if output.is_some() {
                return Ok(Self { output });
            }
            // for item in &module.body {
            //     if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
            //         if let Some(obj) = export_expression.expr.as_object() {
            //             let output = get_string_property_value(&obj.props, "buildDir");
            //             if output.is_some() {
            //                 return Ok(Self {
            //                     output
            //                 });
            //             }
            //         }
            //     }
            // }
        }
        Err(CifrsError::InvalidConfig("nuxtjs".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::NuxtJS;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_nuxtjs() {
        for config in ["tests/fixtures/framework_configs/nuxtjs/nuxt.config.js".into()] {
            let nuxtjs = NuxtJS::new(vec![config]);

            let output = nuxtjs.get_output_dir();
            assert_eq!(output, String::from("build"))
        }
    }
}
