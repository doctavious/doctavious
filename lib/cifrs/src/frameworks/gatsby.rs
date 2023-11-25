// gatsby-config.ts // gatsby-config.js

// /public
// people can use gatsby-plugin-output to change output dir

// gatsby build

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

//read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::{
    find_array_element, get_array_property, get_assignment_obj, get_obj_property,
    get_string_property_value,
};
use crate::{CifrsError, CifrsResult};

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct GatsbyConfig {
    output: String,
}

impl FrameworkConfiguration for GatsbyConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        if let Some(obj) = get_assignment_obj(program) {
            if let Some(plugins) = get_array_property(obj, "plugins") {
                if let Some(resolve_elem) =
                    find_array_element(plugins, "resolve", "gatsby-plugin-output")
                {
                    if let Some(options) = get_obj_property(resolve_elem, "options") {
                        if let Some(output) =
                            get_string_property_value(&options.props, "publicPath")
                        {
                            return Ok(Self { output });
                        }
                    }
                }
            }
        }

        Err(CifrsError::InvalidConfig("gatsby".to_string()))
    }
}

pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<GatsbyConfig>(format)?;
    Ok(Some(config.output))
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_gatsby() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/gatsby/gatsby-config.js",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("dist")))
    }
}
