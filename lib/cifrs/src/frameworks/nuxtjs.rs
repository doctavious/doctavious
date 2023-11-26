// nuxt.config.js
// could also look at package.json -> scripts -> "build": "nuxt build",

// .nuxt --> default
// change be changed via buildDir

// nuxt v2 for static pre-rendered
// nuxt generate
// dist/

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct NuxtJSConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for NuxtJSConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("buildDir");
            if output.is_some() {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nuxtjs".to_string()))
    }

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.output.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::nuxtjs::NuxtJSConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_nuxtjs() {
        for path in ["tests/fixtures/framework_configs/nuxtjs/nuxt.config.js"] {
            let config = NuxtJSConfig::get_config(path).unwrap();
            assert_eq!(config.output_dir, Some(String::from("build")))
        }
    }
}
