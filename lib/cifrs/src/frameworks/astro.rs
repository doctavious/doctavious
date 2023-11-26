// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"

use serde::Deserialize;
use swc_ecma_ast::Program;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::js_module::{get_call_expression, get_call_string_property};
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
pub struct AstroConfig {
    output: String,
}

impl FrameworkConfiguration for AstroConfig {
    type Config = Self;

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

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: Some(config.output.to_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::astro::AstroConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_astro() {
        let config =
            AstroConfig::get_config("tests/fixtures/framework_configs/astro/astro.config.mjs")
                .unwrap();

        assert_eq!(config.output_dir, Some(String::from("./build")))
    }
}
