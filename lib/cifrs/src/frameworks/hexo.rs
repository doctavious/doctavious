// output defaults to public
// _config.yml
// public_dir to override
// hexo-cli
// hexo generate
// hexo --config custom.yml

use serde::Deserialize;

use crate::frameworks::{FrameworkConfigFile, FrameworkConfiguration};

#[derive(Deserialize)]
pub struct HexoConfig {
    public_dir: Option<String>,
}

impl FrameworkConfiguration for HexoConfig {
    type Config = Self;

    fn convert_to_common_config(config: &Self::Config) -> FrameworkConfigFile {
        FrameworkConfigFile {
            output_dir: config.public_dir.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frameworks::hexo::HexoConfig;
    use crate::frameworks::FrameworkConfiguration;

    #[test]
    fn test_hexo() {
        let config =
            HexoConfig::get_config("tests/fixtures/framework_configs/hexo/_config.yml").unwrap();

        assert_eq!(config.output_dir, Some(String::from("build")))
    }
}
