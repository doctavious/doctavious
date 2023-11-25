// config.toml/yaml/json
// multiple can be used
// also has a config directory
// has options that would need to be merged. how to handle?
// hugo command
// hugo -d, --destination

// /public
// can be changed via publishDir

use std::path::PathBuf;

use serde::Deserialize;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HugoConfig {
    publish_dir: Option<String>,
}

impl FrameworkConfiguration for HugoConfig {}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<HugoConfig>(format)?;
    Ok(config.publish_dir)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_hugo() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/hugo/config.toml",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("build")))
    }
}
