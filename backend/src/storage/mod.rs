// Storage backend trait and local implementation
// WAVELET Backend - Preset file storage abstraction

use async_trait::async_trait;
use uuid::Uuid;
use std::path::PathBuf;
use std::fmt;

/// Storage error types
#[derive(Debug)]
pub enum StorageError {
    /// File not found
    NotFound,
    /// Permission denied
    PermissionDenied,
    /// IO error during file operation
    IoError(String),
    /// Invalid path
    InvalidPath,
    /// Other error
    Other(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound => write!(f, "File not found"),
            StorageError::PermissionDenied => write!(f, "Permission denied"),
            StorageError::IoError(msg) => write!(f, "IO error: {}", msg),
            StorageError::InvalidPath => write!(f, "Invalid file path"),
            StorageError::Other(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IoError(e.to_string())
    }
}

/// Storage backend trait for preset files
/// Allows for different storage implementations (local, S3, etc.)
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Upload a preset file
    /// 
    /// # Arguments
    /// * `preset_id` - Unique preset identifier
    /// * `data` - File contents as bytes
    /// 
    /// # Returns
    /// Storage path or URL on success
    async fn upload_preset(&self, preset_id: Uuid, data: &[u8]) -> Result<String, StorageError>;
    
    /// Download a preset file
    /// 
    /// # Arguments
    /// * `preset_id` - Unique preset identifier
    /// 
    /// # Returns
    /// File contents as bytes
    async fn download_preset(&self, preset_id: Uuid) -> Result<Vec<u8>, StorageError>;
    
    /// Delete a preset file
    /// 
    /// # Arguments
    /// * `preset_id` - Unique preset identifier
    /// 
    /// # Returns
    /// Ok(()) on success
    async fn delete_preset(&self, preset_id: Uuid) -> Result<(), StorageError>;
    
    /// Get storage path for a preset (for reference)
    async fn get_preset_path(&self, preset_id: Uuid) -> Result<String, StorageError>;
}

/// Local filesystem storage implementation
/// Stores preset files on the local filesystem
#[derive(Clone, Debug)]
pub struct LocalStorage {
    /// Base directory for preset storage
    base_path: PathBuf,
}

impl LocalStorage {
    /// Create new local storage instance
    /// 
    /// # Arguments
    /// * `base_path` - Base directory for storing preset files
    pub fn new(base_path: PathBuf) -> Self {
        // Ensure base path exists
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)
                .expect("Failed to create preset storage directory");
        }
        
        Self { base_path }
    }
    
    /// Get the full path for a preset file
    fn get_file_path(&self, preset_id: Uuid) -> PathBuf {
        self.base_path.join(format!("{}.json", preset_id))
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn upload_preset(&self, preset_id: Uuid, data: &[u8]) -> Result<String, StorageError> {
        let path = self.get_file_path(preset_id);
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Write file atomically using temp file
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, data).await?;
        tokio::fs::rename(&temp_path, &path).await?;
        
        // Return the file path as string
        Ok(path.to_string_lossy().to_string())
    }
    
    async fn download_preset(&self, preset_id: Uuid) -> Result<Vec<u8>, StorageError> {
        let path = self.get_file_path(preset_id);
        
        if !path.exists() {
            return Err(StorageError::NotFound);
        }
        
        let data = tokio::fs::read(&path).await?;
        Ok(data)
    }
    
    async fn delete_preset(&self, preset_id: Uuid) -> Result<(), StorageError> {
        let path = self.get_file_path(preset_id);
        
        if path.exists() {
            tokio::fs::remove_file(&path).await?;
        }
        
        Ok(())
    }
    
    async fn get_preset_path(&self, preset_id: Uuid) -> Result<String, StorageError> {
        let path = self.get_file_path(preset_id);
        Ok(path.to_string_lossy().to_string())
    }
}

/// In-memory storage for testing purposes
/// Stores presets in memory (not for production use)
#[derive(Clone, Default, Debug)]
pub struct InMemoryStorage {
    /// Map of preset ID to file data
    data: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<Uuid, Vec<u8>>>>,
}

impl InMemoryStorage {
    /// Create new in-memory storage
    pub fn new() -> Self {
        Self {
            data: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    async fn upload_preset(&self, preset_id: Uuid, data: &[u8]) -> Result<String, StorageError> {
        let mut storage = self.data.lock().unwrap();
        storage.insert(preset_id, data.to_vec());
        Ok(format!("memory://preset/{}", preset_id))
    }
    
    async fn download_preset(&self, preset_id: Uuid) -> Result<Vec<u8>, StorageError> {
        let storage = self.data.lock().unwrap();
        storage.get(&preset_id)
            .cloned()
            .ok_or(StorageError::NotFound)
    }
    
    async fn delete_preset(&self, preset_id: Uuid) -> Result<(), StorageError> {
        let mut storage = self.data.lock().unwrap();
        storage.remove(&preset_id);
        Ok(())
    }
    
    async fn get_preset_path(&self, preset_id: Uuid) -> Result<String, StorageError> {
        Ok(format!("memory://preset/{}", preset_id))
    }
}

/// Storage factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create local storage with default path
    /// 
    /// # Arguments
    /// * `data_dir` - Base data directory
    pub fn create_local_storage(data_dir: &str) -> LocalStorage {
        let base_path = PathBuf::from(data_dir).join("presets");
        LocalStorage::new(base_path)
    }
    
    /// Create in-memory storage for testing
    pub fn create_memory_storage() -> InMemoryStorage {
        InMemoryStorage::new()
    }
}
