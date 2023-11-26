// mkdocs.yml
// site --> default
// change be changed via site_dir

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};

#[derive(Deserialize)]
pub struct MKDocsConfig {
    site_dir: Option<String>,
}

impl FrameworkConfiguration for MKDocsConfig {
    type Config = Self;

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.site_dir.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::mkdocs::MKDocsConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_hugo() {
        let config =
            MKDocsConfig::get_config("tests/fixtures/framework_configs/mkdocs/mkdocs.yml").unwrap();

        assert_eq!(config.output_dir, Some(String::from("build")))
    }
}
