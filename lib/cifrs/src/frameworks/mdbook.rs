// book.toml
// ./book -> default
// change be changed via build.build-dir

use std::collections::HashMap;
use serde::Deserialize;
use serde_derive::Serialize;

use crate::backends::LanguageBackends;
use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct MDBookBuildOptions {
    build_dir: Option<String>,
}

#[derive(Deserialize)]
struct MDBookConfig {
    // build: Option<MDBookBuildOptions>,
    build: HashMap<String, String>
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct MDBook {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl MDBook {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "mdbook".to_string(),
                name: "mdBook".to_string(),
                website: "https://rust-lang.github.io/mdBook/".to_string(),
                configs,
                // language: Language::Rust,
                backend: LanguageBackends::Rust,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config { content: None }],
                },
                build: FrameworkBuildSettings {
                    command: "mdbook build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: None,
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d".to_string(),
                            long: "--dest-dir".to_string(),
                        }),
                    }),
                    output_directory: "./book".to_string(),
                },
            },
        }
    }
}

impl Default for MDBook {
    fn default() -> Self {
        MDBook::new(Vec::from(["book.toml".to_string()]))
    }
}

impl FrameworkSupport for MDBook {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }
    
    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<MDBookConfig>(&self.info.configs) {
                Ok(c) => {
                    // if let Some(MDBookBuildOptions { build_dir: Some(v) }) = c.build {
                    //     return v;
                    // }
                    if let Some(build_dir) = c.build.get("build-dir") {
                        return build_dir.to_string();
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
    use super::MDBook;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_mdbook() {
        let book = MDBook::new(vec![
            "tests/fixtures/framework_configs/mdbook/book.toml".to_string()
        ]);

        let output = book.get_output_dir();
        assert_eq!(output, "build")
    }
}
