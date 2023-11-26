// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::js_module::{
    get_assignment_function, get_function_return_obj, get_obj_property, get_string_property_value,
};
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct EleventyConfig {
    output: String,
}

impl FrameworkConfiguration for EleventyConfig {
    type Config = Self;

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(func) = get_assignment_function(program) {
            if let Some(return_obj) = get_function_return_obj(func) {
                if let Some(dir_prop) = get_obj_property(return_obj, "dir") {
                    if let Some(output) = get_string_property_value(&dir_prop.props, "output") {
                        return Ok(Self { output });
                    }
                }
            }
        }

        Err(CifrsError::InvalidConfig("eleventy".to_string()))
    }

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: Some(config.output.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::eleventy::EleventyConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_eleventy() {
        let config =
            EleventyConfig::get_config("tests/fixtures/framework_configs/eleventy/.eleventy.js")
                .unwrap();

        assert_eq!(config.output_dir, Some(String::from("dist")))
    }
}
