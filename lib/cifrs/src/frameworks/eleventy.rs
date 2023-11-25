// .eleventy.js
//
// .eleventy.js
// eleventy.config.js Added in v2.0.0-beta.1
// eleventy.config.cjs Added in v2.0.0-beta.1

// dir.output
// defaults to _site

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::{
    get_assignment_function, get_function_return_obj, get_obj_property, get_string_property_value,
};
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct EleventyConfig {
    output: String,
}

impl FrameworkConfiguration for EleventyConfig {
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
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<EleventyConfig>(format)?;
    Ok(Some(config.output))
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_eleventy() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/eleventy/.eleventy.js",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("dist")))
    }
}
