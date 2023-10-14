// astro.config.mjs
// "npm run build"
// astro build
// outDir: './my-custom-build-directory'
// defaults to "./dist"


use serde::{Deserialize};
use swc_ecma_ast::{Program};
use crate::{CifrsError, CifrsResult};
use crate::framework::{ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs, FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy, FrameworkSupport, read_config_files};
use crate::js_module::{get_call_expression, get_call_string_property};
use crate::language::Language;

pub struct Astro { info: FrameworkInfo }

impl Astro {

    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "Astro",
                website: Some("https://astro.build"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![
                        FrameworkDetectionItem::Dependency { name: "astro" }
                    ]
                },
                build: FrameworkBuildSettings {
                    command: "astro build",
                    command_args: Some(FrameworkBuildArgs {
                        source: None,
                        config: Some(FrameworkBuildArg::Option {
                            short: "",
                            long: "--config",
                        }),
                        output: None,
                    }),
                    output_directory: "./dist",
                },
            }
        }
    }
}

impl Default for Astro {
    fn default() -> Self {
        Astro::new(Some(Vec::from(["astro.config.mjs"])))
    }
}

impl FrameworkSupport for Astro {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<AstroConfig>(configs) {
                Ok(c) => {
                    return c.output
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

#[derive(Deserialize)]
struct AstroConfig { output: String }

impl ConfigurationFileDeserialization for AstroConfig {

    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: do we care what its called?
        let define_config = get_call_expression(program, "defineConfig");
        if let Some(define_config) = define_config {
            if let Some(val) = get_call_string_property(define_config, "outDir") {
                return Ok(Self {
                    output: val
                });
            }
        }

        Err(CifrsError::InvalidConfig("astro".to_ascii_lowercase()))
    }
}


#[cfg(test)]
mod tests {
    use crate::framework::FrameworkSupport;
    use super::Astro;

    #[test]
    fn test_astro() {
        let astro = Astro::new(
            Some(vec!["tests/fixtures/framework_configs/astro/astro.config.mjs"])
        );

        let output = astro.get_output_dir();
        assert_eq!(output, "./build")
    }

}
