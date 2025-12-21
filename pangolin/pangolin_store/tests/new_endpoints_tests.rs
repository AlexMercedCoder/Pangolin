use pangolin_store::{CatalogStore, SqliteStore, PostgresStore, MongoStore};
use pangolin_core::token::TokenInfo;
use pangolin_core::model::{SystemSettings, SyncStats};
use uuid::Uuid;
use chrono::Utc;
use anyhow::Result;

// Helper to create test token
fn create_test_token(user_id: Uuid) -> TokenInfo {
    TokenInfo {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        user_id,
        username: "test_user".to_string(),
        token: Some("test_token_string".to_string()),
        expires_at: Utc::now() + chrono::Duration::hours(24),
        created_at: Utc::now(),
        is_valid: true,
    }
}

#[cfg(test)]
mod sqlite_new_endpoints_tests {
    use super::*;

    async fn setup_sqlite() -> Result<SqliteStore> {
        SqliteStore::new(":memory:").await
    }

    #[tokio::test]
    async fn test_token_management() -> Result<()> {
        let store = setup_sqlite().await?;
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Test storing a token
        let token = create_test_token(user_id);
        store.store_token(token.clone()).await?;

        // Test listing tokens
        let tokens = store.list_active_tokens(tenant_id, user_id).await?;
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].user_id, user_id);

        // Store another token for the same user
        let token2 = create_test_token(user_id);
        store.store_token(token2).await?;

        let tokens = store.list_active_tokens(tenant_id, user_id).await?;
        assert_eq!(tokens.len(), 2);

        Ok(())
    }

    #[tokio::test]
    async fn test_system_settings() -> Result<()> {
        let store = setup_sqlite().await?;
        let tenant_id = Uuid::new_v4();

        // Test getting default settings
        let settings = store.get_system_settings(tenant_id).await?;
        assert!(settings.allow_public_signup.is_none());

        // Test updating settings
        let new_settings = SystemSettings {
            allow_public_signup: Some(true),
            default_warehouse_bucket: Some("test-bucket".to_string()),
            default_retention_days: Some(30),
            smtp_host: Some("smtp.example.com".to_string()),
            smtp_port: Some(587),
            smtp_user: Some("user@example.com".to_string()),
            smtp_password: Some("password".to_string()),
        };

        let updated = store.update_system_settings(tenant_id, new_settings.clone()).await?;
        assert_eq!(updated.allow_public_signup, Some(true));
        assert_eq!(updated.default_warehouse_bucket, Some("test-bucket".to_string()));

        // Test retrieving updated settings
        let retrieved = store.get_system_settings(tenant_id).await?;
        assert_eq!(retrieved.allow_public_signup, Some(true));
        assert_eq!(retrieved.default_retention_days, Some(30));

        Ok(())
    }

    #[tokio::test]
    async fn test_federated_catalog_stats() -> Result<()> {
        let store = setup_sqlite().await?;
        let tenant_id = Uuid::new_v4();
        let catalog_name = "test_catalog";

        // Test getting stats for non-existent catalog
        let stats = store.get_federated_catalog_stats(tenant_id, catalog_name).await?;
        assert_eq!(stats.sync_status, "Never Synced");
        assert!(stats.last_synced_at.is_none());

        // Test syncing catalog
        store.sync_federated_catalog(tenant_id, catalog_name).await?;

        // Test retrieving stats after sync
        let stats = store.get_federated_catalog_stats(tenant_id, catalog_name).await?;
        assert_eq!(stats.sync_status, "Success");
        assert!(stats.last_synced_at.is_some());
        assert_eq!(stats.tables_synced, 0);
        assert_eq!(stats.namespaces_synced, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_merge_branch() -> Result<()> {
        let store = setup_sqlite().await?;
        let tenant_id = Uuid::new_v4();
        let catalog_name = "test_catalog";

        // Create tenant and catalog first
        let tenant = pangolin_core::model::Tenant {
            id: tenant_id,
            name: "Test Tenant".to_string(),
            properties: std::collections::HashMap::new(),
        };
        store.create_tenant(tenant).await?;

        let catalog = pangolin_core::model::Catalog {
            id: Uuid::new_v4(),
            name: catalog_name.to_string(),
            catalog_type: pangolin_core::model::CatalogType::Local,
            warehouse_name: None,
            storage_location: None,
            federated_config: None,
            properties: std::collections::HashMap::new(),
        };
        store.create_catalog(tenant_id, catalog).await?;

        // Create source and target branches
        let source_branch = pangolin_core::model::Branch {
            name: "source".to_string(),
            head_commit_id: Some(Uuid::new_v4()),
            branch_type: pangolin_core::model::BranchType::Experimental,
            assets: vec![],
        };
        store.create_branch(tenant_id, catalog_name, source_branch.clone()).await?;

        let target_branch = pangolin_core::model::Branch {
            name: "target".to_string(),
            head_commit_id: None,
            branch_type: pangolin_core::model::BranchType::Experimental,
            assets: vec![],
        };
        store.create_branch(tenant_id, catalog_name, target_branch).await?;

        // Test merge
        store.merge_branch(tenant_id, catalog_name, "target".to_string(), "source".to_string()).await?;

        // Verify target branch now has source's head
        let merged_target = store.get_branch(tenant_id, catalog_name, "target".to_string()).await?
            .expect("Target branch should exist");
        assert_eq!(merged_target.head_commit_id, source_branch.head_commit_id);

        Ok(())
    }
}

#[cfg(test)]
mod postgres_new_endpoints_tests {
    use super::*;

    async fn setup_postgres() -> Result<PostgresStore> {
        let db_url = std::env::var("TEST_POSTGRES_URL")
            .unwrap_or_else(|_| "postgres://pangolin:password@localhost:5433/pangolin_test".to_string());
        PostgresStore::new(&db_url).await
    }

    #[tokio::test]
    #[ignore] // Run with --ignored when postgres is available
    async fn test_token_management() -> Result<()> {
        let store = setup_postgres().await?;
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let token = create_test_token(user_id);
        store.store_token(token.clone()).await?;

        let tokens = store.list_active_tokens(tenant_id, user_id).await?;
        assert!(!tokens.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_system_settings() -> Result<()> {
        let store = setup_postgres().await?;
        let tenant_id = Uuid::new_v4();

        let settings = store.get_system_settings(tenant_id).await?;
        assert!(settings.allow_public_signup.is_none());

        let new_settings = SystemSettings {
            allow_public_signup: Some(true),
            default_warehouse_bucket: None,
            default_retention_days: None,
            smtp_host: None,
            smtp_port: None,
            smtp_user: None,
            smtp_password: None,
        };

        let updated = store.update_system_settings(tenant_id, new_settings).await?;
        assert_eq!(updated.allow_public_signup, Some(true));

        Ok(())
    }
}

#[cfg(test)]
mod mongo_new_endpoints_tests {
    use super::*;

    async fn setup_mongo() -> Result<MongoStore> {
        let db_url = std::env::var("TEST_MONGO_URL")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());
        MongoStore::new(&db_url, "pangolin_test").await
    }

    #[tokio::test]
    #[ignore] // Run with --ignored when mongo is available
    async fn test_token_management() -> Result<()> {
        let store = setup_mongo().await?;
        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let token = create_test_token(user_id);
        store.store_token(token.clone()).await?;

        let tokens = store.list_active_tokens(tenant_id, user_id).await?;
        assert!(!tokens.is_empty());

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_federated_catalog() -> Result<()> {
        let store = setup_mongo().await?;
        let tenant_id = Uuid::new_v4();
        let catalog_name = "test_catalog";

        store.sync_federated_catalog(tenant_id, catalog_name).await?;

        let stats = store.get_federated_catalog_stats(tenant_id, catalog_name).await?;
        assert_eq!(stats.sync_status, "Success");

        Ok(())
    }
}
