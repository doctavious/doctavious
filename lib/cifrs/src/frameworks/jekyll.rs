// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

use std::path::PathBuf;

use serde::Deserialize;

//read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
struct JekyllConfig {
    destination: Option<String>,
}

impl FrameworkConfiguration for JekyllConfig {}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<JekyllConfig>(format)?;
    Ok(config.destination)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_jekyll() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/jekyll/_config.yml",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("build")))
    }
}
