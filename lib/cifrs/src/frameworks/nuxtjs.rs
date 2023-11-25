// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

// nuxt v2 for static pre-rendered
// nuxt generate
// dist/

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

//read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct NuxtJSConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for NuxtJSConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("buildDir");
            if output.is_some() {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nuxtjs".to_string()))
    }
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<NuxtJSConfig>(format)?;
    Ok(config.output)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_nuxtjs() {
        for path in ["tests/fixtures/framework_configs/nuxtjs/nuxt.config.js"] {
            let config = FrameworkConfigurationFormat::from_path(path).unwrap();
            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        }
    }
}
