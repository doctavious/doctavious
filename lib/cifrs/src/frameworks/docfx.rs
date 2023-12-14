// docfx.json
// "docfx <docfx_project>/docfx.json"
// _site
// docfx build [-o:<output_path>] [-t:<template folder>]

use std::collections::HashMap;

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};

#[derive(Deserialize)]
pub struct DocFxConfig {
    build: HashMap<String, String>,
}

#[derive(Deserialize)]
struct DocFxConfigBuild {
    dest: String,
}

impl FrameworkConfiguration for DocFxConfig {
    type Config = Self;

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.build.get("dest").cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::docfx::DocFxConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_docfx() {
        let config =
            DocFxConfig::get_config("tests/fixtures/framework_configs/docfx/docfx.json").unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("dist")))
    }
}
