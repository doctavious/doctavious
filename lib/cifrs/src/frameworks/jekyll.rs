// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};

#[derive(Deserialize)]
pub struct JekyllConfig {
    destination: Option<String>,
}

impl FrameworkConfiguration for JekyllConfig {
    type Config = Self;

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.destination.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::jekyll::JekyllConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_jekyll() {
        let config =
            JekyllConfig::get_config("tests/fixtures/framework_configs/jekyll/_config.yml")
                .unwrap();

        assert_eq!(config.output_dir, Some(String::from("build")))
    }
}
