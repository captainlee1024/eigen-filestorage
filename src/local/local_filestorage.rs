// 在 src/local/mod.rs 文件中

use crate::core::{BuildError, Builder, FileStorage, StorageConfig};
use std::env::var;
use std::fs;
use std::io::Read;
use std::io::Write;

#[derive(Default)]
pub struct LocalConfig {
    root_path: String,
}

impl StorageConfig for LocalConfig {
    fn load(self, _conf_path: &str) -> Self {
        let root_path = var("ROOT_PATH").unwrap_or("".to_string());

        LocalConfig { root_path }
    }
}

pub struct LocalFileStorageBuilder {
    config: LocalConfig,
}

impl Builder<LocalConfig> for LocalFileStorageBuilder {
    fn with_config(config: LocalConfig) -> Self {
        LocalFileStorageBuilder { config }
    }

    fn build(self) -> Result<Box<dyn FileStorage>, BuildError> {
        Ok(Box::new(LocalFileStorage {
            root_path: self.config.root_path,
        }))
    }
}

pub struct LocalFileStorage {
    root_path: String,
}

impl FileStorage for LocalFileStorage {
    fn read_file(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let mut file = fs::File::open(format!("{}/{}", self.root_path, path))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn write(&mut self, path: &str, buf: &[u8]) -> Result<(), std::io::Error> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .open(format!("{}/{}", self.root_path, path))?;
        file.write_all(buf)
    }

    fn create_file(&self, path: &str) -> Result<(), std::io::Error> {
        let _ = fs::File::create(format!("{}/{}", self.root_path, path))?;
        Ok(())
    }

    fn remove_file(&self, path: &str) -> Result<(), std::io::Error> {
        let full_path = format!("{}/{}", self.root_path, path);
        fs::remove_file(full_path)
    }

    fn copy(&self, source_path: &str, target_path: &str) -> Result<(), std::io::Error> {
        fs::copy(
            format!("{}/{}", self.root_path, source_path),
            format!("{}/{}", self.root_path, target_path),
        )?;
        Ok(())
    }

    fn read_dir(&self, path: &str) -> Result<Vec<String>, std::io::Error> {
        let file_list = fs::read_dir(format!("{}/{}", self.root_path, path))?
            .map(|res| res.map(|e| e.path().display().to_string()))
            .collect::<Result<Vec<_>, std::io::Error>>()?;
        Ok(file_list)
    }

    fn create_dir_all(&self, path: &str) -> Result<(), std::io::Error> {
        fs::create_dir_all(format!("{}/{}", self.root_path, path))?;
        Ok(())
    }

    fn remove_dir_all(&self, path: &str) -> Result<(), std::io::Error> {
        let full_path = format!("{}/{}", self.root_path, path);
        fs::remove_dir_all(full_path)
    }
}
