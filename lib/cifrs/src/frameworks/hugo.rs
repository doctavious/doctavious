// config.toml/yaml/json
// multiple can be used
// also has a config directory
// has options that would need to be merged. how to handle?
// hugo command
// hugo -d, --destination

// /public
// can be changed via publishDir

use serde::Deserialize;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::language::Language;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct HugoConfig {
    publish_dir: Option<String>,
}

pub struct Hugo {
    info: FrameworkInfo,
}

impl Hugo {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Hexo",
                website: Some("https://gohugo.io/"),
                configs,
                language: Language::Go,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config {
                        content: Some("baseURL"),
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "hugo",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config",
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--destination",
                        }),
                    }),
                    output_directory: "/public",
                },
            },
        }
    }
}

impl Default for Hugo {
    fn default() -> Self {
        Hugo::new(Some(Vec::from([
            "config.json",
            "config.toml",
            "config.yaml",
            "hugo.json",
            "hugo.toml",
            "hugo.yaml",
        ])))
    }
}

impl FrameworkSupport for Hugo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<HugoConfig>(configs) {
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
        let hugo = Hugo::new(Some(vec![
            "tests/fixtures/framework_configs/hugo/config.toml",
        ]));

        let output = hugo.get_output_dir();
        assert_eq!(output, "build")
    }
}
