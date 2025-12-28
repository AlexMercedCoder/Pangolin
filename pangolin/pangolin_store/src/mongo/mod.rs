pub mod main;
pub mod tenants;
pub mod warehouses;
pub mod catalogs;
pub mod namespaces;
pub mod assets;
pub mod branches;
pub mod tags;
pub mod commits;
pub mod users;
pub mod roles;
pub mod permissions;
pub mod tokens;
pub mod audit;
pub mod settings;
pub mod service_users;
pub mod access_requests;
pub mod merge;
pub mod business_metadata;
pub mod federated;
pub mod signer;
pub mod io;

pub use main::MongoStore;
use crate::CatalogStore;
use pangolin_core::model::*;
use pangolin_core::user::*;
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest};
use pangolin_core::token::TokenInfo;
use uuid::Uuid;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
impl CatalogStore for MongoStore {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        self.create_tenant_internal(tenant).await
    }
    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        self.get_tenant(id).await
    }
    async fn list_tenants(&self, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tenant>> {
        self.list_tenants(pagination).await
    }
    async fn update_tenant(&self, tenant_id: Uuid, updates: TenantUpdate) -> Result<Tenant> {
        self.update_tenant(tenant_id, updates).await
    }
    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        self.delete_tenant(tenant_id).await
    }

    // Warehouse Operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        self.create_warehouse(tenant_id, warehouse).await
    }
    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        self.get_warehouse(tenant_id, name).await
    }
    async fn list_warehouses(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Warehouse>> {
        self.list_warehouses(tenant_id, pagination).await
    }
    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
        self.update_warehouse(tenant_id, name, updates).await
    }
    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        self.delete_warehouse(tenant_id, name).await
    }

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        self.create_catalog(tenant_id, catalog).await
    }
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        self.get_catalog(tenant_id, name).await
    }
    async fn list_catalogs(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Catalog>> {
        self.list_catalogs(tenant_id, pagination).await
    }
    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: CatalogUpdate) -> Result<Catalog> {
        self.update_catalog(tenant_id, name, updates).await
    }
    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        self.delete_catalog(tenant_id, name).await
    }

    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        self.create_namespace(tenant_id, catalog_name, namespace).await
    }
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        self.get_namespace(tenant_id, catalog_name, namespace).await
    }
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Namespace>> {
        self.list_namespaces(tenant_id, catalog_name, parent, pagination).await
    }
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        self.delete_namespace(tenant_id, catalog_name, namespace).await
    }
    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: HashMap<String, String>) -> Result<()> {
        self.update_namespace_properties(tenant_id, catalog_name, namespace, properties).await
    }

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        self.create_asset(tenant_id, catalog_name, branch, namespace, asset).await
    }
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        self.get_asset(tenant_id, catalog_name, branch, namespace, name).await
    }
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Asset>> {
        self.list_assets(tenant_id, catalog_name, branch, namespace, pagination).await
    }
    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        self.get_asset_by_id(tenant_id, asset_id).await
    }
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        self.delete_asset(tenant_id, catalog_name, branch, namespace, name).await
    }
    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        self.rename_asset(tenant_id, catalog_name, branch, source_namespace, source_name, dest_namespace, dest_name).await
    }

    async fn count_namespaces(&self, tenant_id: Uuid) -> Result<usize> {
        self.count_namespaces(tenant_id).await
    }
    async fn count_assets(&self, tenant_id: Uuid) -> Result<usize> {
        self.count_assets(tenant_id).await
    }

    // Branch Operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        self.create_branch(tenant_id, catalog_name, branch).await
    }
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        self.get_branch(tenant_id, catalog_name, name).await
    }
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Branch>> {
        self.list_branches(tenant_id, catalog_name, pagination).await
    }
    async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_branch(tenant_id, catalog_name, name).await
    }
    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch: String, target_branch: String) -> Result<()> {
        self.merge_branch(tenant_id, catalog_name, source_branch, target_branch).await
    }

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        self.create_tag(tenant_id, catalog_name, tag).await
    }
    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        self.get_tag(tenant_id, catalog_name, name).await
    }
    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tag>> {
        self.list_tags(tenant_id, catalog_name, pagination).await
    }
    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_tag(tenant_id, catalog_name, name).await
    }

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        self.create_commit_internal(tenant_id, commit).await
    }
    async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        self.get_commit_internal(tenant_id, id).await
    }
    async fn get_commit_ancestry(&self, tenant_id: Uuid, commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> {
        self.get_commit_ancestry(tenant_id, commit_id, limit).await
    }

    // Bulk Operations
    async fn copy_assets_bulk(&self, tenant_id: Uuid, catalog_name: &str, src_branch: &str, dest_branch: &str, namespace: Option<String>) -> Result<usize> {
        self.copy_assets_bulk(tenant_id, catalog_name, src_branch, dest_branch, namespace).await
    }

    // User Management
    async fn create_user(&self, user: User) -> Result<()> {
        self.create_user_internal(user).await
    }
    async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        self.get_user(id).await
    }
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.get_user_by_username(username).await
    }
    async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<User>> {
        self.list_users(tenant_id, pagination).await
    }
    async fn update_user(&self, user: User) -> Result<()> {
        self.update_user(user).await
    }
    async fn delete_user(&self, id: Uuid) -> Result<()> {
        self.delete_user(id).await
    }

    // Role Management
    async fn create_role(&self, role: pangolin_core::permission::Role) -> Result<()> {
        self.create_role(role).await
    }
    async fn get_role(&self, id: Uuid) -> Result<Option<pangolin_core::permission::Role>> {
        self.get_role(id).await
    }
    async fn list_roles(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<pangolin_core::permission::Role>> {
        self.list_roles(tenant_id, pagination).await
    }
    async fn delete_role(&self, id: Uuid) -> Result<()> {
        self.delete_role(id).await
    }
    async fn update_role(&self, role: pangolin_core::permission::Role) -> Result<()> {
        self.update_role(role).await
    }

    // User Role Assignment
    async fn assign_role(&self, user_role: pangolin_core::permission::UserRole) -> Result<()> {
        self.assign_role(user_role).await
    }
    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        self.remove_role(user_id, role_id).await
    }
    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<pangolin_core::permission::UserRole>> {
        self.get_user_roles(user_id).await
    }

    // Permissions
    async fn create_permission(&self, permission: pangolin_core::permission::Permission) -> Result<()> {
        self.grant_permission(permission).await
    }
    async fn revoke_permission(&self, id: Uuid) -> Result<()> {
        self.revoke_permission(id).await
    }
    async fn list_user_permissions(&self, user_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<pangolin_core::permission::Permission>> {
        self.list_user_permissions(user_id, pagination).await
    }
    async fn list_permissions(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<pangolin_core::permission::Permission>> {
        self.list_permissions(tenant_id, pagination).await
    }

    // Token Management
    async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<TokenInfo>> {
        self.list_active_tokens(tenant_id, user_id, pagination).await
    }
    async fn store_token(&self, token_info: TokenInfo) -> Result<()> {
        self.store_token(token_info).await
    }
    async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        self.revoke_token(token_id, expires_at, reason).await
    }
    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        self.is_token_revoked(token_id).await
    }
    async fn cleanup_expired_tokens(&self) -> Result<usize> {
        self.cleanup_expired_tokens().await
    }
    async fn validate_token(&self, token: &str) -> Result<Option<TokenInfo>> {
        self.validate_token(token).await
    }

    // Audit Logging
    async fn log_audit_event(&self, _tenant_id: Uuid, entry: pangolin_core::audit::AuditLogEntry) -> Result<()> {
        self.log_audit_event(entry).await
    }
    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> {
        self.list_audit_events(tenant_id, filter).await
    }
    async fn get_audit_event(&self, _tenant_id: Uuid, id: Uuid) -> Result<Option<pangolin_core::audit::AuditLogEntry>> {
        self.get_audit_event(id).await
    }
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        self.count_audit_events(tenant_id, filter).await
    }

    // System Settings
    async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        self.get_system_settings(tenant_id).await
    }
    async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        self.update_system_settings(tenant_id, settings).await
    }

    // Service Users
    async fn create_service_user(&self, service_user: ServiceUser) -> Result<()> {
        self.create_service_user(service_user).await
    }
    async fn get_service_user(&self, id: Uuid) -> Result<Option<ServiceUser>> {
        self.get_service_user(id).await
    }
    async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<ServiceUser>> {
        self.get_service_user_by_api_key_hash(api_key_hash).await
    }
    async fn list_service_users(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<ServiceUser>> {
        self.list_service_users(tenant_id, pagination).await
    }
    async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, active: Option<bool>) -> Result<()> {
        let _ = self.update_service_user(id, name, description, None, active).await?;
        Ok(())
    }
    async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        self.delete_service_user(id).await
    }
    async fn update_service_user_last_used(&self, id: Uuid, _timestamp: chrono::DateTime<chrono::Utc>) -> Result<()> {
        self.update_service_user_last_used(id).await
    }

    // Maintenance
    async fn expire_snapshots(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, retention_ms: i64) -> Result<()> {
        self.expire_snapshots(tenant_id, catalog_name, branch, namespace, table, retention_ms).await
    }
    async fn remove_orphan_files(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, older_than_ms: i64) -> Result<()> {
        self.remove_orphan_files(tenant_id, catalog_name, branch, namespace, table, older_than_ms).await
    }

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        self.get_metadata_location(tenant_id, catalog_name, branch, namespace, table).await
    }
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        self.update_metadata_location(tenant_id, catalog_name, branch, namespace, table, expected_location, new_location).await
    }
    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        self.read_file(path).await
    }
    async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        self.write_file(path, data).await
    }

    // Access Requests
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        self.create_access_request(request).await
    }
    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        self.get_access_request(id).await
    }
    async fn list_access_requests(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<AccessRequest>> {
        self.list_access_requests(tenant_id, pagination).await
    }
    async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        self.update_access_request(request).await
    }

    // Business Metadata
    async fn upsert_business_metadata(&self, metadata: BusinessMetadata) -> Result<()> {
        self.upsert_business_metadata(metadata).await
    }
    async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<BusinessMetadata>> {
        self.get_business_metadata(asset_id).await
    }
    async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        self.delete_business_metadata(asset_id).await
    }

    // Discovery/Search
    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<BusinessMetadata>, String, Vec<String>)>> {
        self.search_assets(tenant_id, query, tags).await
    }
    async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        self.search_catalogs(tenant_id, query).await
    }
    async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        self.search_namespaces(tenant_id, query).await
    }
    async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        self.search_branches(tenant_id, query).await
    }

    // Federated Catalogs
    async fn sync_federated_catalog(&self, _tenant_id: Uuid, _catalog_name: &str) -> Result<()> {
        Ok(())
    }
    async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> {
        Ok(self.get_sync_stats(tenant_id, catalog_name).await?.unwrap_or_else(|| SyncStats {
            last_synced_at: None,
            sync_status: "Never Synced".to_string(),
            tables_synced: 0,
            namespaces_synced: 0,
            error_message: None,
        }))
    }

    // Merge Operations
    async fn create_merge_operation(&self, op: MergeOperation) -> Result<()> {
        self.create_merge_operation(op).await
    }
    async fn get_merge_operation(&self, id: Uuid) -> Result<Option<MergeOperation>> {
        self.get_merge_operation(id).await
    }
    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<MergeOperation>> {
        self.list_merge_operations(tenant_id, catalog_name, pagination).await
    }
    async fn update_merge_operation_status(&self, id: Uuid, status: MergeStatus) -> Result<()> {
        self.update_merge_operation_status(id, status).await
    }
    async fn complete_merge_operation(&self, id: Uuid, result_commit_id: Uuid) -> Result<()> {
        self.complete_merge_operation(id, result_commit_id).await
    }
    async fn abort_merge_operation(&self, id: Uuid) -> Result<()> {
        self.abort_merge_operation(id).await
    }

    // Merge Conflict Methods
    async fn create_merge_conflict(&self, conflict: MergeConflict) -> Result<()> {
        self.create_merge_conflict(conflict).await
    }
    async fn get_merge_conflict(&self, id: Uuid) -> Result<Option<MergeConflict>> {
        self.get_merge_conflict(id).await
    }
    async fn list_merge_conflicts(&self, merge_op_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<MergeConflict>> {
        self.list_merge_conflicts(merge_op_id, pagination).await
    }
    async fn resolve_merge_conflict(&self, id: Uuid, resolution: ConflictResolution) -> Result<()> {
        self.resolve_merge_conflict(id, resolution).await
    }
    async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> {
        self.add_conflict_to_operation(operation_id, conflict_id).await
    }
}
