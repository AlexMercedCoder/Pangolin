use std::sync::Arc;
use async_trait::async_trait;
use pangolin_store::{CatalogStore, PaginationParams};
use pangolin_store::signer::{Signer, Credentials};
use pangolin_core::model::*;
use pangolin_core::user::{ServiceUser, User};
use pangolin_core::token::TokenInfo;
use pangolin_core::permission::{Permission, UserRole};
use uuid::Uuid;
use moka::future::Cache;
use anyhow::Result;
use std::time::Duration;
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct CachedCatalogStore {
    inner: Arc<dyn CatalogStore + Send + Sync>,
    // Cache key: (TenantId, WarehouseName) -> Warehouse
    warehouse_cache: Cache<(Uuid, String), Warehouse>,
}

impl CachedCatalogStore {
    pub fn new(inner: Arc<dyn CatalogStore + Send + Sync>) -> Self {
        Self {
            inner,
            warehouse_cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_secs(60)) // 1 minute TTL for credentials
                .build(),
        }
    }
}

#[async_trait]
impl Signer for CachedCatalogStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
        self.inner.get_table_credentials(location).await
    }

    async fn presign_get(&self, location: &str) -> Result<String> {
        self.inner.presign_get(location).await
    }
}

#[async_trait]
impl CatalogStore for CachedCatalogStore {
    // --- Warehouse Operations (Cached) ---

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let key = (tenant_id, name.clone());
        
        if let Some(warehouse) = self.warehouse_cache.get(&key).await {
            tracing::info!("Cache HIT for warehouse: {}/{}", tenant_id, name);
            return Ok(Some(warehouse));
        }

        tracing::info!("Cache MISS for warehouse: {}/{}", tenant_id, name);
        let warehouse = self.inner.get_warehouse(tenant_id, name.clone()).await?;
        
        if let Some(w) = &warehouse {
            self.warehouse_cache.insert(key, w.clone()).await;
        }
        
        Ok(warehouse)
    }

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
        // Perform update
        let updated_warehouse = self.inner.update_warehouse(tenant_id, name.clone(), updates).await?;
        
        // Update cache with new value immediately
        tracing::info!("Cache UPDATE for warehouse: {}/{}", tenant_id, name);
        self.warehouse_cache.insert((tenant_id, name), updated_warehouse.clone()).await;
        
        Ok(updated_warehouse)
    }

    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        // Invalidate cache on delete
        tracing::info!("Cache INVALIDATE (Delete) for warehouse: {}/{}", tenant_id, name);
        self.warehouse_cache.invalidate(&(tenant_id, name.clone())).await;
        self.inner.delete_warehouse(tenant_id, name).await
    }

    // --- Passthrough Operations (Uncached) ---

    // Tenant
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> { self.inner.create_tenant(tenant).await }
    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> { self.inner.get_tenant(id).await }
    async fn list_tenants(&self, pagination: Option<PaginationParams>) -> Result<Vec<Tenant>> { self.inner.list_tenants(pagination).await }
    async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> { self.inner.update_tenant(tenant_id, updates).await }
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> { self.inner.delete_tenant(tenant_id).await }

    // Warehouse (Create/List)
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> { self.inner.create_warehouse(tenant_id, warehouse).await }
    async fn list_warehouses(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Warehouse>> { self.inner.list_warehouses(tenant_id, pagination).await }

    // Catalog
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> { self.inner.create_catalog(tenant_id, catalog).await }
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> { self.inner.get_catalog(tenant_id, name).await }
    async fn list_catalogs(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Catalog>> { self.inner.list_catalogs(tenant_id, pagination).await }
    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog> { self.inner.update_catalog(tenant_id, name, updates).await }
    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> { self.inner.delete_catalog(tenant_id, name).await }

    // Namespace
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> { self.inner.create_namespace(tenant_id, catalog_name, namespace).await }
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> { self.inner.get_namespace(tenant_id, catalog_name, namespace).await }
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>, pagination: Option<PaginationParams>) -> Result<Vec<Namespace>> { self.inner.list_namespaces(tenant_id, catalog_name, parent, pagination).await }
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> { self.inner.delete_namespace(tenant_id, catalog_name, namespace).await }
    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> { self.inner.update_namespace_properties(tenant_id, catalog_name, namespace, properties).await }

    // Asset
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> { self.inner.create_asset(tenant_id, catalog_name, branch, namespace, asset).await }
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> { self.inner.get_asset(tenant_id, catalog_name, branch, namespace, name).await }
    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> { self.inner.get_asset_by_id(tenant_id, asset_id).await }
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<PaginationParams>) -> Result<Vec<Asset>> { self.inner.list_assets(tenant_id, catalog_name, branch, namespace, pagination).await }
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> { self.inner.delete_asset(tenant_id, catalog_name, branch, namespace, name).await }
    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> { self.inner.rename_asset(tenant_id, catalog_name, branch, source_namespace, source_name, dest_namespace, dest_name).await }
    async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> { self.inner.count_namespaces(tenant_id).await }
    async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> { self.inner.count_assets(tenant_id).await }

    // Branch
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> { self.inner.create_branch(tenant_id, catalog_name, branch).await }
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> { self.inner.get_branch(tenant_id, catalog_name, name).await }
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<Branch>> { self.inner.list_branches(tenant_id, catalog_name, pagination).await }
    async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> { self.inner.delete_branch(tenant_id, catalog_name, name).await }
    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, target_branch: String, source_branch: String) -> Result<()> { self.inner.merge_branch(tenant_id, catalog_name, target_branch, source_branch).await }
    async fn copy_assets_bulk(&self, tenant_id: Uuid, catalog_name: &str, src_branch: &str, dest_branch: &str, namespace: Option<String>) -> Result<usize> { self.inner.copy_assets_bulk(tenant_id, catalog_name, src_branch, dest_branch, namespace).await }
    async fn get_commit_ancestry(&self, tenant_id: Uuid, head_commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> { self.inner.get_commit_ancestry(tenant_id, head_commit_id, limit).await }
    
    async fn find_conflicting_assets(&self, tenant_id: Uuid, catalog_name: &str, source_branch: &str, target_branch: &str) -> Result<Vec<(Asset, Asset)>> { self.inner.find_conflicting_assets(tenant_id, catalog_name, source_branch, target_branch).await }
    async fn list_all_assets_in_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: &str) -> Result<Vec<(Asset, Vec<String>)>> { self.inner.list_all_assets_in_branch(tenant_id, catalog_name, branch).await }

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> { self.inner.create_tag(tenant_id, catalog_name, tag).await }
    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> { self.inner.get_tag(tenant_id, catalog_name, name).await }
    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<Tag>> { self.inner.list_tags(tenant_id, catalog_name, pagination).await }
    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> { self.inner.delete_tag(tenant_id, catalog_name, name).await }

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> { self.inner.create_commit(tenant_id, commit).await }
    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>> { self.inner.get_commit(tenant_id, commit_id).await }

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> { self.inner.get_metadata_location(tenant_id, catalog_name, branch, namespace, table).await }
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> { self.inner.update_metadata_location(tenant_id, catalog_name, branch, namespace, table, expected_location, new_location).await }
    
    // Generic File IO
    async fn read_file(&self, location: &str) -> Result<Vec<u8>> { self.inner.read_file(location).await }
    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()> { self.inner.write_file(location, content).await }

    // Maintenance Operations
    async fn expire_snapshots(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, retention_ms: i64) -> Result<()> { self.inner.expire_snapshots(tenant_id, catalog_name, branch, namespace, table, retention_ms).await }
    async fn remove_orphan_files(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, older_than_ms: i64) -> Result<()> { self.inner.remove_orphan_files(tenant_id, catalog_name, branch, namespace, table, older_than_ms).await }

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: pangolin_core::audit::AuditLogEntry) -> Result<()> { self.inner.log_audit_event(tenant_id, event).await }
    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> { self.inner.list_audit_events(tenant_id, filter).await }
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<pangolin_core::audit::AuditLogEntry>> { self.inner.get_audit_event(tenant_id, event_id).await }
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> { self.inner.count_audit_events(tenant_id, filter).await }

    // User Operations
    async fn create_user(&self, user: User) -> Result<()> { self.inner.create_user(user).await }
    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> { self.inner.get_user(user_id).await }
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> { self.inner.get_user_by_username(username).await }
    async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<PaginationParams>) -> Result<Vec<User>> { self.inner.list_users(tenant_id, pagination).await }
    async fn update_user(&self, user: User) -> Result<()> { self.inner.update_user(user).await }
    async fn delete_user(&self, user_id: Uuid) -> Result<()> { self.inner.delete_user(user_id).await }

    // Role Operations
    async fn create_role(&self, role: pangolin_core::permission::Role) -> Result<()> { self.inner.create_role(role).await }
    async fn get_role(&self, role_id: Uuid) -> Result<Option<pangolin_core::permission::Role>> { self.inner.get_role(role_id).await }
    async fn list_roles(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::permission::Role>> { self.inner.list_roles(tenant_id, pagination).await }
    async fn assign_role(&self, user_role: pangolin_core::permission::UserRole) -> Result<()> { self.inner.assign_role(user_role).await }
    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> { self.inner.revoke_role(user_id, role_id).await }
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<pangolin_core::permission::UserRole>> { self.inner.get_user_roles(user_id).await }
    async fn delete_role(&self, role_id: Uuid) -> Result<()> { self.inner.delete_role(role_id).await }
    async fn update_role(&self, role: pangolin_core::permission::Role) -> Result<()> { self.inner.update_role(role).await }

    // Permission Operations
    async fn create_permission(&self, permission: Permission) -> Result<()> { self.inner.create_permission(permission).await }
    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> { self.inner.revoke_permission(permission_id).await }
    async fn list_user_permissions(&self, user_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>> { self.inner.list_user_permissions(user_id, pagination).await }
    async fn list_permissions(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>> { self.inner.list_permissions(tenant_id, pagination).await }

    // Business Metadata Operations
    async fn upsert_business_metadata(&self, metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> { self.inner.upsert_business_metadata(metadata).await }
    async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> { self.inner.get_business_metadata(asset_id).await }
    async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> { self.inner.delete_business_metadata(asset_id).await }
    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>> { self.inner.search_assets(tenant_id, query, tags).await }
    async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> { self.inner.search_catalogs(tenant_id, query).await }
    async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> { self.inner.search_namespaces(tenant_id, query).await }
    async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> { self.inner.search_branches(tenant_id, query).await }
    
    // Access Requests
    async fn create_access_request(&self, request: pangolin_core::business_metadata::AccessRequest) -> Result<()> { self.inner.create_access_request(request).await }
    async fn get_access_request(&self, id: Uuid) -> Result<Option<pangolin_core::business_metadata::AccessRequest>> { self.inner.get_access_request(id).await }
    async fn list_access_requests(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::business_metadata::AccessRequest>> { self.inner.list_access_requests(tenant_id, pagination).await }
    async fn update_access_request(&self, request: pangolin_core::business_metadata::AccessRequest) -> Result<()> { self.inner.update_access_request(request).await }

    // Service Users
    async fn create_service_user(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> { self.inner.create_service_user(service_user).await }
    async fn get_service_user(&self, id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> { self.inner.get_service_user(id).await }
    async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<pangolin_core::user::ServiceUser>> { self.inner.get_service_user_by_api_key_hash(api_key_hash).await }
    async fn list_service_users(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::user::ServiceUser>> { self.inner.list_service_users(tenant_id, pagination).await }
    async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, active: Option<bool>) -> Result<()> { self.inner.update_service_user(id, name, description, active).await }
    async fn delete_service_user(&self, id: Uuid) -> Result<()> { self.inner.delete_service_user(id).await }
    async fn update_service_user_last_used(&self, id: Uuid, timestamp: chrono::DateTime<chrono::Utc>) -> Result<()> { self.inner.update_service_user_last_used(id, timestamp).await }

    // Merge Operations
    async fn create_merge_operation(&self, operation: MergeOperation) -> Result<()> { self.inner.create_merge_operation(operation).await }
    async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<MergeOperation>> { self.inner.get_merge_operation(operation_id).await }
    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<MergeOperation>> { self.inner.list_merge_operations(tenant_id, catalog_name, pagination).await }
    async fn update_merge_operation_status(&self, operation_id: Uuid, status: MergeStatus) -> Result<()> { self.inner.update_merge_operation_status(operation_id, status).await }
    async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> { self.inner.complete_merge_operation(operation_id, result_commit_id).await }
    async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> { self.inner.abort_merge_operation(operation_id).await }
    
    // Merge Conflicts
    async fn create_merge_conflict(&self, conflict: MergeConflict) -> Result<()> { self.inner.create_merge_conflict(conflict).await }
    async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<MergeConflict>> { self.inner.get_merge_conflict(conflict_id).await }
    async fn list_merge_conflicts(&self, operation_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<MergeConflict>> { self.inner.list_merge_conflicts(operation_id, pagination).await }
    async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: ConflictResolution) -> Result<()> { self.inner.resolve_merge_conflict(conflict_id, resolution).await }
    async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> { self.inner.add_conflict_to_operation(operation_id, conflict_id).await }

    // Token Revocation
    async fn revoke_token(&self, token_id: Uuid, expires_at: DateTime<Utc>, reason: Option<String>) -> Result<()> { self.inner.revoke_token(token_id, expires_at, reason).await }
    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> { self.inner.is_token_revoked(token_id).await }
    async fn cleanup_expired_tokens(&self) -> Result<usize> { self.inner.cleanup_expired_tokens().await }
    async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<PaginationParams>) -> Result<Vec<TokenInfo>> { self.inner.list_active_tokens(tenant_id, user_id, pagination).await }
    async fn store_token(&self, token: TokenInfo) -> Result<()> { self.inner.store_token(token).await }
    async fn validate_token(&self, token: &str) -> Result<Option<TokenInfo>> { self.inner.validate_token(token).await }

    // System Config
    async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> { self.inner.get_system_settings(tenant_id).await }
    async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> { self.inner.update_system_settings(tenant_id, settings).await }

    // Federated Catalog
    async fn sync_federated_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<()> { self.inner.sync_federated_catalog(tenant_id, catalog_name).await }
    async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> { self.inner.get_federated_catalog_stats(tenant_id, catalog_name).await }
}
