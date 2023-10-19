// conf.py -- <sourcedir>/conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use std::env;

use crate::framework::{
    ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::language::Language;

pub struct Sphinx {
    info: FrameworkInfo,
}

impl Sphinx {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Sphinx",
                website: Some("https://www.sphinx-doc.org/en/master/"),
                configs,
                language: Language::Python,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Config { content: None }],
                },
                build: FrameworkBuildSettings {
                    command: "sphinx-build",
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg {
                            index: 1,
                            default_value: Some("docs"),
                        }),
                        config: None,
                        output: Some(FrameworkBuildArg::Arg {
                            index: 2,
                            default_value: None,
                        }), // TODO: should we default?
                    }),
                    // TODO: must be passed in to command which presents a problem if we dont know
                    // where the build script is
                    output_directory: "docs/_build",
                },
            },
        }
    }
}

impl Default for Sphinx {
    fn default() -> Self {
        // this is relative to source and i dont think we need it as it doesnt help with build
        // TODO: should we remove?
        Sphinx::new(Some(vec!["conf.py"]))
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
        let sphinx = Sphinx::new(Some(vec![
            "tests/fixtures/framework_configs/sphinx/config.py",
        ]));

        let output = sphinx.get_output_dir();
        assert_eq!(output, "docs/_build")
    }

    #[test]
    fn should_use_env_var_when_present() {
        temp_env::with_var("BUILDDIR", Some("build"), || {
            let sphinx = Sphinx::new(Some(vec![
                "tests/fixtures/framework_configs/sphinx/config.py",
            ]));

            let output = sphinx.get_output_dir();
            assert_eq!(output, "build")
        });
    }
}
