// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};

#[derive(Debug, Deserialize)]
pub struct JekyllConfig {
    destination: Option<String>,
}

impl FrameworkConfiguration for JekyllConfig {
    type Config = Self;

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.destination.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::FrameworkConfiguration;
    use crate::frameworks::jekyll::JekyllConfig;

    #[test]
    fn test_jekyll() {
        let config =
            JekyllConfig::get_config("tests/fixtures/framework_configs/jekyll/_config.yml")
                .unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("build")))
    }
}
