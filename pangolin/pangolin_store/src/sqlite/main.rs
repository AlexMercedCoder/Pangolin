use async_trait::async_trait;
use crate::CatalogStore;
use pangolin_core::model::*;
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, UserRole as UserRoleAssignment};
use pangolin_core::audit::AuditLogEntry;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use anyhow::Result;
use chrono::{DateTime, Utc, TimeZone};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};
use pangolin_core::token::TokenInfo;
use pangolin_core::model::{SystemSettings, SyncStats};
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteStore {
    pub(crate) pool: SqlitePool,
    pub(crate) object_store_cache: crate::ObjectStoreCache,
    pub(crate) metadata_cache: crate::MetadataCache,
}

impl SqliteStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Extract file path from database URL and ensure it exists
        if database_url.starts_with("sqlite://") {
            let file_path = database_url.strip_prefix("sqlite://").unwrap();
            
            // Create parent directories if they don't exist
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            
            // Create the file if it doesn't exist
            if !std::path::Path::new(file_path).exists() {
                std::fs::File::create(file_path)?;
            }
        }
        
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        
        // Enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await?;
        
        Ok(Self { 
            pool,
            object_store_cache: crate::ObjectStoreCache::new(),
            metadata_cache: crate::MetadataCache::default(),
        })
    }
    
    pub async fn apply_schema(&self, schema_sql: &str) -> Result<()> {
        // Disable foreign keys during schema creation
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&self.pool)
            .await?;
        
        // Parse statements
        let mut statements = Vec::new();
        let mut current_statement = String::new();
        let mut paren_depth = 0;
        
        for line in schema_sql.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("--") {
                continue;
            }
            for ch in line.chars() {
                match ch {
                    '(' => paren_depth += 1,
                    ')' => paren_depth -= 1,
                    ';' if paren_depth == 0 => {
                        if !current_statement.trim().is_empty() {
                            statements.push(current_statement.trim().to_string());
                            current_statement.clear();
                        }
                        continue;
                    }
                    _ => {}
                }
                current_statement.push(ch);
            }
            current_statement.push('\n');
        }
        if !current_statement.trim().is_empty() {
            statements.push(current_statement.trim().to_string());
        }
        
        for statement in statements {
            sqlx::query(&statement).execute(&self.pool).await?;
        }
        
        // Re-enable foreign keys
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub fn row_to_user(&self, row: Option<sqlx::sqlite::SqliteRow>) -> Result<Option<User>> {
        if let Some(row) = row {
            let role_str: String = row.get("role");
            let role = match role_str.as_str() {
                "Root" => UserRole::Root,
                "TenantAdmin" => UserRole::TenantAdmin,
                _ => UserRole::TenantUser,
            };

            let oauth_provider_str: Option<String> = row.get("oauth_provider");
            let oauth_provider = match oauth_provider_str.as_deref() {
                Some("google") => Some(OAuthProvider::Google),
                Some("microsoft") => Some(OAuthProvider::Microsoft),
                Some("github") => Some(OAuthProvider::GitHub),
                Some("okta") => Some(OAuthProvider::Okta),
                _ => None,
            };

            Ok(Some(User {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                username: row.get("username"),
                email: row.get("email"),
                password_hash: row.get("password_hash"),
                oauth_provider,
                oauth_subject: row.get("oauth_subject"),
                tenant_id: row.get::<Option<String>, _>("tenant_id").map(|s| Uuid::parse_str(&s)).transpose()?,
                role,
                created_at: Utc.timestamp_millis_opt(row.get("created_at")).single().unwrap_or_default(),
                updated_at: Utc.timestamp_millis_opt(row.get("updated_at")).single().unwrap_or_default(),
                last_login: row.get::<Option<i64>, _>("last_login").map(|t| Utc.timestamp_millis_opt(t).single().unwrap_or_default()),
                active: row.get::<i32, _>("active") != 0,
            }))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl CatalogStore for SqliteStore {
    // Phase 1: Tenants, Warehouses, Catalogs
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> { self.create_tenant(tenant).await }
    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> { self.get_tenant(id).await }
    async fn list_tenants(&self) -> Result<Vec<Tenant>> { self.list_tenants().await }
    async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> { self.update_tenant(tenant_id, updates).await }
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> { self.delete_tenant(tenant_id).await }

    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> { self.create_warehouse(tenant_id, warehouse).await }
    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> { self.get_warehouse(tenant_id, name).await }
    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> { self.list_warehouses(tenant_id).await }
    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> { self.update_warehouse(tenant_id, name, updates).await }
    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> { self.delete_warehouse(tenant_id, name).await }

    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> { self.create_catalog(tenant_id, catalog).await }
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> { self.get_catalog(tenant_id, name).await }
    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> { self.list_catalogs(tenant_id).await }
    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog> { self.update_catalog(tenant_id, name, updates).await }
    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> { self.delete_catalog(tenant_id, name).await }

    // Phase 2: Namespaces, Assets, Branches, Tags, Commits
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> { self.create_namespace(tenant_id, catalog_name, namespace).await }
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>> { self.list_namespaces(tenant_id, catalog_name, parent).await }
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> { self.get_namespace(tenant_id, catalog_name, namespace).await }
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> { self.delete_namespace(tenant_id, catalog_name, namespace).await }
    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> { self.update_namespace_properties(tenant_id, catalog_name, namespace, properties).await }

    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> { self.create_asset(tenant_id, catalog_name, branch, namespace, asset).await }
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> { self.get_asset(tenant_id, catalog_name, branch, namespace, name).await }
    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> { self.get_asset_by_id(tenant_id, asset_id).await }
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> { self.list_assets(tenant_id, catalog_name, branch, namespace).await }
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> { self.delete_asset(tenant_id, catalog_name, branch, namespace, name).await }
    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> { self.rename_asset(tenant_id, catalog_name, branch, source_namespace, source_name, dest_namespace, dest_name).await }
    async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> { self.count_namespaces(tenant_id).await }
    async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> { self.count_assets(tenant_id).await }

    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> { self.create_branch(tenant_id, catalog_name, branch).await }
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> { self.get_branch(tenant_id, catalog_name, name).await }
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> { self.list_branches(tenant_id, catalog_name).await }
    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch: String, target_branch: String) -> Result<()> { self.merge_branch(tenant_id, catalog_name, source_branch, target_branch).await }

    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> { self.create_tag(tenant_id, catalog_name, tag).await }
    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> { self.get_tag(tenant_id, catalog_name, name).await }
    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> { self.list_tags(tenant_id, catalog_name).await }
    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> { self.delete_tag(tenant_id, catalog_name, name).await }

    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> { self.create_commit(tenant_id, commit).await }
    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>> { self.get_commit(tenant_id, commit_id).await }

    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> { self.get_metadata_location(tenant_id, catalog_name, branch, namespace, table).await }
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> { self.update_metadata_location(tenant_id, catalog_name, branch, namespace, table, expected_location, new_location).await }
    
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> { self.read_file(path).await }
    async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> { self.write_file(path, data).await }

    // Phase 3 & 4 (Access Control, Audit, Settings, etc)
    async fn create_user(&self, user: User) -> Result<()> { self.create_user(user).await }
    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> { self.get_user(user_id).await }
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> { self.get_user_by_username(username).await }
    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<User>> { self.list_users(tenant_id).await }
    async fn update_user(&self, user: User) -> Result<()> { self.update_user(user).await }
    async fn delete_user(&self, user_id: Uuid) -> Result<()> { self.delete_user(user_id).await }

    async fn create_role(&self, role: Role) -> Result<()> { self.create_role(role).await }
    async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> { self.get_role(role_id).await }
    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<Role>> { self.list_roles(tenant_id).await }
    async fn delete_role(&self, role_id: Uuid) -> Result<()> { self.delete_role(role_id).await }
    async fn update_role(&self, role: Role) -> Result<()> { self.update_role(role).await }
    async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> { self.assign_role(user_role).await }
    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> { self.revoke_role(user_id, role_id).await }
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> { self.get_user_roles(user_id).await }

    async fn create_permission(&self, permission: Permission) -> Result<()> { self.create_permission(permission).await }
    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> { self.revoke_permission(permission_id).await }
    async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> { self.list_user_permissions(user_id).await }
    async fn list_permissions(&self, tenant_id: Uuid) -> Result<Vec<Permission>> { self.list_permissions(tenant_id).await }

    async fn store_token(&self, token_info: TokenInfo) -> Result<()> { self.store_token(token_info).await }
    async fn validate_token(&self, token: &str) -> Result<Option<TokenInfo>> { self.validate_token(token).await }
    async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> { self.revoke_token(token_id, expires_at, reason).await }
    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> { self.is_token_revoked(token_id).await }
    async fn cleanup_expired_tokens(&self) -> Result<usize> { self.cleanup_expired_tokens().await }
    async fn list_active_tokens(&self, _tenant_id: Uuid, user_id: Uuid) -> Result<Vec<TokenInfo>> { self.list_active_tokens(user_id).await }

    async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> { self.log_audit_event(tenant_id, event).await }
    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<AuditLogEntry>> { self.list_audit_events(tenant_id, filter).await }
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> { self.count_audit_events(tenant_id, filter).await }
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> { self.get_audit_event(tenant_id, event_id).await }

    async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> { self.get_system_settings(tenant_id).await }
    async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> { self.update_system_settings(tenant_id, settings).await }

    async fn sync_federated_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<()> { self.sync_federated_catalog(tenant_id, catalog_name).await }
    async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> { self.get_federated_catalog_stats(tenant_id, catalog_name).await }

    async fn expire_snapshots(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, retention_ms: i64) -> Result<()> { self.expire_snapshots(tenant_id, catalog_name, branch, namespace, table, retention_ms).await }
    async fn remove_orphan_files(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, older_than_ms: i64) -> Result<()> { self.remove_orphan_files(tenant_id, catalog_name, branch, namespace, table, older_than_ms).await }

    async fn create_access_request(&self, request: AccessRequest) -> Result<()> { self.create_access_request(request).await }
    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> { self.get_access_request(id).await }
    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> { self.list_access_requests(tenant_id).await }
    async fn update_access_request(&self, request: AccessRequest) -> Result<()> { self.update_access_request(request).await }

    // Business Metadata
    async fn upsert_business_metadata(&self, metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> { self.upsert_business_metadata(metadata).await }
    async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> { self.get_business_metadata(asset_id).await }
    async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> { self.delete_business_metadata(asset_id).await }
    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>> { self.search_assets(tenant_id, query, tags).await }
    async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> { self.search_catalogs(tenant_id, query).await }
    async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> { self.search_namespaces(tenant_id, query).await }
    async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> { self.search_branches(tenant_id, query).await }

    // Merge Operations (Phase 5 partial?)
    // If not implemented in merge_operations.rs, we need default Err or logic.
    // Assuming merge_operations.rs handles them?
    // Let's delegate if they exist in SqliteStore.
    // merge_operations.rs was created in Step 2883? No, Step 2920 included it in mod.rs.
    // Step 2920: pub mod merge_operations;
    // Step 2883: fixed merge_operations.rs.
    // So merge operations are supported.
    async fn create_merge_operation(&self, operation: MergeOperation) -> Result<()> { self.create_merge_operation(operation).await }
    async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<MergeOperation>> { self.get_merge_operation(operation_id).await }
    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<MergeOperation>> { self.list_merge_operations(tenant_id, catalog_name).await }
    async fn update_merge_operation_status(&self, operation_id: Uuid, status: MergeStatus) -> Result<()> { self.update_merge_operation_status(operation_id, status).await }
    async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> { self.complete_merge_operation(operation_id, result_commit_id).await }
    async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> { self.abort_merge_operation(operation_id).await }
    
    async fn create_merge_conflict(&self, conflict: MergeConflict) -> Result<()> { self.create_merge_conflict(conflict).await }
    async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<MergeConflict>> { self.get_merge_conflict(conflict_id).await }
    async fn list_merge_conflicts(&self, operation_id: Uuid) -> Result<Vec<MergeConflict>> { self.list_merge_conflicts(operation_id).await }
    async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<()> { self.resolve_merge_conflict(conflict_id, resolution).await }
    // async fn add_conflict_to_operation matches logic in merge_operations?
    // If interface requires it.
    async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> { self.add_conflict_to_operation(operation_id, conflict_id).await }

    // Service Users
    async fn create_service_user(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> { self.create_service_user(service_user).await }
    async fn get_service_user(&self, id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> { self.get_service_user(id).await }
    async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<pangolin_core::user::ServiceUser>> { self.get_service_user_by_api_key_hash(api_key_hash).await }
    async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::user::ServiceUser>> { self.list_service_users(tenant_id).await }
    async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, active: Option<bool>) -> Result<()> { self.update_service_user(id, name, description, active).await }
    async fn delete_service_user(&self, id: Uuid) -> Result<()> { self.delete_service_user(id).await }
    async fn update_service_user_last_used(&self, id: Uuid, timestamp: DateTime<Utc>) -> Result<()> { self.update_service_user_last_used(id, timestamp).await }
}
