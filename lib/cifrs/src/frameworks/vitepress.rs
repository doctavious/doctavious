// .vitepress/config.js
// which should export a JavaScript object:
// .vitepress/dist
// can be configured via the outDir field
// "docs:build": "vitepress build docs",
// do we allow to customize the script we look for? ex: instead of 'build' we look for 'docs:build'
// package.json

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct VitePressConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for VitePressConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        println!("{}", serde_json::to_string(&program)?);
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("outDir");
            if output.is_some() {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("vitepress".to_string()))
    }

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.output.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::vitepress::VitePressConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_vitepress() {
        for path in [
            "tests/fixtures/framework_configs/vitepress/config.js",
            "tests/fixtures/framework_configs/vitepress/config.ts",
        ] {
            let config = VitePressConfig::get_config(path).unwrap();
            assert_eq!(config.output_dir, Some(String::from("build")))
        }
    }
}
