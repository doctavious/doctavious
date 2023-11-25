// .vuepress/config.js
// inside docs directory
// which should export a JavaScript object:
// You can also use YAML (.vuepress/config.yml) or TOML (.vuepress/config.toml) formats for the configuration file.
// package.json -> "docs:build": "vuepress build docs"

// vuepress build [targetDir] -d, --dest <dest>

// .vuepress/dist
// can be configured via the dest field

// .vuepress/config.js
// .vuepress/config.yml
// .vuepress/config.toml
// .vuepress/config.ts

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
struct VuePressConfig {
    dest: Option<String>,
}


impl FrameworkConfiguration for VuePressConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: try and simplify
        if let Some(module) = program.as_module() {
            let dest = module.get_property_as_string("dest");
            if dest.is_some() {
                return Ok(Self { dest });
            }
        }
        Err(CifrsError::InvalidConfig("vuepress".to_string()))
    }
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<VuePressConfig>(format)?;
    Ok(config.dest)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_vuepress() {
        for path in [
            "tests/fixtures/framework_configs/vuepress/config.js",
            "tests/fixtures/framework_configs/vuepress/config.toml",
            "tests/fixtures/framework_configs/vuepress/config.ts",
        ] {
            let config = FrameworkConfigurationFormat::from_path(path).unwrap();
            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        }
    }
}
