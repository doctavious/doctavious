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
use serde_derive::Serialize;

use crate::backends::LanguageBackends;
use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HugoConfig {
    publish_dir: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Hugo {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Hugo {
    fn new(configs: Vec<PathBuf>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "hexo".to_string(),
                name: "Hexo".to_string(),
                website: "https://gohugo.io/".to_string(),
                configs,
                // language: Language::Go,
                backend: LanguageBackends::Go,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config {
                        content: Some("baseURL".to_string()),
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "hugo".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--destination".to_string(),
                        }),
                    }),
                    output_directory: "/public".to_string(),
                },
            },
        }
    }
}

impl Default for Hugo {
    fn default() -> Self {
        Hugo::new(Vec::from([
            "config.json".into(),
            "config.toml".into(),
            "config.yaml".into(),
            "hugo.json".into(),
            "hugo.toml".into(),
            "hugo.yaml".into(),
        ]))
    }
}

impl FrameworkSupport for Hugo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<HugoConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dir) = c.publish_dir {
                        return dir;
                    }
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

impl ConfigurationFileDeserialization for HugoConfig {}

#[cfg(test)]
mod tests {
    use super::Hugo;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_hugo() {
        let hugo = Hugo::new(vec![
            "tests/fixtures/framework_configs/hugo/config.toml".into()
        ]);

        let output = hugo.get_output_dir();
        assert_eq!(output, "build")
    }
}
