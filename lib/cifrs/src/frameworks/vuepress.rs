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
use swc_ecma_ast::Program;

use crate::framework::{
    read_config_files, ConfigurationFileDeserialization, FrameworkBuildArg, FrameworkBuildArgs,
    FrameworkBuildSettings, FrameworkDetectionItem, FrameworkDetector, FrameworkInfo,
    FrameworkMatchingStrategy, FrameworkSupport,
};
use crate::js_module::PropertyAccessor;
use crate::language::Language;
use crate::{CifrsError, CifrsResult};

#[derive(Deserialize)]
struct VuePressConfig {
    dest: Option<String>,
}

pub struct VuePress {
    info: FrameworkInfo,
}

impl VuePress {
    fn new(configs: Option<Vec<&'static str>>) -> Self {
        Self {
            info: FrameworkInfo {
                name: "VuePress",
                website: Some("https://vuepress.vuejs.org/"),
                configs,
                language: Language::Javascript,
                detection: FrameworkDetector {
                    matching_strategy: FrameworkMatchingStrategy::All,
                    detectors: vec![FrameworkDetectionItem::Dependency { name: "vuepress" }],
                },
                build: FrameworkBuildSettings {
                    command: "vuepress build",
                    command_args: Some(FrameworkBuildArgs {
                        source: Some(FrameworkBuildArg::Arg {
                            index: 1,
                            default_value: Some("docs"),
                        }),
                        config: Some(FrameworkBuildArg::Option {
                            short: "-c",
                            long: "--config",
                        }),
                        output: Some(FrameworkBuildArg::Option {
                            short: "-d",
                            long: "--dest",
                        }),
                    }),
                    output_directory: ".vuepress/dist",
                },
            },
        }
    }
}

impl Default for VuePress {
    fn default() -> Self {
        VuePress::new(Some(vec![
            ".vuepress/config.js",
            ".vuepress/config.yml",
            ".vuepress/config.toml",
            ".vuepress/config.ts",
        ]))
    }
}

impl FrameworkSupport for VuePress {
    fn get_info(&self) -> &FrameworkInfo {
        &self.info
    }

    fn get_output_dir(&self) -> String {
        if let Some(configs) = &self.info.configs {
            match read_config_files::<VuePressConfig>(configs) {
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
            "tests/fixtures/framework_configs/vuepress/config.js",
            "tests/fixtures/framework_configs/vuepress/config.toml",
            "tests/fixtures/framework_configs/vuepress/config.ts",
        ];
        for config in configs {
            let vuepress = VuePress::new(Some(vec![config]));

            let output = vuepress.get_output_dir();
            assert_eq!(output, String::from("build"))
        }
    }
}
