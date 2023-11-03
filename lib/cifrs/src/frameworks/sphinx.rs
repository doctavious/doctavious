// conf.py -- <sourcedir>/conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use std::env;
use serde_derive::{Deserialize, Serialize};
use crate::backends::LanguageBackends;

use crate::framework::{
    ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::language::Language;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Sphinx {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Sphinx {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "sphinx".to_string(),
                name: "Sphinx".to_string(),
                website: "https://www.sphinx-doc.org/en/master/".to_string(),
                configs,
                // language: Language::Python,
                language: LanguageBackends::Python,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config { content: None }],
                },
                build: FrameworkBuildSettings {
                    command: "sphinx-build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg {
                            index: 1,
                            default_value: Some("docs".to_string()),
                        }),
                        config: None,
                        output: Some(FrameworkBuildArg::Arg {
                            index: 2,
                            default_value: None,
                        }), // TODO: should we default?
                    }),
                    // TODO: must be passed in to command which presents a problem if we dont know
                    // where the build script is
                    output_directory: "docs/_build".to_string(),
                },
            },
        }
    }
}

impl Default for Sphinx {
    fn default() -> Self {
        // this is relative to source and i dont think we need it as it doesnt help with build
        // TODO: should we remove?
        Sphinx::new(vec!["conf.py".to_string()])
    }
}

impl FrameworkSupport for Sphinx {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    // TODO: how to codify this in yaml?
    fn get_output_dir(&self) -> String {
        if let Ok(build_dir) = env::var("BUILDDIR") {
            return build_dir;
        }

        self.info.build.output_directory.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::Sphinx;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_sphinx() {
        let sphinx = Sphinx::new(vec![
            "tests/fixtures/framework_configs/sphinx/config.py".to_string(),
        ]);

        let output = sphinx.get_output_dir();
        assert_eq!(output, "docs/_build")
    }

    #[test]
    fn should_use_env_var_when_present() {
        temp_env::with_var("BUILDDIR", Some("build"), || {
            let sphinx = Sphinx::new(vec![
                "tests/fixtures/framework_configs/sphinx/config.py".to_string(),
            ]);

            let output = sphinx.get_output_dir();
            assert_eq!(output, "build")
        });
    }
}
