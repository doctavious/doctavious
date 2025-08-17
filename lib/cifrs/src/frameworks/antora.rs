// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>

use std::collections::HashMap;

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFileSettings, FrameworkConfiguration};

#[derive(Debug, Deserialize)]
pub struct AntoraConfig {
    output: HashMap<String, String>,
}

#[derive(Deserialize)]
struct AntoraConfigOutputKeys {
    dir: Option<String>,
}

impl FrameworkConfiguration for AntoraConfig {
    type Config = Self;

    fn get_config_file_settings(config: &Self::Config) -> FrameworkConfigFileSettings {
        FrameworkConfigFileSettings {
            output_dir: config.output.get("dir").cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::FrameworkConfiguration;
    use crate::frameworks::antora::AntoraConfig;

    #[test]
    fn test_antora() {
        let config = AntoraConfig::get_config(
            "tests/fixtures/framework_configs/antora/antora-playbook.yaml",
        )
        .unwrap();

        assert_eq!(config.settings.output_dir, Some(String::from("./launch")))
    }
}
