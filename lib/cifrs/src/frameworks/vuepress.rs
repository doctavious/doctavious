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

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct VuePressConfig {
    dest: Option<String>,
}

impl FrameworkConfiguration for VuePressConfig {
    type Config = Self;

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

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.dest.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::vuepress::VuePressConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_vuepress() {
        for path in [
            "tests/fixtures/framework_configs/vuepress/config.js",
            "tests/fixtures/framework_configs/vuepress/config.toml",
            "tests/fixtures/framework_configs/vuepress/config.ts",
        ] {
            let config = VuePressConfig::get_config(path).unwrap();
            assert_eq!(config.settings.output_dir, Some(String::from("build")))
        }
    }
}
