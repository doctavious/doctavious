// _config.yml or _config.toml
// _site/
// change be changed via destination

// destination: DIR
// jekyll build -d, --destination DIR

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
struct JekyllConfig {
    destination: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Jekyll {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Jekyll {
    fn new(configs: Vec<PathBuf>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "jekyll".to_string(),
                name: "Jekyll".to_string(),
                website: "https://jekyllrb.com/".to_string(),
                configs,
                // language: Language::Ruby,
                backend: LanguageBackends::Ruby,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency {
                            name: "jekyll".to_string(),
                        },
                        FrameworkDetectionItem::File {
                            path: "Gemfile".to_string(),
                            content: Some("jekyll_plugins".to_string()),
                        },
                    ],
                },
                build: FrameworkBuildSettings {
                    // bundle exec jekyll build
                    command: "jekyll build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d".to_string(),
                            long: "--destination".to_string(),
                        }),
                    }),
                    output_directory: "_site".to_string(),
                },
            },
        }
    }
}

impl Default for Jekyll {
    fn default() -> Self {
        Jekyll::new(Vec::from(["_config.yml".into(), "_config.toml".into()]))
    }
}

impl FrameworkSupport for Jekyll {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<JekyllConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(destination) = c.destination {
                        return destination;
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

impl ConfigurationFileDeserialization for JekyllConfig {}

#[cfg(test)]
mod tests {
    use super::Jekyll;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_jekyll() {
        let jekyll = Jekyll::new(vec![
            "tests/fixtures/framework_configs/jekyll/_config.yml".into()
        ]);

        let output = jekyll.get_output_dir();
        assert_eq!(output, "build")
    }
}
