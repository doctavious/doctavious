// docfx.json
// "docfx <docfx_project>/docfx.json"
// _site
// docfx build [-o:<output_path>] [-t:<template folder>]

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
struct DocFxConfig {
    build: HashMap<String, String>,
}

#[derive(Deserialize)]
struct DocFxConfigBuild {
    dest: String,
}

impl FrameworkConfiguration for DocFxConfig {}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<DocFxConfig>(format)?;
    if let Some(dest) = config.build.get("dest") {
        return Ok(Some(dest.to_string()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_docfx() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/docfx/docfx.json",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("dist")))
    }
}
