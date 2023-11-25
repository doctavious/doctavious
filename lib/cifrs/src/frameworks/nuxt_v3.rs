use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct Nuxt3JSConfig {
    output: Option<String>,
}


impl FrameworkConfiguration for Nuxt3JSConfig {
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

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<Nuxt3JSConfig>(format)?;
    Ok(config.output)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_nuxtjs() {
        for path in [
            "tests/fixtures/framework_configs/nuxt3js/nuxt_nitro.config.ts",
            "tests/fixtures/framework_configs/nuxt3js/nuxt_vite.config.ts",
        ] {
            let config = FrameworkConfigurationFormat::from_path(path).unwrap();
            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        }
    }
}
