use std::fmt;

pub trait Builder<C: StorageConfig> {
    fn with_config(config: C) -> Self;
    fn build(self) -> Result<Box<dyn FileStorage>, BuildError>;
}

pub trait FileStorage {
    // file
    fn read_file(&self, path: &str) -> Result<Vec<u8>, std::io::Error>;
    fn write(&mut self, path: &str, buf: &[u8]) -> Result<(), std::io::Error>;
    fn create_file(&self, path: &str) -> Result<(), std::io::Error>;
    fn remove_file(&self, path: &str) -> Result<(), std::io::Error>;
    fn copy(&self, source_path: &str, target_path: &str) -> Result<(), std::io::Error>;

    // dir
    fn read_dir(&self, path: &str) -> Result<Vec<String>, std::io::Error>; // list
    fn create_dir_all(&self, path: &str) -> Result<(), std::io::Error>;
    fn remove_dir_all(&self, path: &str) -> Result<(), std::io::Error>;
}

pub trait StorageConfig {
    fn load(self, conf_path: &str) -> Self;
}

#[derive(Debug)]
pub struct BuildError {
    details: String,
}

impl BuildError {
    pub(crate) fn new(msg: &str) -> BuildError {
        BuildError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for BuildError {
    fn description(&self) -> &str {
        &self.details
    }
}
