// antora-playbook.yml
// antora antora-playbook.yml or npx antora antora-playbook.yml
// build/site
// change change via dir

// antora generate <playbook> --to-dir <dir>


use serde::{Deserialize};
use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::language::Language;

#[derive(Deserialize)]
struct AntoraConfigOutputKeys { dir: Option<String> }

#[derive(Deserialize)]
struct AntoraConfig { output: Option<AntoraConfigOutputKeys> }

pub struct Antora { info: FrameworkInfo }

impl Antora {

    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Antora",
                website: Some("https://antora.org/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::Any,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency { name: "@antora/cli" },
                        FrameworkDetectionItem::Dependency { name: "@antora/site-generator" }
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "antora generate",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Arg { index: 1, default_value: None }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--to-dir"
                        })
                    }),
                    output_directory: "build/site",
                },
            },
        }
    }

}

impl Default for Antora {
    fn default() -> Self {
        Antora::new(Some(Vec::from(["antora-playbook.yaml"])))
    }
}

impl FrameworkSupport for Antora {

    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AntoraConfig>(configs) {
                Ok(c) => {
                    if let Some(AntoraConfigOutputKeys {dir: Some(v)}) = c.output {
                        return v;
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

impl ConfigurationFileDeserialization for AntoraConfig {}


#[cfg(test)]
mod tests {
    use crate::framework::FrameworkSupport;
    use super::Antora;

    #[test]
    fn test_antora() {
        let antora = Antora::new(
            Some(vec!["tests/fixtures/framework_configs/antora/antora-playbook.yaml"])
        );

        let output = antora.get_output_dir();
        assert_eq!(output, "./launch")
    }

}
