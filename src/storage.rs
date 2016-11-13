use std::fs::{File, metadata};
use std::io::Result;
use std::path::{Path,PathBuf};
use super::error::{TransporterResult, TransporterError};

#[derive(Clone)]
pub struct FileStorage {
    base_directory: String
}

impl FileStorage {
    pub fn open(base_directory: &String) -> Result<FileStorage> {
        let path = Path::new(base_directory);
        metadata(&path)?;
        Ok(FileStorage{ base_directory: base_directory.clone() })
    }

    pub fn create(self: &Self, file_name: &str) -> TransporterResult<File> {
        let mut path_buffer = PathBuf::new();
        path_buffer.push(&self.base_directory);
        path_buffer.push(file_name);
        let file = File::create(&path_buffer).map_err(|io| TransporterError::StorageIoError(io))?;
        Ok(file)
    }
}
