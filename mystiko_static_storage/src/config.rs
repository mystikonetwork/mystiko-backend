use serde::{Deserialize, Serialize};
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct FileStorageConfig {
    #[builder(default)]
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct S3StorageConfig {
    #[builder(default = default_s3_bucket())]
    #[serde(default = "default_s3_bucket")]
    pub bucket: String,
    #[builder(default = default_s3_region())]
    #[serde(default = "default_s3_region")]
    pub region: String,
    #[builder(default = default_s3_base())]
    #[serde(default = "default_s3_base")]
    pub base: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageType {
    S3,
    File,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TypedBuilder)]
#[builder(field_defaults(setter(into)))]
pub struct StorageCacheConfig {
    #[builder(default)]
    #[serde(default)]
    pub enabled: bool,
    #[builder(default)]
    #[serde(default)]
    pub path: Option<String>,
}

impl Default for FileStorageConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Default for S3StorageConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl Default for StorageType {
    fn default() -> Self {
        Self::S3
    }
}

impl Default for StorageCacheConfig {
    fn default() -> Self {
        Self::builder().build()
    }
}

fn default_s3_bucket() -> String {
    String::from("static.mystiko.network")
}

fn default_s3_region() -> String {
    String::from("us-east-1")
}

fn default_s3_base() -> String {
    String::from("/")
}
