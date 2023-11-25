// defaults to ".svelte-kit"
// svelte.config.js
// outDir overrides
// dependency - adapter-static

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::{
    get_string_property_value, get_variable_declaration, get_variable_properties,
};
use crate::{CifrsError, CifrsResult};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct SvelteKitConfig {
    output: Option<String>,
}


impl FrameworkConfiguration for SvelteKitConfig {
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
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<SvelteKitConfig>(format)?;
    Ok(config.output)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_sveltekit() {
        // tests/fixtures/framework_configs/sveltekit/svelte.config.js
        // tests/fixtures/framework_configs/sveltekit/svelte.config.js
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/sveltekit/svelte.config.js",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("build")))
    }
}
