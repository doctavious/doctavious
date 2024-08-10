use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use minijinja::{AutoEscape, Environment};
use serde::Serialize;
use serde_json::{to_value, Value};
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum TemplatingError {
    #[error("json serialize/deserialize error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Creating a Context from a Value/Serialize requires it being a JSON object")]
    TemplateContextError(),

    /// Error that may occur while template operations such as parse and render.
    #[error("Template error: `{0}`")]
    TemplateError(#[from] minijinja::Error),

    /// Error that may occur while parsing the template.
    #[error("Template parse error:\n{0}")]
    TemplateParseError(String),
}

pub type TemplatingResult<T> = Result<T, TemplatingError>;

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
    pub fn from_value(obj: Value) -> TemplatingResult<Self> {
        match obj {
            Value::Object(m) => {
                let mut data = BTreeMap::new();
                for (key, value) in m {
                    data.insert(key, value);
                }
                Ok(TemplateContext { data })
            }
            _ => Err(TemplatingError::TemplateContextError()),
        }
    }

    /// Converts the `val` parameter to `Value` and insert it into the context.
    ///
    /// Panics if the serialization fails.
    pub fn insert<T: Serialize + ?Sized, S: Into<String>>(&mut self, key: S, val: &T) {
        // TODO: remove unwrap
        self.data.insert(key.into(), to_value(val).unwrap());
    }

    /// Takes something that impl Serialize and create a context with it.
    /// Meant to be used if you have a hashmap or a struct and don't want to insert values
    /// one by one in the context.
    pub fn from_serialize(value: impl Serialize) -> TemplatingResult<Self> {
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

impl<S: Into<String>, V: Serialize, const N: usize> From<[(S, V); N]> for TemplateContext {
    fn from(arr: [(S, V); N]) -> Self {
        Self::from_iter(arr)
    }
}

impl<S: Into<String>, V: Serialize> FromIterator<(S, V)> for TemplateContext {
    fn from_iter<T: IntoIterator<Item = (S, V)>>(iter: T) -> Self {
        let mut data = BTreeMap::new();
        for (k, v) in iter {
            // TODO: remove unwrap
            data.insert(k.into(), to_value(v).unwrap());
        }
        Self { data }
    }
}

#[derive(Debug)]
pub struct Templates<'a> {
    env: Environment<'a>,
}

impl<'a> Templates<'a> {
    /// Constructs a new instance.
    pub fn new() -> TemplatingResult<Self> {
        Ok(Self {
            env: Environment::new(),
        })
    }

    pub fn new_with_templates(templates: HashMap<&'a str, String>) -> TemplatingResult<Self> {
        let mut env = Environment::new();
        for (k, v) in templates {
            if let Err(e) = env.add_template_owned(k, v) {
                return if let Some(error_source) = e.source() {
                    Err(TemplatingError::TemplateParseError(
                        error_source.to_string(),
                    ))
                } else {
                    Err(TemplatingError::TemplateError(e))
                };
            }
        }

        Ok(Self { env })
    }

    /// Renders the template.
    pub fn render(&self, template: &str, context: TemplateContext) -> TemplatingResult<String> {
        let tmpl = self.env.get_template(template)?;
        let context = &context.data;

        Ok(tmpl.render(context)?)
    }

    pub fn one_off(
        template: &str,
        context: TemplateContext,
        escape: bool,
    ) -> TemplatingResult<String> {
        let mut env = Environment::new();
        if escape {
            env.set_auto_escape_callback(|_| AutoEscape::Html);
        }

        Ok(env.render_str(template, context.data)?)
    }
}
