use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::str::FromStr;

pub fn as_hashmap() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (key, val) in env::vars_os() {
        if let (Ok(k), Ok(v)) = (key.into_string(), val.into_string()) {
            map.insert(k, v);
        }
    }

    map
}

pub fn as_boolean<K: AsRef<OsStr>>(key: K) -> bool {
    env::var(key)
        .ok()
        .is_some_and(|a| a.to_lowercase() == "true")
}

pub fn as_boolean_truthy<K: AsRef<OsStr>>(key: K) -> bool {
    env::var(key)
        .ok()
        .is_some_and(|a| a.to_lowercase() == "true" || a == "1")
}

pub fn parse<K: AsRef<OsStr>, T>(key: K) -> Result<T, String>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let value = env::var(key).map_err(|e| format!("{}", e))?;
    value
        .parse::<T>()
        .map_err(|e| format!("Parse error for '{}': {}", value, e))
}

pub fn parse_with_default<K: AsRef<OsStr>, R: std::str::FromStr>(key: K, default: R) -> R {
    match env::var(key) {
        Ok(v) => v.parse::<R>().unwrap_or(default),
        Err(_) => default,
    }
}

#[cfg(test)]
mod tests {
    use crate::env::{as_boolean, as_boolean_truthy, as_hashmap, parse, parse_with_default};

    #[test]
    fn to_hashmap() {
        temp_env::with_vars([("FIRST", Some("Hi")), ("SECOND", Some("1"))], || {
            let map = as_hashmap();
            assert_eq!(Some(&String::from("Hi")), map.get("FIRST"));
            assert_eq!(Some(&String::from("1")), map.get("SECOND"));
        });
    }

    #[test]
    fn test_as_boolean() {
        temp_env::with_vars(
            [
                ("FIRST", Some("Hi")),
                ("SECOND", Some("true")),
                ("THIRD", Some("True")),
            ],
            || {
                assert!(!as_boolean("FIRST"));
                assert!(as_boolean("SECOND"));
                assert!(as_boolean("THIRD"));
            },
        );
    }

    #[test]
    fn test_as_boolean_truthy() {
        temp_env::with_vars(
            [
                ("FIRST", Some("Hi")),
                ("SECOND", Some("true")),
                ("THIRD", Some("True")),
                ("FOURTH", Some("1")),
            ],
            || {
                assert!(!as_boolean_truthy("FIRST"));
                assert!(as_boolean_truthy("SECOND"));
                assert!(as_boolean_truthy("THIRD"));
                assert!(as_boolean_truthy("FOURTH"));
            },
        );
    }

    #[test]
    fn invalid_parse_should_return_error() {
        temp_env::with_vars([("INT_VAR", Some("1a"))], || {
            let result: Result<u32, String> = parse("INT_VAR");
            assert!(result.is_err());
            assert!(result.err().unwrap().contains("1a"))
        });
    }

    #[test]
    fn should_parse_numbers_successfully() {
        temp_env::with_vars([("INT_VAR", Some("1")), ("FLOAT_VAR", Some("2.5"))], || {
            assert_eq!(1, parse("INT_VAR").unwrap());
            assert_eq!(2.5, parse("FLOAT_VAR").unwrap());
        });
    }

    #[test]
    fn test_parse_with_default() {
        temp_env::with_vars([("PRESENT", Some("1a"))], || {
            assert_eq!(0, parse_with_default("PRESENT", 0));
            assert_eq!(10, parse_with_default("MISSING", 10));
        });
    }
}
