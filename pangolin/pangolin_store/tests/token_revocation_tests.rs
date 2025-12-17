use pangolin_store::{CatalogStore, MemoryStore};
use chrono::{Duration, Utc};
use uuid::Uuid;

#[tokio::test]
async fn test_token_revocation() {
    let store = MemoryStore::new();
    let token_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::hours(24);
    
    // Token should not be revoked initially
    let is_revoked = store.is_token_revoked(token_id).await.unwrap();
    assert!(!is_revoked, "Token should not be revoked initially");
    
    // Revoke the token
    store.revoke_token(token_id, expires_at, Some("User logout".to_string()))
        .await
        .unwrap();
    
    // Token should now be revoked
    let is_revoked = store.is_token_revoked(token_id).await.unwrap();
    assert!(is_revoked, "Token should be revoked after revocation");
}

#[tokio::test]
async fn test_token_cleanup() {
    let store = MemoryStore::new();
    
    // Create some tokens - some expired, some not
    let expired_token1 = Uuid::new_v4();
    let expired_token2 = Uuid::new_v4();
    let valid_token = Uuid::new_v4();
    
    let past = Utc::now() - Duration::hours(1);
    let future = Utc::now() + Duration::hours(24);
    
    // Revoke tokens
    store.revoke_token(expired_token1, past, None).await.unwrap();
    store.revoke_token(expired_token2, past, None).await.unwrap();
    store.revoke_token(valid_token, future, None).await.unwrap();
    
    // All should be revoked
    assert!(store.is_token_revoked(expired_token1).await.unwrap());
    assert!(store.is_token_revoked(expired_token2).await.unwrap());
    assert!(store.is_token_revoked(valid_token).await.unwrap());
    
    // Cleanup expired tokens
    let cleaned = store.cleanup_expired_tokens().await.unwrap();
    assert_eq!(cleaned, 2, "Should have cleaned 2 expired tokens");
    
    // Expired tokens should be removed
    assert!(!store.is_token_revoked(expired_token1).await.unwrap());
    assert!(!store.is_token_revoked(expired_token2).await.unwrap());
    
    // Valid token should still be revoked
    assert!(store.is_token_revoked(valid_token).await.unwrap());
}

#[tokio::test]
async fn test_multiple_token_revocations() {
    let store = MemoryStore::new();
    let expires_at = Utc::now() + Duration::hours(24);
    
    // Revoke multiple tokens
    let tokens: Vec<Uuid> = (0..10).map(|_| Uuid::new_v4()).collect();
    
    for token_id in &tokens {
        store.revoke_token(*token_id, expires_at, Some(format!("Revoked {}", token_id)))
            .await
            .unwrap();
    }
    
    // All should be revoked
    for token_id in &tokens {
        assert!(store.is_token_revoked(*token_id).await.unwrap());
    }
    
    // Random token should not be revoked
    let random_token = Uuid::new_v4();
    assert!(!store.is_token_revoked(random_token).await.unwrap());
}
