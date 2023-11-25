// .vitepress/config.js
// which should export a JavaScript object:
// .vitepress/dist
// can be configured via the outDir field
// "docs:build": "vitepress build docs",
// do we allow to customize the script we look for? ex: instead of 'build' we look for 'docs:build'
// package.json

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
struct VitePressConfig {
    output: Option<String>,
}

impl FrameworkConfiguration for VitePressConfig {
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
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<VitePressConfig>(format)?;
    Ok(config.output)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_vitepress() {
        for path in [
            "tests/fixtures/framework_configs/vitepress/config.js",
            "tests/fixtures/framework_configs/vitepress/config.ts",
        ] {
            let config = FrameworkConfigurationFormat::from_path(path).unwrap();
            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        }
    }
}
