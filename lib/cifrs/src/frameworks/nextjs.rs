// next.config.js / next.config.mjs
// this is a regular Node.js module
// could also look at package.json -> scripts -> "build": "next build",

// .next -> default directory
// change be changed via distDir

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct NextJSConfig {
    output: String,
}

impl FrameworkConfiguration for NextJSConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            if let Some(output) = module.get_property_as_string("distDir") {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nextjs".to_string()))
    }

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: Some(config.output.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::nextjs::NextJSConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_nextjs() {
        for path in [
            "tests/fixtures/framework_configs/nextjs/next_js_v1.mjs",
            "tests/fixtures/framework_configs/nextjs/next_js_v2.mjs",
        ] {
            let config = NextJSConfig::get_config(path).unwrap();
            assert_eq!(config.settings.output_dir, Some(String::from("build")))
        }
    }
}
