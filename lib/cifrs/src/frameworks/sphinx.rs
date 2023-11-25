// conf.py -- <sourcedir>/conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use std::env;
use std::path::PathBuf;

use serde_derive::Deserialize;

// ConfigurationFileDeserialization
use crate::framework::{FrameworkConfiguration, FrameworkConfigurationFormat, FrameworkSupport};
use crate::CifrsResult;


pub fn get_output_dir(_format: &FrameworkConfigurationFormat) -> CifrsResult<Option<String>> {
    Ok(env::var("BUILDDIR").ok())
}

#[cfg(test)]
mod tests {
    use crate::framework::{FrameworkConfigurationFormat, FrameworkSupport};

    #[test]
    fn test_sphinx() {
        let config = FrameworkConfigurationFormat::from_path(
            "tests/fixtures/framework_configs/sphinx/conf.py",
        )
        .unwrap();

        let output = super::get_output_dir(&config).unwrap();
        assert_eq!(output, None)
    }

    #[test]
    fn should_use_env_var_when_present() {
        temp_env::with_var("BUILDDIR", Some("build"), || {
            let config = FrameworkConfigurationFormat::from_path(
                "tests/fixtures/framework_configs/sphinx/conf.py",
            )
            .unwrap();

            let output = super::get_output_dir(&config).unwrap();
            assert_eq!(output, Some(String::from("build")))
        });
    }
}
