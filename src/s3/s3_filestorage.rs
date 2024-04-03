use crate::core::{BuildError, Builder, FileStorage, StorageConfig};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::error::SdkError;
use aws_sdk_s3::operation::create_bucket::CreateBucketError;
use aws_sdk_s3::primitives::ByteStreamError;
use aws_sdk_s3::types::{CreateBucketConfiguration, Delete, ObjectIdentifier};
use aws_sdk_s3::{config::Region, Client};
use aws_smithy_types::byte_stream::ByteStream;
use std::env::var;

#[derive(Default)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub endpoint: String,
}

impl StorageConfig for S3Config {
    fn load(self, _conf_path: &str) -> Self {
        // TODO: support config
        // from env
        let bucket = var("BUCKET").unwrap_or("file-storage-bucket".to_string());
        let region = var("ENDPOINT").unwrap_or("http://localhost:4566".to_string());
        let endpoint = var("REGION").unwrap_or("us-west-2".to_string());
        S3Config {
            bucket,
            region,
            endpoint,
        }
    }
}

pub struct S3FileStorageBuilder {
    config: S3Config,
}

impl Builder<S3Config> for S3FileStorageBuilder {
    fn with_config(config: S3Config) -> Self {
        S3FileStorageBuilder { config }
    }

    fn build(self) -> Result<Box<dyn FileStorage>, BuildError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let region_clone = self.config.region.clone();

        return match rt.block_on(async {
            let region_provider = RegionProviderChain::first_try(Region::new(region_clone));
            let region = match region_provider.region().await {
                Some(region) => region,
                None => return Err(BuildError::new("region not found")),
            };

            let shared_config = aws_config::from_env()
                .region(region_provider)
                .endpoint_url(&self.config.endpoint)
                .load()
                .await;
            let mut config_builder = aws_sdk_s3::config::Builder::from(&shared_config);
            let s3_config_builder = config_builder.set_force_path_style(Some(true));
            let client = Client::from_conf(s3_config_builder.clone().build());

            Ok(Box::new(S3FileStorage {
                bucket: self.config.bucket,
                region,
                client,
            }))
        }) {
            Ok(storage) => Ok(storage),
            Err(e) => Err(BuildError::new(&e.to_string())),
        };
    }
}

pub struct S3FileStorage {
    client: Client,
    region: Region,
    bucket: String,
}

impl S3FileStorage {
    fn path_to_key(&self, path: &str) -> String {
        path.to_string()
    }

    fn key_to_path(&self, key: &str) -> String {
        key.to_string()
    }

    fn dir_to_prefix(&self, dir_path: &str) -> String {
        dir_path.to_string()
    }
}

impl FileStorage for S3FileStorage {
    // get object from s3
    fn read_file(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(async {
            let key = self.path_to_key(path);
            let object = match self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
            {
                Ok(object) => object,
                Err(e) => return Err(e),
            };

            // while let Some(bytes) = object.body.try_next().await {
            //     let bytes_len = bytes.len();
            // }
            let bytes = object
                .body
                .collect()
                .await
                .map_err(|e| <ByteStreamError as Into<std::io::Error>>::into(e))
                .unwrap()
                .into_bytes();

            Ok(bytes)
        }) {
            Ok(bytes) => Ok(bytes.to_vec()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // put object in s3
    fn write(&mut self, path: &str, buf: &[u8]) -> Result<(), std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(async {
            // read file from the path
            // let body = ByteStream::from_path(Path::new(path)).await;
            let body = ByteStream::from(buf.to_vec());

            let key = self.path_to_key(path);

            self.client
                .put_object()
                .bucket(&self.bucket)
                .key(key)
                .body(body)
                .send()
                .await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // put object in s3
    fn create_file(&self, _path: &str) -> Result<(), std::io::Error> {
        Ok(())
    }

    // delete object in s3
    fn remove_file(&self, path: &str) -> Result<(), std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(async {
            let key = self.path_to_key(path);
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(key)
                .send()
                .await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // copy object in s3
    fn copy(&self, source_path: &str, target_path: &str) -> Result<(), std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut source_bucket_and_object: String = "".to_owned();
        source_bucket_and_object.push_str(&self.bucket);
        source_bucket_and_object.push('/');
        source_bucket_and_object.push_str(&self.path_to_key(source_path));

        match rt.block_on(async {
            self.client
                .copy_object()
                .copy_source(source_bucket_and_object)
                .bucket(&self.bucket)
                .key(&self.path_to_key(target_path))
                .send()
                .await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // list objects in s3
    //
    fn read_dir(&self, dir_path: &str) -> Result<Vec<String>, std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        match rt.block_on(async {
            let prefix = self.dir_to_prefix(dir_path);
            self.client
                .list_objects_v2()
                .prefix(prefix)
                .bucket(&self.bucket)
                .send()
                .await
        }) {
            Ok(resp) => {
                let mut file_list = vec![];
                for object in resp.contents() {
                    // TODO: key to file path
                    file_list.push(object.key().unwrap_or_default().to_string());
                }
                Ok(file_list)
            }
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // create a directory in s3
    // create bucket? maybe not
    fn create_dir_all(&self, _path: &str) -> Result<(), std::io::Error> {
        let rt = tokio::runtime::Runtime::new().unwrap();

        match rt.block_on(async {
            let create_bucket_configuration = CreateBucketConfiguration::builder()
                .set_location_constraint(Some(
                    self.client
                        .config()
                        .region()
                        .unwrap()
                        .as_ref()
                        .to_string()
                        .parse()
                        .unwrap(),
                ))
                .build();

            match self
                .client
                .create_bucket()
                .bucket(&self.bucket)
                .create_bucket_configuration(create_bucket_configuration)
                .send()
                .await
            {
                Ok(_) => Ok(()),
                Err(SdkError::ServiceError(service_error)) => match service_error.err() {
                    CreateBucketError::BucketAlreadyExists(_)
                    | CreateBucketError::BucketAlreadyOwnedByYou(_) => Ok(()),
                    _ => Err(SdkError::ServiceError(service_error)),
                },
                Err(other_err) => Err(other_err),
            }
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }

    // remove a directory in s3
    fn remove_dir_all(&self, path: &str) -> Result<(), std::io::Error> {
        let files = match self.read_dir(path) {
            Ok(files) => files,
            Err(e) => return Err(e),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();

        let mut delete_objects: Vec<ObjectIdentifier> = vec![];

        // TODO: files to objects
        for obj in files {
            let obj_id = ObjectIdentifier::builder()
                .set_key(Some(obj))
                .build()
                .expect("building ObjectIdentifier");
            delete_objects.push(obj_id);
        }

        let delete = Delete::builder()
            .set_objects(Some(delete_objects))
            .build()
            .expect("building Delete");

        match rt.block_on(async {
            self.client
                .delete_objects()
                .bucket(&self.bucket)
                .delete(delete)
                .send()
                .await
        }) {
            Ok(_) => Ok(()),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )),
        }
    }
}
