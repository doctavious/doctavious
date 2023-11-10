// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>

use std::collections::HashMap;
use serde::Deserialize;
use serde_derive::Serialize;

use crate::backends::LanguageBackends;
use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};

#[derive(Deserialize)]
struct AntoraConfig {
    output: HashMap<String, String>
}

#[derive(Deserialize)]
struct AntoraConfigOutputKeys {
    dir: Option<String>,
}

impl ConfigurationFileDeserialization for AntoraConfig {}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Antora {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Antora {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "antora".to_string(),
                name: "Antora".to_string(),
                website: "https://antora.org/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency {
                            name: "@antora/cli".to_string(),
                        },
                        FrameworkDetectionItem::Dependency {
                            name: "@antora/site-generator".to_string(),
                        },
                    ],
                },
                build: FrameworkBuildSettings {
                    command: "antora generate".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Arg {
                            index: 1,
                            default_value: None,
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--to-dir".to_string(),
                        }),
                    }),
                    output_directory: "build/site".to_string(),
                },
            },
        }
    }
}

impl Default for Antora {
    fn default() -> Self {
        Antora::new(Vec::from(["antora-playbook.yaml".to_string()]))
    }
}

impl FrameworkSupport for Antora {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<AntoraConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dir) = c.output.get("dir") {
                        return dir.to_string()
                    }
                    // if let Some(AntoraConfigOutputKeys { dir: Some(v) }) = c.output {
                    //     return v;
                    // }
                }
                Err(e) => {
                    // log warning/error
                    println!("{}", e);
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Antora;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_antora() {
        let antora = Antora::new(vec![
            "tests/fixtures/framework_configs/antora/antora-playbook.yaml".to_string(),
        ]);

        let output = antora.get_output_dir();
        assert_eq!(output, "./launch")
    }
}
