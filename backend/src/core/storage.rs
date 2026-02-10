use crate::core::models::MediaItem;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage I/O failure: {0}")]
    Io(#[from] std::io::Error),

    #[error("Data serialization failure: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Data file corruption: {0}")]
    Corruption(String),

    #[error("Database error: {0}")]
    Database(String),
}

pub trait StorageProvider {
    fn load_all(&self) -> Result<Vec<MediaItem>, StorageError>;
    fn save_all(&self, items: &[MediaItem]) -> Result<(), StorageError>;
}