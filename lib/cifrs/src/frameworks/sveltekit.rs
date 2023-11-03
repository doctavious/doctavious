// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::{
    get_string_property_value, get_variable_declaration, get_variable_properties,
};
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct SvelteKitConfig {
    output: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct SvelteKit {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl SvelteKit {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "sveltekit".to_string(),
                name: "SvelteKit".to_string(),
                website: "https://kit.svelte.dev/".to_string(),
                configs,
                // language: Language::Javascript,
                language: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency {
                        name: "@sveltejs/kit".to_string(),
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "vite build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--outDir".to_string(),
                        }),
                    }),
                    // TODO: validate
                    // according to the following https://github.com/netlify/build/pull/4823
                    // .svelte-kit is the internal build dir, not the publish dir.
                    output_directory: "build".to_string(), //".svelte-kit",
                },
            },
        }
    }
}

impl Default for SvelteKit {
    fn default() -> Self {
        SvelteKit::new(vec!["svelte.config.js".to_string()])
    }
}

impl FrameworkSupport for SvelteKit {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<SvelteKitConfig>(&self.info.configs) {
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

impl ConfigurationFileDeserialization for SvelteKitConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: not sure we need to specifically get 'config' and perhaps rather look for
        // kit and/or outDir
        // if let Some(module) = program.as_module() {
        //     let output = module.get_property_as_string("outDir");
        //     if output.is_some() {
        //         return Ok(Self {
        //             output
        //         });
        //     }
        // }

        let var = get_variable_declaration(program, "config");
        if let Some(var) = var {
            let properties = get_variable_properties(var, "kit");
            if let Some(properties) = properties {
                let output = get_string_property_value(properties, "outDir");
                if output.is_some() {
                    return Ok(Self { output });
                }
            }
        }

        Err(CifrsError::InvalidConfig("sveltekit".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::SvelteKit;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_sveltekit() {
        let sveltekit = SvelteKit::new(
            // tests/fixtures/framework_configs/sveltekit/svelte.config.js
            // tests/fixtures/framework_configs/sveltekit/svelte.config.js
            vec![
                "tests/fixtures/framework_configs/sveltekit/svelte.config.js".to_string(),
            ],
        );

        let output = sveltekit.get_output_dir();
        assert_eq!(output, "build")
    }
}
