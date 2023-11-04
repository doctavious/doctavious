// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::{get_call_expression, get_call_string_property};
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Astro {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl Astro {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "astro".to_string(),
                name: "Astro".to_string(),
                website: "https://astro.build".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "astro".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "astro build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: None,
                    }),
                    output_directory: "./dist".to_string(),
                },
            },
        }
    }
}

impl Default for Astro {
    fn default() -> Self {
        Astro::new(Vec::from(["astro.config.mjs".to_string()]))
    }
}

impl FrameworkSupport for Astro {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<AstroConfig>(&self.info.configs) {
                Ok(c) => return c.output,
                Err(e) => {
                    // log warning/error
                    println!("{}", e);
                }
            }
        }

        self.info.build.output_directory.to_string()
    }
}

#[derive(Deserialize)]
struct AstroConfig {
    output: String,
}

impl ConfigurationFileDeserialization for AstroConfig {
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

#[cfg(test)]
mod tests {
    use super::Astro;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_astro() {
        let astro = Astro::new(vec![
            "tests/fixtures/framework_configs/astro/astro.config.mjs".to_string(),
        ]);

        let output = astro.get_output_dir();
        assert_eq!(output, "./build")
    }
}
