use dashmap::DashMap;
use lru::LruCache;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::error::Result as CasterResult;
use crate::{ContentSource, ContentType};

/// Cached content item with metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CachedContent {
    pub id: String,
    pub content_type: ContentType,
    pub source: ContentSource,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub size: usize,
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Content cache with LRU eviction and persistent storage
pub struct ContentCache {
    /// In-memory LRU cache
    memory_cache: Arc<Mutex<LruCache<String, CachedContent>>>,
    /// Mapping of cache keys to file paths for persistent storage
    disk_cache: Arc<DashMap<String, PathBuf>>,
    /// Cache directory
    cache_dir: PathBuf,
    /// Maximum cache size in bytes
    max_size: usize,
    /// Current cache size in bytes
    current_size: Arc<Mutex<usize>>,
}

impl ContentCache {
    /// Create a new content cache with default settings
    pub fn new() -> CasterResult<Self> {
        let cache_dir = directories::ProjectDirs::from("is", "8b", "q8-caster")
            .map(|dirs| dirs.cache_dir().to_path_buf())
            .unwrap_or_else(|| std::env::temp_dir().join("q8-caster-cache"));

        Self::with_config(cache_dir, 500) // 500MB default
    }

    /// Create a new content cache with custom configuration
    pub fn with_config(cache_dir: PathBuf, max_size_mb: usize) -> CasterResult<Self> {
        let max_size = max_size_mb * 1024 * 1024; // Convert to bytes
        let capacity = NonZeroUsize::new(100).unwrap(); // Store up to 100 items in memory

        std::fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            memory_cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            disk_cache: Arc::new(DashMap::new()),
            cache_dir,
            max_size,
            current_size: Arc::new(Mutex::new(0)),
        })
    }

    /// Store content in cache
    pub async fn store(
        &self,
        content_type: ContentType,
        source: ContentSource,
        data: Vec<u8>,
        mime_type: String,
    ) -> CasterResult<String> {
        let id = Uuid::new_v4().to_string();
        let size = data.len();

        // Check if we need to evict items
        self.ensure_capacity(size).await?;

        let cached_content = CachedContent {
            id: id.clone(),
            content_type,
            source,
            data: data.clone(),
            mime_type,
            size,
            cached_at: chrono::Utc::now(),
        };

        // Store in memory cache
        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.put(id.clone(), cached_content.clone());
        }

        // Store on disk for persistence
        let file_path = self.cache_dir.join(&id);
        let meta_path = self.cache_dir.join(format!("{}.meta", &id));
        
        // Write content data
        let mut file = fs::File::create(&file_path).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;
        
        // Write metadata as JSON (without the data field to save space)
        let metadata = serde_json::json!({
            "id": cached_content.id,
            "content_type": cached_content.content_type,
            "source": cached_content.source,
            "mime_type": cached_content.mime_type,
            "size": cached_content.size,
            "cached_at": cached_content.cached_at,
        });
        let mut meta_file = fs::File::create(&meta_path).await?;
        meta_file.write_all(serde_json::to_vec(&metadata)?.as_slice()).await?;
        meta_file.sync_all().await?;

        self.disk_cache.insert(id.clone(), file_path);

        // Update current size
        {
            let mut current_size = self.current_size.lock().unwrap();
            *current_size += size;
        }

        Ok(id)
    }

    /// Retrieve content from cache
    pub async fn get(&self, key: &str) -> CasterResult<Option<CachedContent>> {
        // Try memory cache first
        {
            let mut cache = self.memory_cache.lock().unwrap();
            if let Some(content) = cache.get(key) {
                return Ok(Some(content.clone()));
            }
        }

        // Try disk cache - reconstruct from file and metadata
        if let Some(path) = self.disk_cache.get(key) {
            let meta_path = self.cache_dir.join(format!("{}.meta", key));
            
            // Check if both data and metadata files exist
            if tokio::fs::try_exists(path.value()).await? && tokio::fs::try_exists(&meta_path).await? {
                let data = fs::read(path.value()).await?;
                let meta_data = fs::read(&meta_path).await?;
                let metadata: serde_json::Value = serde_json::from_slice(&meta_data)?;
                
                // Reconstruct CachedContent from metadata and data
                let cached_content = CachedContent {
                    id: metadata["id"].as_str().unwrap_or(key).to_string(),
                    content_type: serde_json::from_value(metadata["content_type"].clone())?,
                    source: serde_json::from_value(metadata["source"].clone())?,
                    data,
                    mime_type: metadata["mime_type"].as_str().unwrap_or("application/octet-stream").to_string(),
                    size: metadata["size"].as_u64().unwrap_or(0) as usize,
                    cached_at: serde_json::from_value(metadata["cached_at"].clone())?,
                };
                
                // Put back in memory cache for faster access next time
                {
                    let mut cache = self.memory_cache.lock().unwrap();
                    cache.put(key.to_string(), cached_content.clone());
                }
                
                return Ok(Some(cached_content));
            }
        }

        Ok(None)
    }

    /// Remove content from cache
    pub async fn remove(&self, key: &str) -> CasterResult<()> {
        // Remove from memory
        let size = {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.pop(key).map(|c| c.size)
        };

        // Remove from disk (both data and metadata files)
        if let Some((_, path)) = self.disk_cache.remove(key) {
            let _ = fs::remove_file(&path).await; // Ignore errors
            let meta_path = self.cache_dir.join(format!("{}.meta", key));
            let _ = fs::remove_file(&meta_path).await; // Ignore errors
        }

        // Update size
        if let Some(size) = size {
            let mut current_size = self.current_size.lock().unwrap();
            *current_size = current_size.saturating_sub(size);
        }

        Ok(())
    }

    /// Clear all cached content
    pub async fn clear(&self) -> CasterResult<()> {
        // Clear memory cache
        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.clear();
        }

        // Clear disk cache
        self.disk_cache.clear();

        // Remove all files
        let mut read_dir = fs::read_dir(&self.cache_dir).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let _ = fs::remove_file(entry.path()).await;
        }

        // Reset size
        {
            let mut current_size = self.current_size.lock().unwrap();
            *current_size = 0;
        }

        Ok(())
    }

    /// Ensure there's enough capacity for new content
    async fn ensure_capacity(&self, needed: usize) -> CasterResult<()> {
        loop {
            let current_size = *self.current_size.lock().unwrap();
            
            if current_size + needed <= self.max_size {
                break;
            }
            
            // Need to evict items
            let to_free = (current_size + needed) - self.max_size;
            
            // Check if we have items to evict
            let has_items = self.memory_cache.lock().unwrap().len() > 0;
            if !has_items {
                // No items to evict but still need space - this is an error condition
                return Err(crate::error::CasterError::Cache(
                    format!("Cannot fit item of size {} bytes in cache (max: {} bytes)", needed, self.max_size)
                ));
            }
            
            self.evict_lru(to_free).await?;
        }

        Ok(())
    }

    /// Evict least recently used items
    async fn evict_lru(&self, target_size: usize) -> CasterResult<()> {
        let mut freed = 0;
        let keys_to_remove: Vec<String> = {
            let mut cache = self.memory_cache.lock().unwrap();
            let mut keys = Vec::new();

            while freed < target_size {
                if let Some((key, content)) = cache.pop_lru() {
                    freed += content.size;
                    keys.push(key);
                } else {
                    break;
                }
            }

            keys
        };

        // Remove from disk cache (both data and metadata files)
        for key in keys_to_remove {
            if let Some((_, path)) = self.disk_cache.remove(&key) {
                let _ = fs::remove_file(&path).await;
                let meta_path = self.cache_dir.join(format!("{}.meta", &key));
                let _ = fs::remove_file(&meta_path).await;
            }
        }

        // Update size
        {
            let mut current_size = self.current_size.lock().unwrap();
            *current_size = current_size.saturating_sub(freed);
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let memory_count = self.memory_cache.lock().unwrap().len();
        let disk_count = self.disk_cache.len();
        let current_size = *self.current_size.lock().unwrap();

        CacheStats {
            memory_items: memory_count,
            disk_items: disk_count,
            total_size_bytes: current_size,
            max_size_bytes: self.max_size,
        }
    }
    
    /// Get cache statistics with recalculated disk usage for validation
    pub async fn validated_stats(&self) -> CacheStats {
        let memory_count = self.memory_cache.lock().unwrap().len();
        let disk_count = self.disk_cache.len();
        let disk_usage = self.calculate_actual_disk_usage().await;

        CacheStats {
            memory_items: memory_count,
            disk_items: disk_count,
            total_size_bytes: disk_usage,
            max_size_bytes: self.max_size,
        }
    }

    /// Recalculate actual disk usage by summing file sizes in cache_dir
    async fn calculate_actual_disk_usage(&self) -> usize {
        let mut total = 0usize;
        let mut entries = match fs::read_dir(&self.cache_dir).await {
            Ok(e) => e,
            Err(_) => return 0,
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(metadata) = entry.metadata().await {
                if metadata.is_file() {
                    total += metadata.len() as usize;
                }
            }
        }
        total
    }
}

/// Cache statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct CacheStats {
    pub memory_items: usize,
    pub disk_items: usize,
    pub total_size_bytes: usize,
    pub max_size_bytes: usize,
}
