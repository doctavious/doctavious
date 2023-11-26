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

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::CifrsResult;

#[derive(Deserialize)]
pub struct DocusaurusConfig;

impl FrameworkConfiguration for DocusaurusConfig {
    type Config = Self;

    fn from_js_module(_program: &Program) -> CifrsResult<Self> {
        Ok(Self {})
    }

    fn convert_to_common_config(_config: &Self::Config) -> FrameworkConfigFile {
        // TODO: verify this implementation
        // Vercel checks if there is a a single file (directory) under build and if so uses it
        // otherwise uses build.
        // Current implementation doesnt give us access to framework info
        // would need to pass in framework info (to get default output dir) and cwd path
        FrameworkConfigFile { output_dir: None }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::docusaurus_v2::DocusaurusConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_docusaurus() {
        let config = DocusaurusConfig::get_config(
            "tests/fixtures/framework_configs/docusaurus_v2/docusaurus.config.js",
        )
        .unwrap();

        assert_eq!(config.output_dir, None)
    }
}
