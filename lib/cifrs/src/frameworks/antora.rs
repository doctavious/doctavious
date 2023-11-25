// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::framework::FrameworkMatchingStrategy::Any;
// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::CifrsResult;

#[derive(Deserialize)]
pub struct AntoraConfig {
    output: HashMap<String, String>,
}

#[derive(Deserialize)]
struct AntoraConfigOutputKeys {
    dir: Option<String>,
}


impl FrameworkConfiguration for AntoraConfig { }

impl AntoraConfig {
    // pub fn get_output(configs: Vec<PathBuf>) -> CifrsResult<String> {
    //     let s = AntoraConfig::read_config_files::<Self>(&configs);
    // }
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<AntoraConfig>(format)?;
    if let Some(dir) = config.output.get("dir") {
        return Ok(Some(dir.to_string()));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_antora() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/antora/antora-playbook.yaml",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("./launch")))
    }
}
