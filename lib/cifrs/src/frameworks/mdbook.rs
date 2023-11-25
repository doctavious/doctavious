// book.toml
// ./book -> default
// change be changed via build.build-dir

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MDBookBuildOptions {
    build_dir: Option<String>,
}

#[derive(Deserialize)]
struct MDBookConfig {
    build: HashMap<String, String>,
}

impl FrameworkConfiguration for MDBookConfig {}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<MDBookConfig>(format)?;
    if let Some(build_dir) = config.build.get("build-dir") {
        return Ok(Some(build_dir.to_string()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_mdbook() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/mdbook/book.toml",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("build")))
    }
}
