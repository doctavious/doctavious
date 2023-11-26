// conf.py -- <sourcedir>/conf.py
// sphinx package
// i dont see a way to configure this outside env var
// we could just default it ourselves
// BUILDDIR env var

use std::env;

use serde_derive::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};
use crate::CifrsResult;

#[derive(Deserialize)]
pub struct SphinxConfig {}

impl FrameworkConfiguration for SphinxConfig {
    type Config = Self;

    fn from_python(_content: &str) -> CifrsResult<Self> {
        Ok(Self {})
    }

    fn convert_to_common_config(_config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: env::var("BUILDDIR").ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::sphinx::SphinxConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_sphinx() {
        let config =
            SphinxConfig::get_config("tests/fixtures/framework_configs/sphinx/conf.py").unwrap();

        assert_eq!(config.output_dir, None)
    }

    #[test]
    fn should_use_env_var_when_present() {
        temp_env::with_var("BUILDDIR", Some("build"), || {
            let config =
                SphinxConfig::get_config("tests/fixtures/framework_configs/sphinx/conf.py")
                    .unwrap();

            assert_eq!(config.output_dir, Some(String::from("build")))
        });
    }
}
