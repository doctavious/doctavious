use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::path::PathBuf;

use minijinja::functions::Function;
use minijinja::{AutoEscape, Environment};
use serde::Serialize;
use serde_json::{to_value, Value};

use crate::settings::{
    DEFAULT_ADR_INIT_TEMPLATE_PATH, DEFAULT_ADR_RECORD_TEMPLATE_PATH,
    DEFAULT_ADR_TOC_TEMPLATE_PATH, DEFAULT_RFD_RECORD_TEMPLATE_PATH,
    DEFAULT_TIL_POST_TEMPLATE_PATH, DEFAULT_TIL_README_TEMPLATE_PATH,
};
use crate::{CliResult, DoctaviousCliError};

pub enum TemplateType {
    Adr(AdrTemplateType),
    Rfd,
    Til(TilTemplateType),
}
#[derive(Default)]
pub enum AdrTemplateType {
    Init,
    #[default]
    Record,
    ToC,
}

#[derive(Default)]
pub enum RfdTemplateType {
    #[default]
    Record,
    ToC,
}

pub enum TilTemplateType {
    ReadMe,
    Post,
}

impl TemplateType {
    // TODO: should probably consolidate this with other ways to get template paths
    pub fn get_default_path(&self) -> PathBuf {
        let s = match self {
            TemplateType::Adr(templates) => match templates {
                AdrTemplateType::Init => DEFAULT_ADR_INIT_TEMPLATE_PATH,
                AdrTemplateType::Record => DEFAULT_ADR_RECORD_TEMPLATE_PATH,
                AdrTemplateType::ToC => DEFAULT_ADR_TOC_TEMPLATE_PATH,
            },
            TemplateType::Rfd => DEFAULT_RFD_RECORD_TEMPLATE_PATH,
            TemplateType::Til(templates) => match templates {
                TilTemplateType::ReadMe => DEFAULT_TIL_README_TEMPLATE_PATH,
                TilTemplateType::Post => DEFAULT_TIL_POST_TEMPLATE_PATH,
            },
        };

        PathBuf::from(s)
    }
}

// TODO: do we really need a new-type or could this be a type alias?
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TemplateContext {
    pub data: BTreeMap<String, Value>,
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
    env: Environment<'a>,
}

impl<'a> Templates<'a> {
    /// Constructs a new instance.
    pub fn new() -> CliResult<Self> {
        return Ok(Self {
            env: Environment::new(),
        });
    }

    pub fn new_with_templates(templates: HashMap<&'a str, String>) -> CliResult<Self> {
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
        }

        return Ok(Self { env });
    }

    /// Renders the template.
    pub fn render(&self, template: &str, context: TemplateContext) -> CliResult<String> {
        let tmpl = self.env.get_template(template).unwrap();
        let context = &context.data;
        Ok(tmpl.render(context)?)
    }

    pub fn one_off(template: &str, context: TemplateContext, escape: bool) -> CliResult<String> {
        let mut env = Environment::new();
        if escape {
            env.set_auto_escape_callback(|_| AutoEscape::Html);
        }

        Ok(env.render_str(template, context.data)?)
    }
}

// TODO: tests
#[cfg(test)]
mod tests {

    // TODO: invalid template should return valid error
}
