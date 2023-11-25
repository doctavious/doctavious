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

// ConfigurationFileDeserialization
use crate::backends::LanguageBackends;
use crate::framework::{
    FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkConfigurationFormat,
    FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy,
    FrameworkSupport,
};
use crate::CifrsResult;

// TODO: verify this implementation
// Vercel checks if there is a a single file (directory) under build and if so uses it
// otherwise uses build
pub fn get_output_dir(_format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    Ok(None)
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_docusaurus() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/docusaurus_v2/docusaurus.config.js",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, None)
    }
}
