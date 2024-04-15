use mystiko_static_storage::{FileStorageConfig, S3StorageConfig, StorageCacheConfig, StorageType};

#[test]
fn test_default_storage_config() {
    let config = StorageType::default();
    assert_eq!(config, StorageType::S3);
}

#[test]
fn test_default_s3_storage_config() {
    let config = S3StorageConfig::default();
    assert_eq!(config.bucket, "static.mystiko.network".to_string());
    assert_eq!(config.region, "us-east-1".to_string());
    assert_eq!(config.base, "/".to_string());
}

#[test]
fn test_default_file_storage_config() {
    let config = FileStorageConfig::default();
    assert!(config.path.is_none());
}

#[test]
fn test_default_storage_cache_config() {
    let config = StorageCacheConfig::default();
    assert!(!config.enabled);
    assert!(config.path.is_none());
}
