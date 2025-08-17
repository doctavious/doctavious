// mkdocs.yml
// site --> default
// change be changed via site_dir

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};

#[derive(Debug, Deserialize)]
pub struct MKDocsConfig {
    site_dir: Option<String>,
}

impl FrameworkConfiguration for MKDocsConfig {
    type Config = Self;

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.site_dir.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::FrameworkConfiguration;
    use crate::frameworks::mkdocs::MKDocsConfig;

    #[test]
    fn test_hugo() {
        let config =
            MKDocsConfig::get_config("tests/fixtures/framework_configs/mkdocs/mkdocs.yml").unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("build")))
    }
}
