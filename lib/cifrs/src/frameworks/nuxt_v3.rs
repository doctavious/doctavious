use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct Nuxt3JSConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for Nuxt3JSConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("publicDir");
            if output.is_some() {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nuxt".to_string()))
    }

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.output.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::nuxt_v3::Nuxt3JSConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_nuxtjs() {
        for path in [
            "tests/fixtures/framework_configs/nuxt3js/nuxt_nitro.config.ts",
            "tests/fixtures/framework_configs/nuxt3js/nuxt_vite.config.ts",
        ] {
            let config = Nuxt3JSConfig::get_config(path).unwrap();
            assert_eq!(config.settings.output_dir, Some(String::from("build")))
        }
    }
}
