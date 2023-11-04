// .vitepress/config.js
// which should export a JavaScript object:
// .vitepress/dist
// can be configured via the outDir field
// "docs:build": "vitepress build docs",
// do we allow to customize the script we look for? ex: instead of 'build' we look for 'docs:build'
// package.json

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildSettings,
    FrameworkDetectionItem, FrameworkDetector, FrameworkInfo, FrameworkMatchingStrategy,
    FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

#[derive(Deserialize)]
struct VitePressConfig {
    output: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct VitePress {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl VitePress {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "vitepress".to_string(),
                name: "VitePress".to_string(),
                website: "https://vitepress.vuejs.org/".to_string(),
                configs,
                // language: Language::Javascript,
                backend: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "vitepress".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "vitepress build docs".to_string(),
                    command_args: None, // TODO: check
                    output_directory: "docs/.vitepress/dist".to_string(),
                },
            },
        }
    }
}

impl Default for VitePress {
    fn default() -> Self {
        VitePress::new(vec![
            ".vitepress/config.cjs".to_string(),
            ".vitepress/config.js".to_string(),
            ".vitepress/config.mjs".to_string(),
            ".vitepress/config.mts".to_string(),
            ".vitepress/config.ts".to_string(),
        ])
    }
}

impl FrameworkSupport for VitePress {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<VitePressConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dest) = c.output {
                        return dest;
                    }
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

impl ConfigurationFileDeserialization for VitePressConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        println!("{}", serde_json::to_string(&program)?);
        if let Some(module) = program.as_module() {
            let output = module.get_property_as_string("outDir");
            if output.is_some() {
                return Ok(Self { output });
            }
            // for item in &module.body {
            //     if let Some(Decl(decl)) = item.as_stmt() {
            //         if let Some(variable_decl) = decl.as_var() {
            //             let variable = &**variable_decl;
            //             for declaration in &variable.decls {
            //                let output = get_variable_property_as_string(&declaration, "outDir");
            //                 if output.is_some() {
            //                     return Ok(Self {
            //                         output
            //                     });
            //                 }
            //             }
            //         }
            //     } else if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
            //         if let Some(call) = export_expression.expr.as_call() {
            //             if is_call_ident(&call, "defineConfig") {
            //                 let output = get_call_string_property(&call, "outDir");
            //                 if output.is_some() {
            //                     return Ok(Self {
            //                         output
            //                     });
            //                 }
            //             }
            //         }
            //     }
            // }
        }
        Err(CifrsError::InvalidConfig("vitepress".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::VitePress;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_vitepress() {
        let configs = [
            "tests/fixtures/framework_configs/vitepress/config.js".to_string(),
            "tests/fixtures/framework_configs/vitepress/config.ts".to_string(),
        ];
        for config in configs {
            let vitepress = VitePress::new(vec![config]);

            let output = vitepress.get_output_dir();
            assert_eq!(output, String::from("build"))
        }
    }
}
