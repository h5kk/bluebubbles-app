//! Cache service for managing cached data (attachments, thumbnails, avatars).
//!
//! Tracks cache size, provides cleanup of old entries using LRU eviction,
//! and manages the on-disk cache directory.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tracing::{info, warn, debug};

use bb_core::error::{BbError, BbResult};

use crate::event_bus::EventBus;
use crate::service::{Service, ServiceState};

/// Metadata about a cached file entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Absolute path to the cached file.
    pub path: PathBuf,
    /// Size of the file in bytes.
    pub size: u64,
    /// Last access/modification time (epoch seconds).
    pub last_accessed: u64,
}

/// Summary of the current cache state.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total number of cached files.
    pub file_count: usize,
    /// Total size of all cached files in bytes.
    pub total_bytes: u64,
    /// Path to the cache directory.
    pub cache_dir: PathBuf,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mb = self.total_bytes as f64 / (1024.0 * 1024.0);
        write!(
            f,
            "{} files, {:.1} MB in {}",
            self.file_count,
            mb,
            self.cache_dir.display()
        )
    }
}

/// Service for managing the on-disk cache directory.
///
/// Provides cache size tracking, LRU-based eviction when the cache exceeds
/// a configured maximum, and manual cleanup operations.
pub struct CacheService {
    state: ServiceState,
    event_bus: EventBus,
    /// Root cache directory.
    cache_dir: PathBuf,
    /// Maximum cache size in bytes (default 500 MB).
    max_cache_bytes: u64,
}

/// Default maximum cache size: 500 MB.
const DEFAULT_MAX_CACHE_BYTES: u64 = 500 * 1024 * 1024;

impl CacheService {
    /// Create a new CacheService with the given cache directory.
    pub fn new(event_bus: EventBus, cache_dir: PathBuf) -> Self {
        Self {
            state: ServiceState::Created,
            event_bus,
            cache_dir,
            max_cache_bytes: DEFAULT_MAX_CACHE_BYTES,
        }
    }

    /// Set the maximum cache size in bytes.
    pub fn set_max_cache_bytes(&mut self, max: u64) {
        self.max_cache_bytes = max;
    }

    /// Get the cache directory path.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Ensure the cache directory exists.
    fn ensure_cache_dir(&self) -> BbResult<()> {
        if !self.cache_dir.exists() {
            fs::create_dir_all(&self.cache_dir)
                .map_err(|e| BbError::Config(format!("failed to create cache dir: {e}")))?;
        }
        Ok(())
    }

    /// Calculate the current cache statistics by scanning the cache directory.
    pub fn stats(&self) -> BbResult<CacheStats> {
        let entries = self.scan_entries()?;
        let total_bytes: u64 = entries.iter().map(|e| e.size).sum();
        Ok(CacheStats {
            file_count: entries.len(),
            total_bytes,
            cache_dir: self.cache_dir.clone(),
        })
    }

    /// Scan all files in the cache directory and return sorted entries.
    fn scan_entries(&self) -> BbResult<Vec<CacheEntry>> {
        if !self.cache_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();

        Self::scan_directory_recursive(&self.cache_dir, &mut entries)?;

        Ok(entries)
    }

    /// Recursively scan a directory for cached files.
    fn scan_directory_recursive(dir: &Path, entries: &mut Vec<CacheEntry>) -> BbResult<()> {
        let read_dir = fs::read_dir(dir)
            .map_err(|e| BbError::Config(format!("failed to read cache dir: {e}")))?;

        for entry_result in read_dir {
            let entry = entry_result
                .map_err(|e| BbError::Config(format!("failed to read dir entry: {e}")))?;

            let path = entry.path();

            if path.is_dir() {
                Self::scan_directory_recursive(&path, entries)?;
            } else if path.is_file() {
                let metadata = fs::metadata(&path)
                    .map_err(|e| BbError::Config(format!("failed to stat file: {e}")))?;

                let last_accessed = metadata
                    .modified()
                    .or_else(|_| metadata.created())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                entries.push(CacheEntry {
                    path,
                    size: metadata.len(),
                    last_accessed,
                });
            }
        }

        Ok(())
    }

    /// Check if a file exists in the cache.
    pub fn is_cached(&self, relative_path: &str) -> bool {
        self.cache_dir.join(relative_path).exists()
    }

    /// Get the absolute path for a cache entry.
    pub fn cache_path(&self, relative_path: &str) -> PathBuf {
        self.cache_dir.join(relative_path)
    }

    /// Run LRU eviction to bring the cache under the configured maximum size.
    ///
    /// Removes the least recently accessed files first until the total size
    /// is below `max_cache_bytes`. Returns the number of files removed and
    /// bytes freed.
    pub fn evict_lru(&self) -> BbResult<(usize, u64)> {
        let mut entries = self.scan_entries()?;
        let total_bytes: u64 = entries.iter().map(|e| e.size).sum();

        if total_bytes <= self.max_cache_bytes {
            debug!("cache within limits ({} bytes <= {} max)", total_bytes, self.max_cache_bytes);
            return Ok((0, 0));
        }

        // Sort by last_accessed ascending (oldest first for eviction)
        entries.sort_by_key(|e| e.last_accessed);

        let mut freed = 0u64;
        let mut removed = 0usize;
        let target = total_bytes - self.max_cache_bytes;

        for entry in &entries {
            if freed >= target {
                break;
            }

            match fs::remove_file(&entry.path) {
                Ok(_) => {
                    freed += entry.size;
                    removed += 1;
                    debug!("evicted: {} ({} bytes)", entry.path.display(), entry.size);
                }
                Err(e) => {
                    warn!("failed to evict {}: {e}", entry.path.display());
                }
            }
        }

        info!("LRU eviction: removed {removed} files, freed {freed} bytes");
        Ok((removed, freed))
    }

    /// Clear the entire cache directory.
    ///
    /// Removes all files and subdirectories. The cache directory itself is
    /// recreated empty.
    pub fn clear(&self) -> BbResult<(usize, u64)> {
        let entries = self.scan_entries()?;
        let total_bytes: u64 = entries.iter().map(|e| e.size).sum();
        let count = entries.len();

        if self.cache_dir.exists() {
            fs::remove_dir_all(&self.cache_dir)
                .map_err(|e| BbError::Config(format!("failed to clear cache: {e}")))?;
        }

        self.ensure_cache_dir()?;

        info!("cleared cache: {count} files, {total_bytes} bytes");
        Ok((count, total_bytes))
    }

    /// Remove a specific file from the cache by its relative path.
    pub fn remove(&self, relative_path: &str) -> BbResult<bool> {
        let full_path = self.cache_dir.join(relative_path);
        if full_path.exists() {
            fs::remove_file(&full_path)
                .map_err(|e| BbError::Config(format!("failed to remove cache entry: {e}")))?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Service for CacheService {
    fn name(&self) -> &str {
        "cache"
    }

    fn state(&self) -> ServiceState {
        self.state
    }

    fn init(&mut self) -> BbResult<()> {
        self.state = ServiceState::Initializing;
        self.ensure_cache_dir()?;

        match self.stats() {
            Ok(stats) => info!("cache service initialized: {stats}"),
            Err(e) => warn!("could not read initial cache stats: {e}"),
        }

        self.state = ServiceState::Running;
        Ok(())
    }

    fn shutdown(&mut self) -> BbResult<()> {
        self.state = ServiceState::Stopped;
        info!("cache service stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn setup_cache() -> (tempfile::TempDir, CacheService) {
        let dir = tempfile::TempDir::new().unwrap();
        let cache_dir = dir.path().join("cache");
        let bus = EventBus::new(16);
        let svc = CacheService::new(bus, cache_dir);
        (dir, svc)
    }

    #[test]
    fn test_cache_service_name() {
        let (_dir, svc) = setup_cache();
        assert_eq!(svc.name(), "cache");
    }

    #[test]
    fn test_cache_init_creates_dir() {
        let (_dir, mut svc) = setup_cache();
        svc.init().unwrap();
        assert!(svc.cache_dir().exists());
    }

    #[test]
    fn test_cache_stats_empty() {
        let (_dir, mut svc) = setup_cache();
        svc.init().unwrap();
        let stats = svc.stats().unwrap();
        assert_eq!(stats.file_count, 0);
        assert_eq!(stats.total_bytes, 0);
    }

    #[test]
    fn test_cache_is_cached() {
        let (_dir, mut svc) = setup_cache();
        svc.init().unwrap();

        assert!(!svc.is_cached("test.txt"));

        let path = svc.cache_path("test.txt");
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(b"hello").unwrap();

        assert!(svc.is_cached("test.txt"));
    }

    #[test]
    fn test_cache_remove() {
        let (_dir, mut svc) = setup_cache();
        svc.init().unwrap();

        let path = svc.cache_path("remove-me.txt");
        fs::write(&path, b"data").unwrap();
        assert!(svc.is_cached("remove-me.txt"));

        assert!(svc.remove("remove-me.txt").unwrap());
        assert!(!svc.is_cached("remove-me.txt"));

        // Removing again returns false
        assert!(!svc.remove("remove-me.txt").unwrap());
    }

    #[test]
    fn test_lru_eviction() {
        let (_dir, mut svc) = setup_cache();
        svc.set_max_cache_bytes(100); // 100 byte limit
        svc.init().unwrap();

        // Create files that exceed the limit
        for i in 0..5 {
            let path = svc.cache_path(&format!("file-{i}.bin"));
            fs::write(&path, vec![0u8; 50]).unwrap(); // 50 bytes each = 250 total
        }

        let stats_before = svc.stats().unwrap();
        assert_eq!(stats_before.file_count, 5);
        assert_eq!(stats_before.total_bytes, 250);

        let (removed, freed) = svc.evict_lru().unwrap();
        assert!(removed >= 3); // Need to remove at least 150 bytes = 3 files
        assert!(freed >= 150);

        let stats_after = svc.stats().unwrap();
        assert!(stats_after.total_bytes <= 100);
    }

    #[test]
    fn test_cache_clear() {
        let (_dir, mut svc) = setup_cache();
        svc.init().unwrap();

        fs::write(svc.cache_path("a.txt"), b"aaa").unwrap();
        fs::write(svc.cache_path("b.txt"), b"bbb").unwrap();

        let (count, bytes) = svc.clear().unwrap();
        assert_eq!(count, 2);
        assert_eq!(bytes, 6);

        let stats = svc.stats().unwrap();
        assert_eq!(stats.file_count, 0);
        // Dir should be recreated
        assert!(svc.cache_dir().exists());
    }

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            file_count: 42,
            total_bytes: 10_485_760,
            cache_dir: PathBuf::from("/tmp/cache"),
        };
        let display = format!("{stats}");
        assert!(display.contains("42 files"));
        assert!(display.contains("10.0 MB"));
    }
}
