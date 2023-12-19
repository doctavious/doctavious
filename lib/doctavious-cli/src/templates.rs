use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use minijinja::functions::Function;
use minijinja::{context, render, AutoEscape, Environment};
use serde::Serialize;
use serde_json::{to_value, Value};

// use tera::{Context, Function, Tera};
use crate::{CliResult, DoctaviousCliError};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TemplateContext {
    data: BTreeMap<String, Value>,
}

impl TemplateContext {
    /// Initializes an empty context
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    /// Takes a serde-json `Value` and convert it into a `Context` with no overhead/cloning.
    pub fn from_value(obj: Value) -> CliResult<Self> {
        match obj {
            Value::Object(m) => {
                let mut data = BTreeMap::new();
                for (key, value) in m {
                    data.insert(key, value);
                }
                Ok(TemplateContext { data })
            }
            _ => Err(DoctaviousCliError::TemplateContextError()),
        }
    }

    /// Converts the `val` parameter to `Value` and insert it into the context.
    ///
    /// Panics if the serialization fails.
    ///
    /// ```rust
    /// # use templates::TemplateContext;
    /// let mut context = templates::TemplateContext::new();
    /// context.insert("number_users", &42);
    /// ```
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        self.data.insert(key.into(), to_value(val).unwrap());
    }

    /// Takes something that impl Serialize and create a context with it.
    /// Meant to be used if you have a hashmap or a struct and don't want to insert values
    /// one by one in the context.
    pub fn from_serialize(value: impl Serialize) -> CliResult<Self> {
        let obj = to_value(value)?;
        TemplateContext::from_value(obj)
    }

    /// Returns the value at a given key index.
    pub fn get(&self, index: &str) -> Option<&Value> {
        self.data.get(index)
    }

    /// Remove a key from the context, returning the value at the key if the key was previously
    /// inserted into the context.
    pub fn remove(&mut self, index: &str) -> Option<Value> {
        self.data.remove(index)
    }

    /// Checks if a value exists at a specific index.
    pub fn contains_key(&self, index: &str) -> bool {
        self.data.contains_key(index)
    }
}

#[derive(Debug)]
pub struct Templates<'a> {
    // tera: Tera,
    env: Environment<'a>,
}

impl<'a> Templates<'a> {
    /// Constructs a new instance.
    pub fn new() -> CliResult<Self> {
        // let tera = Tera::default();

        return Ok(Self {
            env: Environment::new(),
        });
    }

    pub fn new_with_templates(templates: HashMap<&'a str, String>) -> CliResult<Self> {
        // let mut tera = Tera::default();
        let mut env = Environment::new();
        for (k, v) in templates {
            if let Err(e) = env.add_template_owned(k, v) {
                return if let Some(error_source) = e.source() {
                    Err(DoctaviousCliError::TemplateParseError(
                        error_source.to_string(),
                    ))
                } else {
                    Err(DoctaviousCliError::TemplateError(e))
                };
            }

            // if let Err(e) = tera.add_raw_template(k, v.as_str()) {
            //     return if let Some(error_source) = e.source() {
            //         Err(DoctaviousCliError::TemplateParseError(
            //             error_source.to_string(),
            //         ))
            //     } else {
            //         Err(DoctaviousCliError::TemplateError(e))
            //     };
            // }
        }

        return Ok(Self { env });
    }

    // TODO: probably makes sense to make this Into<&str, String>?
    /// Renders the template.
    pub fn render(&self, template: &str, context: &TemplateContext) -> CliResult<String> {
        // let tera_context = Context::from_serialize(&context.data)?;
        let tmpl = self.env.get_template(template).unwrap();
        Ok(tmpl.render(context!(context))?)
        // return Ok(self.tera.render(template, &tera_context)?);
    }

    // pub fn register_function<F: Function + 'static>(
    //     &mut self,
    //     name: &str,
    //     function: F,
    // ) {
    //     self.tera.register_function(name, function)
    // }

    pub fn one_off(template: &str, context: &TemplateContext, escape: bool) -> CliResult<String> {
        let mut env = Environment::new();
        env.set_auto_escape_callback(|_| AutoEscape::Html);
        Ok(render!(in env, template, context))
        // let tera_context = Context::from_serialize(&context.data)?;
        // println!("{:?}", tera_context);
        // return Ok(Tera::one_off(template, &tera_context, escape)?);
    }
}

// TODO: This is wrong for ADRs init as it doesnt look for a custom init template
// does it need to take in name?
pub(crate) fn get_template(
    dir: &str,
    extension: &str,
    default_template_path: &str,
) -> PathBuf {
    let custom_template =
        Path::new(dir).join("template").with_extension(extension);

    let template = if custom_template.exists() {
        custom_template
    } else {
        Path::new(default_template_path)
            .with_extension(extension.to_string())
            .to_path_buf()
    };

    return template;
}

pub(crate) fn get_template_content(
    dir: &str,
    extension: &str,
    default_template_path: &str,
) -> String {
    let template_path = get_template(dir, extension, default_template_path);
    // TODO: we shouldnt panic here
    return fs::read_to_string(&template_path).expect(&format!(
        "failed to read file {}.",
        &template_path.to_string_lossy()
    ));
}

// TODO: tests
#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::{env, fs};

    // TODO: invalid template should return valid error
}
