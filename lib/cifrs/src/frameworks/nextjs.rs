// next.config.js / next.config.mjs
// this is a regular Node.js module
// could also look at package.json -> scripts -> "build": "next build",

// .next -> default directory
// change be changed via distDir

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct NextJSConfig {
    output: String,
}


impl FrameworkConfiguration for NextJSConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(module) = program.as_module() {
            if let Some(output) = module.get_property_as_string("distDir") {
                return Ok(Self { output });
            }
        }
        Err(CifrsError::InvalidConfig("nextjs".to_string()))
    }
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<NextJSConfig>(format)?;
    Ok(Some(config.output))
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_nextjs() {
        for path in [
            "tests/fixtures/framework_configs/nextjs/next_js_v1.mjs",
            "tests/fixtures/framework_configs/nextjs/next_js_v2.mjs",
        ] {
            let config = FrameworkConfigurationFormat::from_path(path).unwrap();
            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        }
    }
}
