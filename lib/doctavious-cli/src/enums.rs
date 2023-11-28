use std::collections::HashMap;
use thiserror::Error;

pub fn parse_enum<A: Copy>(
    env: &'static HashMap<&'static str, A>,
    src: &str,
) -> Result<A, EnumError> {
    match env.get(src) {
        Some(p) => Ok(*p),
        None => {
            let supported: Vec<&&str> = env.keys().collect();
            Err(EnumError{
                message: format!(
                    "Unsupported value: \"{}\". Supported values: {:?}",
                    src, supported
                )
            })
        }
    }
}

#[derive(serde::Deserialize, Error, Debug)]
#[error("Enum error: {message}")]
pub struct EnumError {
    pub message: String,
}