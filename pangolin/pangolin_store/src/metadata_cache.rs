use moka::future::Cache;
use std::time::Duration;
use anyhow::Result;

/// LRU cache for Iceberg metadata files
pub struct MetadataCache {
    cache: Cache<String, Vec<u8>>,
}

impl MetadataCache {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(max_capacity)
                .time_to_live(Duration::from_secs(ttl_seconds))
                .build(),
        }
    }

    /// Get cached metadata or fetch using provided function
    pub async fn get_or_fetch<F, Fut>(
        &self,
        location: &str,
        fetch_fn: F,
    ) -> Result<Vec<u8>>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<Vec<u8>>>,
    {
        // Check cache first
        if let Some(cached) = self.cache.get(location).await {
            tracing::debug!("Metadata cache HIT for {}", location);
            return Ok(cached);
        }

        tracing::debug!("Metadata cache MISS for {}", location);
        
        // Fetch from source
        let data = fetch_fn().await?;
        
        // Store in cache
        self.cache.insert(location.to_string(), data.clone()).await;
        
        Ok(data)
    }

    /// Invalidate a specific entry
    pub async fn invalidate(&self, location: &str) {
        self.cache.invalidate(location).await;
    }

    /// Clear all cached entries
    pub async fn clear(&self) {
        self.cache.invalidate_all();
    }

    /// Get cache statistics
    pub fn entry_count(&self) -> u64 {
        self.cache.entry_count()
    }
}

impl Clone for MetadataCache {
    fn clone(&self) -> Self {
        Self {
            cache: self.cache.clone(),
        }
    }
}

impl Default for MetadataCache {
    fn default() -> Self {
        // Default: 10,000 entries, 5 minute TTL
        Self::new(10_000, 300)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let cache = MetadataCache::new(100, 60);
        
        let location = "s3://bucket/metadata.json";
        let data = b"test data".to_vec();
        
        // First fetch - should be a miss
        let result = cache.get_or_fetch(location, || async {
            Ok(data.clone())
        }).await.unwrap();
        
        assert_eq!(result, data);
        assert_eq!(cache.entry_count(), 1);
        
        // Second fetch - should be a hit
        let result2 = cache.get_or_fetch(location, || async {
            panic!("Should not be called - cache hit expected");
        }).await.unwrap();
        
        assert_eq!(result2, data);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = MetadataCache::new(100, 60);
        
        let location = "s3://bucket/metadata.json";
        let data = b"test data".to_vec();
        
        cache.get_or_fetch(location, || async {
            Ok(data.clone())
        }).await.unwrap();
        
        assert_eq!(cache.entry_count(), 1);
        
        cache.invalidate(location).await;
        
        // Entry count might not immediately reflect invalidation
        // but next fetch should miss
        let mut fetch_called = false;
        cache.get_or_fetch(location, || async {
            fetch_called = true;
            Ok(data.clone())
        }).await.unwrap();
        
        assert!(fetch_called, "Fetch should be called after invalidation");
    }
}
