// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static

use serde::Deserialize;
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

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct SvelteKitConfig {
    output: Option<String>,
}

pub struct SvelteKit {
    info: FrameworkInfo,
}

impl SvelteKit {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "SvelteKit",
                website: Some("https://kit.svelte.dev/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency {
                        name: "@sveltejs/kit",
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "vite build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--outDir",
                        }),
                    }),
                    // TODO: validate
                    // according to the following https://github.com/netlify/build/pull/4823
                    // .svelte-kit is the internal build dir, not the publish dir.
                    output_directory: "build", //".svelte-kit",
                },
            },
        }
    }
}

impl Default for SvelteKit {
    fn default() -> Self {
        SvelteKit::new(Some(vec!["svelte.config.js"]))
    }
}

impl FrameworkSupport for SvelteKit {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<SvelteKitConfig>(configs) {
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
            Some(vec![
                "tests/fixtures/framework_configs/sveltekit/svelte.config.js",
            ]),
        );

        let output = sveltekit.get_output_dir();
        assert_eq!(output, "build")
    }
}
