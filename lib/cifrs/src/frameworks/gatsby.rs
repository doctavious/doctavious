// gatsby-config.ts // gatsby-config.js

// /public
// people can use gatsby-plugin-output to change output dir

// gatsby build

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildSettings,
    FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy,
    FrameworkSupport,
};
use crate::js_module::{
    find_array_element, get_array_property, get_assignment_obj, get_obj_property,
    get_string_property_value,
};
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

// TODO: given there is no option to override does it make sense to still enforce Deserialize
// and ConfigurationFileDeserialization?
// I suppose we can determine if gatsby-plugin-output is in the plugins and grab it from there
#[derive(Deserialize)]
struct GatsbyConfig {
    output: String,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Gatsby {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Gatsby {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "gatsby".to_string(),
                name: "Gatsby".to_string(),
                website: "https://www.gatsbyjs.com/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "gatsby".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "gatsby build".to_string(),
                    command_args: None,
                    output_directory: "/public".to_string(),
                },
            },
        }
    }
}

impl Default for Gatsby {
    fn default() -> Self {
        Gatsby::new(Vec::from([
            "gatsby-config.js".to_string(),
            "gatsby-config.ts".to_string()]
        ))
    }
}

impl FrameworkSupport for Gatsby {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<GatsbyConfig>(&self.info.configs) {
                Ok(c) => {
                    return c.output;
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

impl ConfigurationFileDeserialization for GatsbyConfig {
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

#[cfg(test)]
mod tests {
    use super::Gatsby;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_gatsby() {
        let gatsby = Gatsby::new(vec![
            "tests/fixtures/framework_configs/gatsby/gatsby-config.js".to_string(),
        ]);

        let output = gatsby.get_output_dir();
        assert_eq!(output, "dist")
    }
}
