// mkdocs.yml
// site --> default
// change be changed via site_dir

use serde::{Deserialize};
use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::language::Language;
#[derive(Deserialize)]
struct MKDocsConfig { site_dir: Option<String> }

pub struct MKDocs { info: FrameworkInfo }

impl MKDocs {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "MkDocs",
                website: Some("https://www.mkdocs.org/"),
                configs,
                language: Language::Python,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency { name: "mkdocs" }
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "mkdocs build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "-f",
                            long: "--config-file"
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d",
                            long: "--site-dir"
                        })
                    }),
                    output_directory: "site",
                },
            }
        }
    }
}

impl Default for MKDocs {
    fn default() -> Self {
        MKDocs::new(Some(Vec::from(["mkdocs.yml"])))
    }
}


impl FrameworkSupport for MKDocs {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<MKDocsConfig>(configs) {
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
    use crate::framework::FrameworkSupport;
    use super::MKDocs;

    #[test]
    fn test_hugo() {
        let mkdocs = MKDocs::new(
            Some(vec!["tests/fixtures/framework_configs/mkdocs/mkdocs.yml"])
        );

        let output = mkdocs.get_output_dir();
        assert_eq!(output, "build")
    }

}
