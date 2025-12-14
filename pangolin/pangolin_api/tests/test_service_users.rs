use pangolin_core::user::{ServiceUser, UserRole};
use pangolin_store::{CatalogStore, MemoryStore};
use uuid::Uuid;
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};

#[tokio::test]
async fn test_create_service_user() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let api_key = "test_api_key_12345";
    let api_key_hash = hash(api_key, DEFAULT_COST).unwrap();
    
    let service_user = ServiceUser::new(
        "test-service".to_string(),
        Some("Test service user".to_string()),
        tenant_id,
        api_key_hash.clone(),
        UserRole::TenantUser,
        creator_id,
        None, // No expiration
    );
    
    let service_user_id = service_user.id;
    
    store.create_service_user(service_user).await.unwrap();
    
    let retrieved = store.get_service_user(service_user_id).await.unwrap();
    assert!(retrieved.is_some());
    
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.name, "test-service");
    assert_eq!(retrieved.tenant_id, tenant_id);
    assert_eq!(retrieved.role, UserRole::TenantUser);
    assert!(retrieved.active);
    assert!(verify(api_key, &retrieved.api_key_hash).unwrap());
}

#[tokio::test]
async fn test_list_service_users() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    // Create multiple service users
    for i in 0..3 {
        let api_key_hash = hash(&format!("key_{}", i), DEFAULT_COST).unwrap();
        let service_user = ServiceUser::new(
            format!("service-{}", i),
            None,
            tenant_id,
            api_key_hash,
            UserRole::TenantUser,
            creator_id,
            None,
        );
        store.create_service_user(service_user).await.unwrap();
    }
    
    let service_users = store.list_service_users(tenant_id).await.unwrap();
    assert_eq!(service_users.len(), 3);
}

#[tokio::test]
async fn test_update_service_user() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let api_key_hash = hash("test_key", DEFAULT_COST).unwrap();
    let service_user = ServiceUser::new(
        "original-name".to_string(),
        Some("Original description".to_string()),
        tenant_id,
        api_key_hash,
        UserRole::TenantUser,
        creator_id,
        None,
    );
    
    let service_user_id = service_user.id;
    store.create_service_user(service_user).await.unwrap();
    
    // Update name and description
    store.update_service_user(
        service_user_id,
        Some("updated-name".to_string()),
        Some("Updated description".to_string()),
        None,
    ).await.unwrap();
    
    let updated = store.get_service_user(service_user_id).await.unwrap().unwrap();
    assert_eq!(updated.name, "updated-name");
    assert_eq!(updated.description, Some("Updated description".to_string()));
}

#[tokio::test]
async fn test_deactivate_service_user() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let api_key_hash = hash("test_key", DEFAULT_COST).unwrap();
    let service_user = ServiceUser::new(
        "test-service".to_string(),
        None,
        tenant_id,
        api_key_hash,
        UserRole::TenantUser,
        creator_id,
        None,
    );
    
    let service_user_id = service_user.id;
    store.create_service_user(service_user).await.unwrap();
    
    // Deactivate
    store.update_service_user(
        service_user_id,
        None,
        None,
        Some(false),
    ).await.unwrap();
    
    let deactivated = store.get_service_user(service_user_id).await.unwrap().unwrap();
    assert!(!deactivated.active);
    assert!(!deactivated.is_valid());
}

#[tokio::test]
async fn test_delete_service_user() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let api_key_hash = hash("test_key", DEFAULT_COST).unwrap();
    let service_user = ServiceUser::new(
        "test-service".to_string(),
        None,
        tenant_id,
        api_key_hash,
        UserRole::TenantUser,
        creator_id,
        None,
    );
    
    let service_user_id = service_user.id;
    store.create_service_user(service_user).await.unwrap();
    
    store.delete_service_user(service_user_id).await.unwrap();
    
    let deleted = store.get_service_user(service_user_id).await.unwrap();
    assert!(deleted.is_none());
}

#[tokio::test]
async fn test_service_user_expiration() {
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    let api_key_hash = hash("test_key", DEFAULT_COST).unwrap();
    
    // Create expired service user
    let expired_service_user = ServiceUser::new(
        "expired-service".to_string(),
        None,
        tenant_id,
        api_key_hash.clone(),
        UserRole::TenantUser,
        creator_id,
        Some(Utc::now() - Duration::days(1)), // Expired yesterday
    );
    
    assert!(expired_service_user.is_expired());
    assert!(!expired_service_user.is_valid());
    
    // Create non-expired service user
    let valid_service_user = ServiceUser::new(
        "valid-service".to_string(),
        None,
        tenant_id,
        api_key_hash,
        UserRole::TenantUser,
        creator_id,
        Some(Utc::now() + Duration::days(30)), // Expires in 30 days
    );
    
    assert!(!valid_service_user.is_expired());
    assert!(valid_service_user.is_valid());
}

#[tokio::test]
async fn test_update_last_used() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let api_key_hash = hash("test_key", DEFAULT_COST).unwrap();
    let service_user = ServiceUser::new(
        "test-service".to_string(),
        None,
        tenant_id,
        api_key_hash,
        UserRole::TenantUser,
        creator_id,
        None,
    );
    
    let service_user_id = service_user.id;
    store.create_service_user(service_user).await.unwrap();
    
    // Initially last_used should be None
    let initial = store.get_service_user(service_user_id).await.unwrap().unwrap();
    assert!(initial.last_used.is_none());
    
    // Update last_used
    let now = Utc::now();
    store.update_service_user_last_used(service_user_id, now).await.unwrap();
    
    let updated = store.get_service_user(service_user_id).await.unwrap().unwrap();
    assert!(updated.last_used.is_some());
    assert_eq!(updated.last_used.unwrap().timestamp(), now.timestamp());
}

#[tokio::test]
async fn test_api_key_rotation() {
    let store = MemoryStore::new();
    let tenant_id = Uuid::new_v4();
    let creator_id = Uuid::new_v4();
    
    let old_key = "old_api_key_12345";
    let old_hash = hash(old_key, DEFAULT_COST).unwrap();
    
    let mut service_user = ServiceUser::new(
        "test-service".to_string(),
        None,
        tenant_id,
        old_hash.clone(),
        UserRole::TenantUser,
        creator_id,
        None,
    );
    
    let service_user_id = service_user.id;
    store.create_service_user(service_user.clone()).await.unwrap();
    
    // Verify old key works
    assert!(verify(old_key, &old_hash).unwrap());
    
    // Rotate to new key
    let new_key = "new_api_key_67890";
    let new_hash = hash(new_key, DEFAULT_COST).unwrap();
    service_user.api_key_hash = new_hash.clone();
    
    // Update with new hash (simulating rotation)
    store.create_service_user(service_user).await.unwrap();
    
    let rotated = store.get_service_user(service_user_id).await.unwrap().unwrap();
    
    // Old key should not work
    assert!(!verify(old_key, &rotated.api_key_hash).unwrap());
    
    // New key should work
    assert!(verify(new_key, &rotated.api_key_hash).unwrap());
}
