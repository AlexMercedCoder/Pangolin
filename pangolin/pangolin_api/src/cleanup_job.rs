use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use pangolin_store::CatalogStore;

/// Background job that periodically cleans up expired revoked tokens
/// 
/// This job runs every hour and removes tokens from the blacklist that have expired.
/// This prevents the blacklist from growing indefinitely.
pub async fn start_token_cleanup_job(store: Arc<dyn CatalogStore + Send + Sync>) {
    let mut cleanup_interval = interval(Duration::from_secs(3600)); // Run every hour
    
    tracing::info!("Token cleanup job started (runs every hour)");
    
    loop {
        cleanup_interval.tick().await;
        
        match store.cleanup_expired_tokens().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Token cleanup job: removed {} expired tokens", count);
                } else {
                    tracing::debug!("Token cleanup job: no expired tokens to remove");
                }
            }
            Err(e) => {
                tracing::error!("Token cleanup job failed: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_store::MemoryStore;
    use chrono::{Utc, Duration as ChronoDuration};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_cleanup_job_removes_expired_tokens() {
        let store = Arc::new(MemoryStore::new()) as Arc<dyn CatalogStore + Send + Sync>;
        
        // Add some expired tokens
        let expired_token1 = Uuid::new_v4();
        let expired_token2 = Uuid::new_v4();
        let past = Utc::now() - ChronoDuration::hours(1);
        
        store.revoke_token(expired_token1, past, Some("Test 1".to_string())).await.unwrap();
        store.revoke_token(expired_token2, past, Some("Test 2".to_string())).await.unwrap();
        
        // Add a valid token
        let valid_token = Uuid::new_v4();
        let future = Utc::now() + ChronoDuration::hours(24);
        store.revoke_token(valid_token, future, Some("Test 3".to_string())).await.unwrap();
        
        // Verify all are revoked
        assert!(store.is_token_revoked(expired_token1).await.unwrap());
        assert!(store.is_token_revoked(expired_token2).await.unwrap());
        assert!(store.is_token_revoked(valid_token).await.unwrap());
        
        // Run cleanup
        let count = store.cleanup_expired_tokens().await.unwrap();
        assert_eq!(count, 2, "Should have cleaned up 2 expired tokens");
        
        // Verify expired tokens are removed
        assert!(!store.is_token_revoked(expired_token1).await.unwrap());
        assert!(!store.is_token_revoked(expired_token2).await.unwrap());
        
        // Verify valid token still exists
        assert!(store.is_token_revoked(valid_token).await.unwrap());
    }
}
