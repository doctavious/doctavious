// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};
use crate::js_module::{
    get_string_property_value, get_variable_declaration, get_variable_properties,
};
use crate::{CifrsError, CifrsResult};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
pub struct SvelteKitConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for SvelteKitConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: not sure we need to specifically get 'config' and perhaps rather look for
        // kit and/or outDir
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

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.output.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::sveltekit::SvelteKitConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_sveltekit() {
        // tests/fixtures/framework_configs/sveltekit/svelte.config.js
        // tests/fixtures/framework_configs/sveltekit/svelte.config.js
        let config = SvelteKitConfig::get_config(
            "tests/fixtures/framework_configs/sveltekit/svelte.config.js",
        )
        .unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("build")))
    }
}
