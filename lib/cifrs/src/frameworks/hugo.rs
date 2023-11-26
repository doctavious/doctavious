// config.toml/yaml/json
// multiple can be used
// also has a config directory
// has options that would need to be merged. how to handle?
// hugo command
// hugo -d, --destination

// /public
// can be changed via publishDir

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HugoConfig {
    publish_dir: Option<String>,
}

impl FrameworkConfiguration for HugoConfig {
    type Config = Self;

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.publish_dir.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::hugo::HugoConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_hugo() {
        let config =
            HugoConfig::get_config("tests/fixtures/framework_configs/hugo/config.toml").unwrap();

        assert_eq!(config.output_dir, Some(String::from("build")))
    }
}
