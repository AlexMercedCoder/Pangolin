use dashmap::DashMap;
use object_store::ObjectStore;
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Thread-safe cache for ObjectStore instances
pub struct ObjectStoreCache {
    cache: Arc<DashMap<String, Arc<dyn ObjectStore>>>,
}

impl ObjectStoreCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Generate cache key from warehouse configuration
    pub fn cache_key(
        endpoint: &str,
        bucket: &str,
        access_key: &str,
        region: &str,
    ) -> String {
        let mut hasher = DefaultHasher::new();
        endpoint.hash(&mut hasher);
        bucket.hash(&mut hasher);
        access_key.hash(&mut hasher);
        region.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get or insert ObjectStore instance
    pub fn get_or_insert<F>(&self, key: String, factory: F) -> Arc<dyn ObjectStore>
    where
        F: FnOnce() -> Arc<dyn ObjectStore>,
    {
        self.cache
            .entry(key)
            .or_insert_with(factory)
            .clone()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

impl Default for ObjectStoreCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ObjectStoreCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let key1 = ObjectStoreCache::cache_key(
            "http://localhost:9000",
            "warehouse",
            "minioadmin",
            "us-east-1",
        );
        let key2 = ObjectStoreCache::cache_key(
            "http://localhost:9000",
            "warehouse",
            "minioadmin",
            "us-east-1",
        );
        let key3 = ObjectStoreCache::cache_key(
            "http://localhost:9000",
            "different-bucket",
            "minioadmin",
            "us-east-1",
        );

        assert_eq!(key1, key2, "Same config should produce same key");
        assert_ne!(key1, key3, "Different config should produce different key");
    }
}
