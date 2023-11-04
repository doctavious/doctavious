// mkdocs.yml
// site --> default
// change be changed via site_dir

use serde::Deserialize;
use serde_derive::Serialize;
use crate::backends::LanguageBackends;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::language::Language;
#[derive(Deserialize)]
struct MKDocsConfig {
    site_dir: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MKDocs {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl MKDocs {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "mkdocs".to_string(),
                name: "MkDocs".to_string(),
                website: "https://www.mkdocs.org/".to_string(),
                configs,
                // language: Language::Python,
                backend: LanguageBackends::Python,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "mkdocs".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "mkdocs build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "-f".to_string(),
                            long: "--config-file".to_string(),
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d".to_string(),
                            long: "--site-dir".to_string(),
                        }),
                    }),
                    output_directory: "site".to_string(),
                },
            },
        }
    }
}

impl Default for MKDocs {
    fn default() -> Self {
        MKDocs::new(Vec::from(["mkdocs.yml".to_string()]))
    }
}

impl FrameworkSupport for MKDocs {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<MKDocsConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dir) = c.site_dir {
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

impl ConfigurationFileDeserialization for MKDocsConfig {}

#[cfg(test)]
mod tests {
    use super::MKDocs;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_hugo() {
        let mkdocs = MKDocs::new(vec![
            "tests/fixtures/framework_configs/mkdocs/mkdocs.yml".to_string(),
        ]);

        let output = mkdocs.get_output_dir();
        assert_eq!(output, "build")
    }
}
