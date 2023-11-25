// mkdocs.yml
// site --> default
// change be changed via site_dir

use std::path::PathBuf;

use serde::Deserialize;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
struct MKDocsConfig {
    site_dir: Option<String>,
}

impl FrameworkConfiguration for MKDocsConfig {}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<MKDocsConfig>(format)?;
    Ok(config.site_dir)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_hugo() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/mkdocs/mkdocs.yml",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("build")))
    }
}
