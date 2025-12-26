// Submodule declarations
pub mod main;
mod tenants;
mod warehouses;
mod catalogs;
mod namespaces;
mod assets;
mod branches;
mod tags;
mod commits;
mod io;
mod audit;
mod users;
mod service_users;
mod roles;
mod permissions;
mod tokens;
mod business_metadata;
mod access_requests;
mod merge;
mod federated;
mod settings;
mod signer;

// Re-export the main struct
pub use main::MemoryStore;

use crate::CatalogStore;
use crate::signer::{Signer, Credentials};
use async_trait::async_trait;
use anyhow::Result;
use uuid::Uuid;
use chrono::{Utc, DateTime};
use pangolin_core::model::*; // Import model types to ensure they are available

// Implement CatalogStore trait with delegation to submodules
#[async_trait]
impl CatalogStore for MemoryStore {
    // Tenant operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        self.create_tenant_internal(tenant).await
    }
    
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>> {
        self.get_tenant_internal(tenant_id).await
    }
    
    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        self.list_tenants_internal().await
    }
    
    async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> {
        self.update_tenant_internal(tenant_id, updates).await
    }
    
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        self.delete_tenant_internal(tenant_id).await
    }
    
    // Warehouse operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        self.create_warehouse_internal(tenant_id, warehouse).await
    }
    
    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        self.get_warehouse_internal(tenant_id, name).await
    }
    
    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        self.list_warehouses_internal(tenant_id).await
    }
    
    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
        self.update_warehouse_internal(tenant_id, name, updates).await
    }
    
    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        self.delete_warehouse_internal(tenant_id, name).await
    }
    
    // Catalog operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        self.create_catalog_internal(tenant_id, catalog).await
    }
    
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        self.get_catalog_internal(tenant_id, name).await
    }
    
    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        self.list_catalogs_internal(tenant_id).await
    }
    
    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog> {
        self.update_catalog_internal(tenant_id, name, updates).await
    }
    
    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        self.delete_catalog_internal(tenant_id, name).await
    }
    
    // Namespace operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        self.create_namespace_internal(tenant_id, catalog_name, namespace).await
    }
    
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>> {
        self.list_namespaces_internal(tenant_id, catalog_name, parent).await
    }
    
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        self.get_namespace_internal(tenant_id, catalog_name, namespace).await
    }
    
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        self.delete_namespace_internal(tenant_id, catalog_name, namespace).await
    }
    
    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        self.update_namespace_properties_internal(tenant_id, catalog_name, namespace, properties).await
    }
    
    async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> {
        self.count_namespaces_internal(tenant_id).await
    }

    // Asset operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        self.create_asset_internal(tenant_id, catalog_name, branch, namespace, asset).await
    }
    
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        self.get_asset_internal(tenant_id, catalog_name, branch, namespace, name).await
    }
    
    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        self.get_asset_by_id_internal(tenant_id, asset_id).await
    }
    
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        self.list_assets_internal(tenant_id, catalog_name, branch, namespace).await
    }
    
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        self.delete_asset_internal(tenant_id, catalog_name, branch, namespace, name).await
    }
    
    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        self.rename_asset_internal(tenant_id, catalog_name, branch, source_namespace, source_name, dest_namespace, dest_name).await
    }
    
    async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> {
        self.count_assets_internal(tenant_id).await
    }
    
    // Branch operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        self.create_branch_internal(tenant_id, catalog_name, branch).await
    }
    
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        self.get_branch_internal(tenant_id, catalog_name, name).await
    }
    
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        self.list_branches_internal(tenant_id, catalog_name).await
    }
    
    async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_branch_internal(tenant_id, catalog_name, name).await
    }
    
    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch_name: String, target_branch_name: String) -> Result<()> {
        self.merge_branch_internal(tenant_id, catalog_name, source_branch_name, target_branch_name).await
    }
    
    // Tag operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        self.create_tag_internal(tenant_id, catalog_name, tag).await
    }
    
    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        self.get_tag_internal(tenant_id, catalog_name, name).await
    }
    
    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
        self.list_tags_internal(tenant_id, catalog_name).await
    }
    
    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_tag_internal(tenant_id, catalog_name, name).await
    }
    
    // Commit operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        self.create_commit_internal(tenant_id, commit).await
    }
    
    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>> {
        self.get_commit_internal(tenant_id, commit_id).await
    }
    
    // File I/O operations
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        self.get_metadata_location_internal(tenant_id, catalog_name, branch, namespace, table).await
    }
    
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        self.update_metadata_location_internal(tenant_id, catalog_name, branch, namespace, table, expected_location, new_location).await
    }
    
    async fn read_file(&self, location: &str) -> Result<Vec<u8>> {
        self.read_file_internal(location).await
    }
    
    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()> {
        self.write_file_internal(location, content).await
    }
    
    async fn expire_snapshots(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, retention_ms: i64) -> Result<()> {
        self.expire_snapshots_internal(tenant_id, catalog_name, branch, namespace, table, retention_ms).await
    }
    
    async fn remove_orphan_files(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, older_than_ms: i64) -> Result<()> {
        self.remove_orphan_files_internal(tenant_id, catalog_name, branch, namespace, table, older_than_ms).await
    }
    
    // Audit operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: pangolin_core::audit::AuditLogEntry) -> Result<()> {
        self.log_audit_event_internal(tenant_id, event).await
    }
    
    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> {
        self.list_audit_events_internal(tenant_id, filter).await
    }
    
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<pangolin_core::audit::AuditLogEntry>> {
        self.get_audit_event_internal(tenant_id, event_id).await
    }
    
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        self.count_audit_events_internal(tenant_id, filter).await
    }
    
    // User operations
    async fn create_user(&self, user: pangolin_core::user::User) -> Result<()> {
        self.create_user_internal(user).await
    }
    
    async fn get_user(&self, user_id: Uuid) -> Result<Option<pangolin_core::user::User>> {
        self.get_user_internal(user_id).await
    }
    
    async fn get_user_by_username(&self, username: &str) -> Result<Option<pangolin_core::user::User>> {
        self.get_user_by_username_internal(username).await
    }
    
    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<pangolin_core::user::User>> {
        self.list_users_internal(tenant_id).await
    }
    
    async fn update_user(&self, user: pangolin_core::user::User) -> Result<()> {
        self.update_user_internal(user).await
    }
    
    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        self.delete_user_internal(user_id).await
    }
    
    // Service user operations
    async fn create_service_user(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> {
        self.create_service_user_internal(service_user).await
    }
    
    async fn get_service_user(&self, service_user_id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> {
        self.get_service_user_internal(service_user_id).await
    }
    
    async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::user::ServiceUser>> {
        self.list_service_users_internal(tenant_id).await
    }
    
    async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, active: Option<bool>) -> Result<()> {
        self.update_service_user_internal(id, name, description, active).await
    }
    
    async fn delete_service_user(&self, service_user_id: Uuid) -> Result<()> {
        self.delete_service_user_internal(service_user_id).await
    }
    
    // Role operations
    async fn create_role(&self, role: pangolin_core::permission::Role) -> Result<()> {
        self.create_role_internal(role).await
    }
    
    async fn get_role(&self, role_id: Uuid) -> Result<Option<pangolin_core::permission::Role>> {
        self.get_role_internal(role_id).await
    }
    
    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::permission::Role>> {
        self.list_roles_internal(tenant_id).await
    }
    
    async fn update_role(&self, role: pangolin_core::permission::Role) -> Result<()> {
        self.update_role_internal(role).await
    }
    
    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        self.delete_role_internal(role_id).await
    }
    
    async fn assign_role(&self, user_role: pangolin_core::permission::UserRole) -> Result<()> {
        self.assign_role_internal(user_role).await
    }
    
    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        self.revoke_role_internal(user_id, role_id).await
    }
    
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<pangolin_core::permission::UserRole>> {
        self.get_user_roles_internal(user_id).await
    }
    
    // Permission operations
    async fn create_permission(&self, permission: pangolin_core::permission::Permission) -> Result<()> {
        self.create_permission_internal(permission).await
    }
    
    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        self.revoke_permission_internal(permission_id).await
    }
    
    async fn list_permissions(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::permission::Permission>> {
        self.list_permissions_internal(None, None).await
    }
    
    // Token operations
    async fn store_token(&self, token_info: pangolin_core::token::TokenInfo) -> Result<()> {
        self.store_token_internal(token_info).await
    }
    
    async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Vec<pangolin_core::token::TokenInfo>> {
        self.list_active_tokens_internal(tenant_id, user_id).await
    }
    
    async fn revoke_token(&self, _token_id: Uuid, expires_at: DateTime<Utc>, reason: Option<String>) -> Result<()> {
        self.revoke_token_internal(_token_id, expires_at, reason).await
    }
    
    // Business metadata operations
    async fn upsert_business_metadata(&self, metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> {
        self.upsert_business_metadata_internal(metadata).await
    }
    
    async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> {
        self.get_business_metadata_internal(asset_id).await
    }
    
    async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        self.delete_business_metadata_internal(asset_id).await
    }
    
    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>> {
        self.search_assets_internal(tenant_id, query, tags).await
    }
    
    async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        self.search_catalogs_internal(tenant_id, query).await
    }
    
    async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        self.search_namespaces_internal(tenant_id, query).await
    }
    
    async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        self.search_branches_internal(tenant_id, query).await
    }
    
    // Access request operations
    async fn create_access_request(&self, request: pangolin_core::business_metadata::AccessRequest) -> Result<()> {
        self.create_access_request_internal(request).await
    }
    
    async fn get_access_request(&self, request_id: Uuid) -> Result<Option<pangolin_core::business_metadata::AccessRequest>> {
        self.get_access_request_internal(request_id).await
    }
    
    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::business_metadata::AccessRequest>> {
        self.list_access_requests_internal(tenant_id).await
    }
    
    // Merge operations
    async fn create_merge_operation(&self, operation: pangolin_core::model::MergeOperation) -> Result<()> {
        self.create_merge_operation_internal(operation).await
    }
    
    async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<pangolin_core::model::MergeOperation>> {
        self.get_merge_operation_internal(operation_id).await
    }
    
    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<pangolin_core::model::MergeOperation>> {
        self.list_merge_operations_internal(tenant_id, catalog_name).await
    }
    
    async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        self.complete_merge_operation_internal(operation_id, result_commit_id).await
    }
    
    async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> {
        self.abort_merge_operation_internal(operation_id).await
    }
    
    async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<pangolin_core::model::MergeConflict>> {
        self.get_merge_conflict_internal(conflict_id).await
    }
    
    // Federated catalog operations
    async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> {
        self.get_federated_catalog_stats_internal(tenant_id, catalog_name).await
    }
    
    // System settings operations
    async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        self.get_system_settings_internal(tenant_id).await
    }
    
    async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        self.update_system_settings_internal(tenant_id, settings).await
    }
}

// Implement Signer trait
#[async_trait]
impl Signer for MemoryStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
        self.get_table_credentials_internal(location).await
    }
    
    async fn presign_get(&self, location: &str) -> Result<String> {
        self.presign_get_internal(location).await
    }
}
