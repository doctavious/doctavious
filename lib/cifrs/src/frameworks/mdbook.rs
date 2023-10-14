// book.toml
// ./book -> default
// change be changed via build.build-dir

use serde::{Deserialize};
use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::language::Language;
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MDBookBuildOptions { build_dir: Option<String> }

#[derive(Deserialize)]
struct MDBookConfig { build: Option<MDBookBuildOptions> }

pub struct MDBook { info: FrameworkInfo }

impl MDBook {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "mdBook",
                website: Some("https://rust-lang.github.io/mdBook/"),
                configs,
                language: Language::Rust,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Config { content: None }
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "mdbook build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d",
                            long: "--dest-dir"
                        })
                    }),
                    output_directory: "./book",
                },
            }
        }
    }
}

impl Default for MDBook {
    fn default() -> Self {
        MDBook::new(Some(Vec::from(["book.toml"])))
    }
}

impl FrameworkSupport for MDBook {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<MDBookConfig>(configs) {
                Ok(c) => {
                    if let Some(MDBookBuildOptions {build_dir: Some(v)}) = c.build {
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

impl ConfigurationFileDeserialization for MDBookConfig {}

#[cfg(test)]
mod tests {
    use crate::framework::FrameworkSupport;
    use super::MDBook;

    #[test]
    fn test_mdbook() {
        let book = MDBook::new(
            Some(vec!["tests/fixtures/framework_configs/mdbook/book.toml"])
        );

        let output = book.get_output_dir();
        assert_eq!(output, "build")
    }

}
