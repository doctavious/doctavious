// output defaults to public
// _config.yml
// public_dir to override
// hexo-cli
// hexo generate
// hexo --config custom.yml

use serde::{Deserialize};
use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::language::Language;

#[derive(Deserialize)]
struct HexoConfig { public_dir: Option<String> }

pub struct Hexo { info: FrameworkInfo }

impl Hexo {

    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Hexo",
                website: Some("https://hexo.io/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency { name: "hexo"}
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "hexo generate",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config"
                        }),
                        output: None
                    }),
                    output_directory: "public",
                },
            }
        }
    }

}

impl Default for Hexo {
    fn default() -> Self {
        Hexo::new(Some(Vec::from(["_config.yml"])))
    }
}

impl FrameworkSupport for Hexo {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<HexoConfig>(configs) {
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
    use crate::framework::FrameworkSupport;
    use super::Hexo;

    #[test]
    fn test_hexo() {
        let hexo = Hexo::new(
            Some(vec!["tests/fixtures/framework_configs/hexo/_config.yml"])
        );

        let output = hexo.get_output_dir();
        assert_eq!(output, "build")
    }

}
