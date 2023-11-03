// docfx.json
// "docfx <docfx_project>/docfx.json"
// _site
// docfx build [-o:<output_path>] [-t:<template folder>]

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
struct DocFxConfigBuild {
    dest: String,
}

#[derive(Deserialize)]
struct DocFxConfig {
    build: DocFxConfigBuild,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DocFx {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl DocFx {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "docfx".to_string(),
                name: "DocFX".to_string(),
                website: "https://dotnet.github.io/docfx/".to_string(),
                configs,
                // language: Language::CSharp, // F# will be supported in the future.
                language: LanguageBackends::DotNet,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config { content: None }],
                },
                build: FrameworkBuildSettings {
                    command: "docfx build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "-o".to_string(),
                            long: "".to_string(),
                        }),
                    }),
                    output_directory: "_site".to_string(),
                },
            },
        }
    }
}

impl Default for DocFx {
    fn default() -> Self {
        DocFx::new(Vec::from(["docfx.json".to_string()]))
    }
}

impl FrameworkSupport for DocFx {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<DocFxConfig>(&self.info.configs) {
                Ok(c) => return c.build.dest,
                Err(e) => {
                    // log warning/error
                    println!("{}", e);
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

impl ConfigurationFileDeserialization for DocFxConfig {}

#[cfg(test)]
mod tests {
    use crate::framework::FrameworkSupport;
    use crate::frameworks::docfx::DocFx;

    #[test]
    fn test_docfx() {
        let docfx = DocFx::new(vec![
            "tests/fixtures/framework_configs/docfx/docfx.json".to_string(),
        ]);

        let output = docfx.get_output_dir();
        assert_eq!(output, "dist")
    }
}
