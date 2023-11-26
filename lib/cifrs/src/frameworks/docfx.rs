// docfx.json
// "docfx <docfx_project>/docfx.json"
// _site
// docfx build [-o:<output_path>] [-t:<template folder>]

use std::collections::HashMap;

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};

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

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
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

        assert_eq!(config.output_dir, Some(String::from("dist")))
    }
}
