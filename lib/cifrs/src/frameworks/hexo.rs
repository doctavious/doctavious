// output defaults to public
// _config.yml
// public_dir to override
// hexo-cli
// hexo generate
// hexo --config custom.yml

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
struct HexoConfig {
    public_dir: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Hexo {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Hexo {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "hexo".to_string(),
                name: "Hexo".to_string(),
                website: "https://hexo.io/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "hexo".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "hexo generate".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: None,
                    }),
                    output_directory: "public".to_string(),
                },
            },
        }
    }
}

impl Default for Hexo {
    fn default() -> Self {
        Hexo::new(Vec::from(["_config.yml".to_string()]))
    }
}

impl FrameworkSupport for Hexo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<HexoConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dir) = c.public_dir {
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

impl ConfigurationFileDeserialization for HexoConfig {}

#[cfg(test)]
mod tests {
    use super::Hexo;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_hexo() {
        let hexo = Hexo::new(vec![
            "tests/fixtures/framework_configs/hexo/_config.yml".to_string(),
        ]);

        let output = hexo.get_output_dir();
        assert_eq!(output, "build")
    }
}
