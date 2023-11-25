// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"

use std::path::PathBuf;

use serde::Deserialize;
use swc_ecma_ast::Program;

// read_config_files, ConfigurationFileDeserialization,
use crate::framework::{
    deser_config, FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport,
};
use crate::js_module::{get_call_expression, get_call_string_property};
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct AstroConfig {
    output: String,
}

impl FrameworkConfiguration for AstroConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: do we care what its called?
        let define_config = get_call_expression(program, "defineConfig");
        if let Some(define_config) = define_config {
            if let Some(val) = get_call_string_property(define_config, "outDir") {
                return Ok(Self { output: val });
            }
        }
        
        Err(CifrsError::InvalidConfig("astro".to_ascii_lowercase()))
    }

}


pub fn get_output_dir(format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    let config = deser_config::<AstroConfig>(format)?;
    Ok(Some(config.output))
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_astro() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/astro/astro.config.mjs",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, Some(String::from("./build")))
    }
}
