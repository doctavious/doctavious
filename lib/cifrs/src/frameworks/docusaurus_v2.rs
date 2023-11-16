// docusaurus.config.js
// npm run build / docusaurus build
// build directory
// Both build/serve commands take an output dir option, and there's even a --build option on the serve command. We don't plan to add output dir to the config sorry

// docusaurus v1
// docusaurus-start
// website/siteConfig.js
// publish directory -> website/build/<projectName>
// where projectName is the value you defined in your siteConfig.js

// vercel just sees if there is a single file (directory) and uses it
// Code
// If there is only one file in it that is a dir we'll use it as dist dir
// if (content.length === 1 && content[0].isDirectory()) {
// return join(base, content[0].name);
// }

// docusaurus v2
// docusaurus build --out-dir
// docusaurus.config.js - doesnt contain output
// defaults to build

// TODO: support monorepo

use std::path::PathBuf;

use serde::Deserialize;
use serde_derive::Serialize;

use crate::backends::LanguageBackends;
use crate::framework::{
    ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct DocusaurusV2 {
    #[serde(flatten)]
    info: FrameworkInfo,
}
impl DocusaurusV2 {
    // there is a saying that if things are hard to test then the design sucks (aka is wrong)
    // and that might be true here. Testing that we can get output directory from a config,
    // specifically a JS config, is hard. That is, configs have static names that we want to search
    // for but the contents can have different structures that we ultimately want to test for.
    // This forces us to have test config file names that differ from the predefined ones we would
    // look for outside testing. I dont have a better idea on how to do this.
    fn new(configs: Vec<PathBuf>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "docusarus-v2".to_string(),
                name: "Docusaurus 2".to_string(),
                website: "https://docusaurus.io/".to_string(),
                configs,
                // project_file: None,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency {
                        name: "@docusaurus/core".to_string(),
                    }],
                },
                build: FrameworkBuildSettings {
                    command: "docusaurus build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--out-dir".to_string(),
                        }),
                    }),
                    output_directory: "build".to_string(),
                },
            },
        }
    }
}

impl Default for DocusaurusV2 {
    fn default() -> Self {
        DocusaurusV2::new(Vec::from(["docusaurus.config.js".into()]))
    }
}

impl FrameworkSupport for DocusaurusV2 {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    // Vercel checks if there is a a single file (directory) under build and if so uses it
    // otherwise uses build
    fn get_output_dir(&self) -> String {
        self.info.build.output_directory.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::DocusaurusV2;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_docusaurus() {
        // TODO: lets just put file contents in tests and write to tempdir + known file
        let docusaurus = DocusaurusV2::new(vec![
            "tests/fixtures/framework_configs/docusaurus2/docusaurus.config.js".into(),
        ]);

        let output = docusaurus.get_output_dir();
        assert_eq!(output, "build")
    }
}
