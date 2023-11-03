// .vuepress/config.js
// inside docs directory
// which should export a JavaScript object:
// You can also use YAML (.vuepress/config.yml) or TOML (.vuepress/config.toml) formats for the configuration file.
// package.json -> "docs:build": "vuepress build docs"

// vuepress build [targetDir] -d, --dest <dest>

// .vuepress/dist
// can be configured via the dest field

// .vuepress/config.js
// .vuepress/config.yml
// .vuepress/config.toml
// .vuepress/config.ts

use serde::Deserialize;
use serde_derive::Serialize;
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::language::Language;
use crate::{CifrsError, CifrsResult};
use crate::backends::LanguageBackends;

#[derive(Deserialize)]
struct VuePressConfig {
    dest: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct VuePress {
    #[serde(flatten)]
    info: FrameworkInfo,
}

impl VuePress {
    fn new(configs: Vec<String>) -> Self {
        Self {
            info: FrameworkInfo {
                id: "vuepress".to_string(),
                name: "VuePress".to_string(),
                website: "https://vuepress.vuejs.org/".to_string(),
                configs,
                // language: Language::Javascript,
                language: LanguageBackends::JavaScript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "vuepress".to_string() }],
                },
                build: FrameworkBuildSettings {
                    command: "vuepress build".to_string(),
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg {
                            index: 1,
                            default_value: Some("docs".to_string()),
                        }),
                        config: Some(FrameworkBuildArg::Option {
                            short: "-c".to_string(),
                            long: "--config".to_string(),
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d".to_string(),
                            long: "--dest".to_string(),
                        }),
                    }),
                    output_directory: ".vuepress/dist".to_string(),
                },
            },
        }
    }
}

impl Default for VuePress {
    fn default() -> Self {
        VuePress::new(vec![
            ".vuepress/config.js".to_string(),
            ".vuepress/config.yml".to_string(),
            ".vuepress/config.toml".to_string(),
            ".vuepress/config.ts".to_string(),
        ])
    }
}

impl FrameworkSupport for VuePress {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if !self.info.configs.is_empty() {
            match read_config_files::<VuePressConfig>(&self.info.configs) {
                Ok(c) => {
                    if let Some(dest) = c.dest {
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

impl ConfigurationFileDeserialization for VuePressConfig {
    fn from_js_module(program: &Program) -> CifrsResult<Self> {
        // TODO: try and simplify
        println!("{}", serde_json::to_string(&program)?);
        if let Some(module) = program.as_module() {
            let dest = module.get_property_as_string("dest");
            if dest.is_some() {
                return Ok(Self { dest });
            }
            // TODO: get defineConfig?
            // for item in &module.body {
            //     // TODO: this should also look at ExportExpr
            //     if let Some(ExportDefaultExpr(export_expression)) = item.as_module_decl() {
            //         if let Some(call) = export_expression.expr.as_call() {
            //             if is_call_ident(&call, "defineConfig") {
            //                 let dest = get_call_string_property(&call, "dest");
            //                 if dest.is_some() {
            //                     return Ok(Self {
            //                         dest
            //                     });
            //                 }
            //             }
            //         }
            //     } else if let Some(Expr(stmt)) = item.as_stmt() {
            //         let expression = &*stmt.expr;
            //         if let Some(assign) = expression.as_assign() {
            //             let rhs = &*assign.right;
            //             if let Some(obj) = rhs.as_object() {
            //                 let dest = get_string_property_value(&obj.props, "dest");
            //                 if dest.is_some() {
            //                     return Ok(Self {
            //                         dest
            //                     });
            //                 }
            //             }
            //         }
            //     }
            // }
        }
        Err(CifrsError::InvalidConfig("vuepress".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::VuePress;
    use crate::framework::FrameworkSupport;

    #[test]
    fn test_vuepress() {
        let configs = [
            "tests/fixtures/framework_configs/vuepress/config.js".to_string(),
            "tests/fixtures/framework_configs/vuepress/config.toml".to_string(),
            "tests/fixtures/framework_configs/vuepress/config.ts".to_string(),
        ];
        for config in configs {
            let vuepress = VuePress::new(vec![config]);

            let output = vuepress.get_output_dir();
            assert_eq!(output, String::from("build"))
        }
    }
}
