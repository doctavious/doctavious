// book.toml
// ./book -> default
// change be changed via build.build-dir

use std::collections::HashMap;

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MDBookBuildOptions {
    build_dir: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MDBookConfig {
    build: HashMap<String, String>,
}

impl FrameworkConfiguration for MDBookConfig {
    type Config = Self;

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.build.get("build-dir").cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::FrameworkConfiguration;
    use crate::frameworks::mdbook::MDBookConfig;

    #[test]
    fn test_mdbook() {
        let config =
            MDBookConfig::get_config("tests/fixtures/framework_configs/mdbook/book.toml").unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("build")))
    }
}
