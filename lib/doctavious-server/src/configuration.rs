use figment::Figment;
use figment::providers::{Env, Format, Json, Toml};
use opendal::Operator;
use opendal::services::Gcs;
// use serde;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
}

#[derive(Deserialize)]
pub struct AppConfiguration {
    // port
}

#[derive(Deserialize)]
pub struct DatabaseConfiguration {
    pub user: String,
    pub pwd: String,
    pub url: String,
    pub driver: String, // Postgres / MySQL / sqlite / etc
    pub name: String,
}

// Figment might not support enum with derive See https://github.com/SergioBenitez/Figment/issues/1
#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageType {
    Filesystem,
    Gcs,
    S3,
}

// TODO: need to support filesystem as well
#[derive(Deserialize)]
pub struct ExternalStorageConfiguration {
    pub url: String,
    pub storage_type: StorageType,
}

impl ExternalStorageConfiguration {
    pub fn get_storage(self) {
        match &self.storage_type {
            StorageType::Filesystem => todo!(),
            StorageType::Gcs => todo!(),
            StorageType::S3 => todo!(),
        }
    }
}
#[derive(Deserialize)]
pub struct FileStorageConfig {
    pub path: String,

    #[serde(
        default,
        skip_serializing_if = "crate::serde::skip_serializing_if_default"
    )]
    pub compression: Compression,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Compression {
    /// [Gzip][gzip] compression.
    ///
    /// [gzip]: https://www.gzip.org/
    Gzip,

    /// [Zstandard][zstd] compression.
    ///
    /// [zstd]: https://facebook.github.io/zstd/
    Zstd,

    /// No compression.
    #[default]
    None,
}

// TODO: honeycomb config: https://github.com/vectordotdev/vector/blob/fa8a55385dd391aa2429c3f2e9821198c364c6a0/src/sinks/honeycomb.rs

// https://github.com/vectordotdev/vector/blob/fa8a55385dd391aa2429c3f2e9821198c364c6a0/src/sinks/opendal_common.rs
// https://github.com/vectordotdev/vector/blob/fa8a55385dd391aa2429c3f2e9821198c364c6a0/src/sinks/mod.rs

pub fn get_configuration() { //-> Result<Configuration> {
    // let config: Config = Figment::new()
    //     .merge(Toml::file("Cargo.toml"))
    //     .merge(Env::prefixed("CARGO_"))
    //     .merge(Env::raw().only(&["RUSTC", "RUSTDOC"]))
    //     .join(Json::file("Cargo.json"))
    //     .extract()?;
}
