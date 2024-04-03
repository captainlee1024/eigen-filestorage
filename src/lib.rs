pub mod core;
pub mod local;
pub mod s3;

pub use core::{Builder, FileStorage, StorageConfig};

pub enum StorageType {
    S3,
    Local,
}

pub fn build_storage(storage_type: StorageType, config_path: &str) -> Box<dyn core::FileStorage> {
    match storage_type {
        StorageType::S3 => {
            let config: s3::S3Config = s3::S3Config::default().load(config_path);
            s3::S3FileStorageBuilder::with_config(config)
                .build()
                .unwrap()
        }
        StorageType::Local => {
            let config: local::LocalConfig = local::LocalConfig::default().load(config_path);
            local::LocalFileStorageBuilder::with_config(config)
                .build()
                .unwrap()
        }
    }
}
