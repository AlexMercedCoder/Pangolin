use crate::{CatalogStore, PaginationParams};
use anyhow::Result;
use async_trait::async_trait;
use pangolin_core::model::{
    Asset, Branch, Catalog, Commit, Namespace, Tag, Tenant, Warehouse, VendingStrategy,
};
use crate::signer::{Signer, Credentials};
use pangolin_core::user::{User, UserRole, OAuthProvider};
use pangolin_core::permission::{Role, Permission, PermissionGrant, UserRole as UserRoleAssignment};
use pangolin_core::audit::AuditLogEntry;
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use pangolin_core::token::TokenInfo;
use pangolin_core::model::{SystemSettings, SyncStats};
use object_store::{aws::AmazonS3Builder, ObjectStore, path::Path as ObjPath};

#[derive(Clone)]
pub struct PostgresStore {
    pub(crate) pool: PgPool,
    object_store_cache: crate::ObjectStoreCache,
    metadata_cache: crate::MetadataCache,
}

impl PostgresStore {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let max_connections = std::env::var("DATABASE_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(5);
        
        let min_connections = std::env::var("DATABASE_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(2);
        
        let connect_timeout = std::env::var("DATABASE_CONNECT_TIMEOUT")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);
        
        tracing::info!("Initializing PostgreSQL pool: max={}, min={}, timeout={}s", 
            max_connections, min_connections, connect_timeout);
        
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .acquire_timeout(std::time::Duration::from_secs(connect_timeout))
            .connect(connection_string)
            .await?;
        
        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { 
        pool,
        object_store_cache: crate::ObjectStoreCache::new(),
        metadata_cache: crate::MetadataCache::default(),
    })
    }
}

#[async_trait]
impl CatalogStore for PostgresStore {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        self.create_tenant(tenant).await
    }

    async fn get_tenant(&self, id: Uuid) -> Result<Option<Tenant>> {
        self.get_tenant(id).await
    }

    async fn list_tenants(&self, pagination: Option<PaginationParams>) -> Result<Vec<Tenant>> {
        self.list_tenants(pagination).await
    }

    async fn update_tenant(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
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

    async fn list_warehouses(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Warehouse>> {
        self.list_warehouses(tenant_id, pagination).await
    }

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
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

    async fn list_catalogs(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Catalog>> {
        self.list_catalogs(tenant_id, pagination).await
    }

    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
        self.update_catalog(tenant_id, name, updates).await
    }

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        self.delete_catalog(tenant_id, name).await
    }

    // Namespace Operations
    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        self.create_namespace(tenant_id, catalog_name, namespace).await
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        self.get_namespace(tenant_id, catalog_name, namespace).await
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, _parent: Option<String>, pagination: Option<PaginationParams>) -> Result<Vec<Namespace>> {
        self.list_namespaces(tenant_id, catalog_name, _parent, pagination).await
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        self.delete_namespace(tenant_id, catalog_name, namespace).await
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        self.update_namespace_properties(tenant_id, catalog_name, namespace, properties).await
    }

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        self.create_asset(tenant_id, catalog_name, branch, namespace, asset).await
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        self.get_asset(tenant_id, catalog_name, branch, namespace, name).await
    }

    async fn get_asset_by_id(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
        self.get_asset_by_id(tenant_id, asset_id).await
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<PaginationParams>) -> Result<Vec<Asset>> {
        self.list_assets(tenant_id, catalog_name, branch, namespace, pagination).await
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

    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<Branch>> {
        self.list_branches(tenant_id, catalog_name, pagination).await
    }

    async fn delete_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_branch(tenant_id, catalog_name, name).await
    }

    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, target_branch: String, source_branch: String) -> Result<()> {
        self.merge_branch(tenant_id, catalog_name, target_branch, source_branch).await
    }

    async fn copy_assets_bulk(
        &self, 
        tenant_id: Uuid, 
        catalog_name: &str, 
        src_branch: &str, 
        dest_branch: &str, 
        namespace: Option<String>
    ) -> Result<usize> {
        self.copy_assets_bulk(tenant_id, catalog_name, src_branch, dest_branch, namespace).await
    }

    // Token Management
    async fn list_active_tokens(&self, tenant_id: Uuid, user_id: Option<Uuid>, pagination: Option<PaginationParams>) -> Result<Vec<TokenInfo>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = if let Some(uid) = user_id {
             sqlx::query("SELECT t.token_id, t.user_id, t.token, t.expires_at FROM active_tokens t JOIN users u ON t.user_id = u.id WHERE u.tenant_id = $1 AND t.user_id = $2 AND t.expires_at > $3 LIMIT $4 OFFSET $5")
                .bind(tenant_id)
                .bind(uid)
                .bind(Utc::now())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
             sqlx::query("SELECT t.token_id, t.user_id, t.token, t.expires_at FROM active_tokens t JOIN users u ON t.user_id = u.id WHERE u.tenant_id = $1 AND t.expires_at > $2 LIMIT $3 OFFSET $4")
                .bind(tenant_id)
                .bind(Utc::now())
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        let mut tokens = Vec::new();
        for row in rows {
            tokens.push(TokenInfo {
                id: row.get("token_id"),
                tenant_id,
                user_id: row.get("user_id"),
                username: "unknown".to_string(),
                token: Some(row.get("token")),
                expires_at: row.get("expires_at"),
                created_at: Utc::now(),
                is_valid: true,
            });
        }
        Ok(tokens)
    }

    async fn store_token(&self, token_info: TokenInfo) -> Result<()> {
        sqlx::query("INSERT INTO active_tokens (token_id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)")
            .bind(token_info.id)
            .bind(token_info.user_id)
            .bind(token_info.token.unwrap_or_default())
            .bind(token_info.expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // System Settings
    async fn get_system_settings(&self, tenant_id: Uuid) -> Result<SystemSettings> {
        self.get_system_settings(tenant_id).await
    }

    async fn update_system_settings(&self, tenant_id: Uuid, settings: SystemSettings) -> Result<SystemSettings> {
        self.update_system_settings(tenant_id, settings).await
    }

    // Service User Operations
    async fn create_service_user(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> {
        let role_str = match service_user.role {
            UserRole::Root => "Root",
            UserRole::TenantAdmin => "TenantAdmin",
            UserRole::TenantUser => "TenantUser",
        };

        sqlx::query(
            "INSERT INTO service_users (id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
        )
        .bind(service_user.id)
        .bind(service_user.name)
        .bind(service_user.description)
        .bind(service_user.tenant_id)
        .bind(service_user.api_key_hash)
        .bind(role_str)
        .bind(service_user.created_at)
        .bind(service_user.created_by)
        .bind(service_user.last_used)
        .bind(service_user.expires_at)
        .bind(service_user.active)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_service_user(&self, id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            self.row_to_service_user(row)
        } else {
            Ok(None)
        }
    }

    async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<pangolin_core::user::ServiceUser>> {
        let row = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE api_key_hash = $1 AND active = true"
        )
        .bind(api_key_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            self.row_to_service_user(row)
        } else {
            Ok(None)
        }
    }

    async fn list_service_users(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::user::ServiceUser>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, name, description, tenant_id, api_key_hash, role, created_at, created_by, last_used, expires_at, active 
             FROM service_users WHERE tenant_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut service_users = Vec::new();
        for row in rows {
            if let Some(su) = self.row_to_service_user(row)? {
                service_users.push(su);
            }
        }

        Ok(service_users)
    }

    async fn update_service_user(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        active: Option<bool>,
    ) -> Result<()> {
        let mut updates = Vec::new();
        let mut bind_idx = 2; // $1 is id

        if name.is_some() {
            updates.push(format!("name = ${}", bind_idx));
            bind_idx += 1;
        }
        if description.is_some() {
            updates.push(format!("description = ${}", bind_idx));
            bind_idx += 1;
        }
        if active.is_some() {
            updates.push(format!("active = ${}", bind_idx));
        }

        if updates.is_empty() {
            return Ok(());
        }

        let query_str = format!(
            "UPDATE service_users SET {} WHERE id = $1",
            updates.join(", ")
        );

        let mut query = sqlx::query(&query_str).bind(id);

        if let Some(n) = name {
            query = query.bind(n);
        }
        if let Some(d) = description {
            query = query.bind(d);
        }
        if let Some(a) = active {
            query = query.bind(a);
        }

        query.execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM service_users WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_service_user_last_used(&self, id: Uuid, timestamp: DateTime<Utc>) -> Result<()> {
        sqlx::query("UPDATE service_users SET last_used = $1 WHERE id = $2")
            .bind(timestamp)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Merge Operation Methods
    async fn create_merge_operation(&self, operation: pangolin_core::model::MergeOperation) -> Result<()> {
        let status_str = match operation.status {
            pangolin_core::model::MergeStatus::Pending => "Pending",
            pangolin_core::model::MergeStatus::Conflicted => "Conflicted",
            pangolin_core::model::MergeStatus::Resolving => "Resolving",
            pangolin_core::model::MergeStatus::Ready => "Ready",
            pangolin_core::model::MergeStatus::Completed => "Completed",
            pangolin_core::model::MergeStatus::Aborted => "Aborted",
        };

        sqlx::query(
            "INSERT INTO merge_operations (id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, conflicts, initiated_by, initiated_at, completed_at, result_commit_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)"
        )
        .bind(operation.id)
        .bind(operation.tenant_id)
        .bind(operation.catalog_name)
        .bind(operation.source_branch)
        .bind(operation.target_branch)
        .bind(operation.base_commit_id)
        .bind(status_str)
        .bind(&operation.conflicts)
        .bind(operation.initiated_by)
        .bind(operation.initiated_at)
        .bind(operation.completed_at)
        .bind(operation.result_commit_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<pangolin_core::model::MergeOperation>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, conflicts, initiated_by, initiated_at, completed_at, result_commit_id
             FROM merge_operations WHERE id = $1"
        )
        .bind(operation_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "Pending" => pangolin_core::model::MergeStatus::Pending,
                "Conflicted" => pangolin_core::model::MergeStatus::Conflicted,
                "Resolving" => pangolin_core::model::MergeStatus::Resolving,
                "Ready" => pangolin_core::model::MergeStatus::Ready,
                "Completed" => pangolin_core::model::MergeStatus::Completed,
                "Aborted" => pangolin_core::model::MergeStatus::Aborted,
                _ => pangolin_core::model::MergeStatus::Pending,
            };

            Ok(Some(pangolin_core::model::MergeOperation {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                catalog_name: row.get("catalog_name"),
                source_branch: row.get("source_branch"),
                target_branch: row.get("target_branch"),
                base_commit_id: row.get("base_commit_id"),
                status,
                conflicts: row.get("conflicts"),
                initiated_by: row.get("initiated_by"),
                initiated_at: row.get("initiated_at"),
                completed_at: row.get("completed_at"),
                result_commit_id: row.get("result_commit_id"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::model::MergeOperation>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, tenant_id, catalog_name, source_branch, target_branch, base_commit_id, status, conflicts, initiated_by, initiated_at, completed_at, result_commit_id
             FROM merge_operations WHERE tenant_id = $1 AND catalog_name = $2 ORDER BY initiated_at DESC LIMIT $3 OFFSET $4"
        )
        .bind(tenant_id)
        .bind(catalog_name)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut operations = Vec::new();
        for row in rows {
            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "Pending" => pangolin_core::model::MergeStatus::Pending,
                "Conflicted" => pangolin_core::model::MergeStatus::Conflicted,
                "Resolving" => pangolin_core::model::MergeStatus::Resolving,
                "Ready" => pangolin_core::model::MergeStatus::Ready,
                "Completed" => pangolin_core::model::MergeStatus::Completed,
                "Aborted" => pangolin_core::model::MergeStatus::Aborted,
                _ => pangolin_core::model::MergeStatus::Pending,
            };

            operations.push(pangolin_core::model::MergeOperation {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                catalog_name: row.get("catalog_name"),
                source_branch: row.get("source_branch"),
                target_branch: row.get("target_branch"),
                base_commit_id: row.get("base_commit_id"),
                status,
                conflicts: row.get("conflicts"),
                initiated_by: row.get("initiated_by"),
                initiated_at: row.get("initiated_at"),
                completed_at: row.get("completed_at"),
                result_commit_id: row.get("result_commit_id"),
            });
        }

        Ok(operations)
    }

    async fn update_merge_operation_status(&self, operation_id: Uuid, status: pangolin_core::model::MergeStatus) -> Result<()> {
        let status_str = match status {
            pangolin_core::model::MergeStatus::Pending => "Pending",
            pangolin_core::model::MergeStatus::Conflicted => "Conflicted",
            pangolin_core::model::MergeStatus::Resolving => "Resolving",
            pangolin_core::model::MergeStatus::Ready => "Ready",
            pangolin_core::model::MergeStatus::Completed => "Completed",
            pangolin_core::model::MergeStatus::Aborted => "Aborted",
        };

        sqlx::query("UPDATE merge_operations SET status = $1 WHERE id = $2")
            .bind(status_str)
            .bind(operation_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE merge_operations SET status = 'Completed', result_commit_id = $1, completed_at = $2 WHERE id = $3"
        )
        .bind(result_commit_id)
        .bind(Utc::now())
        .bind(operation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE merge_operations SET status = 'Aborted', completed_at = $1 WHERE id = $2"
        )
        .bind(Utc::now())
        .bind(operation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Merge Conflict Methods
    async fn create_merge_conflict(&self, conflict: pangolin_core::model::MergeConflict) -> Result<()> {
        let conflict_type_str = serde_json::to_string(&conflict.conflict_type)?;
        let resolution_json = conflict.resolution.as_ref().map(|r| serde_json::to_value(r)).transpose()?;

        sqlx::query(
            "INSERT INTO merge_conflicts (id, merge_operation_id, conflict_type, asset_id, description, resolution, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)"
        )
        .bind(conflict.id)
        .bind(conflict.merge_operation_id)
        .bind(conflict_type_str)
        .bind(conflict.asset_id)
        .bind(conflict.description)
        .bind(resolution_json)
        .bind(conflict.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<pangolin_core::model::MergeConflict>> {
        let row = sqlx::query(
            "SELECT id, merge_operation_id, conflict_type, asset_id, description, resolution, created_at
             FROM merge_conflicts WHERE id = $1"
        )
        .bind(conflict_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let conflict_type_str: String = row.get("conflict_type");
            let conflict_type: pangolin_core::model::ConflictType = serde_json::from_str(&conflict_type_str)?;
            
            let resolution_json: Option<serde_json::Value> = row.get("resolution");
            let resolution = resolution_json.map(|v| serde_json::from_value(v)).transpose()?;

            Ok(Some(pangolin_core::model::MergeConflict {
                id: row.get("id"),
                merge_operation_id: row.get("merge_operation_id"),
                conflict_type,
                asset_id: row.get("asset_id"),
                description: row.get("description"),
                resolution,
                created_at: row.get("created_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_merge_conflicts(&self, operation_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<pangolin_core::model::MergeConflict>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT id, merge_operation_id, conflict_type, asset_id, description, resolution, created_at
             FROM merge_conflicts WHERE merge_operation_id = $1 ORDER BY created_at LIMIT $2 OFFSET $3"
        )
        .bind(operation_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut conflicts = Vec::new();
        for row in rows {
            let conflict_type_str: String = row.get("conflict_type");
            let conflict_type: pangolin_core::model::ConflictType = serde_json::from_str(&conflict_type_str)?;
            
            let resolution_json: Option<serde_json::Value> = row.get("resolution");
            let resolution = resolution_json.map(|v| serde_json::from_value(v)).transpose()?;

            conflicts.push(pangolin_core::model::MergeConflict {
                id: row.get("id"),
                merge_operation_id: row.get("merge_operation_id"),
                conflict_type,
                asset_id: row.get("asset_id"),
                description: row.get("description"),
                resolution,
                created_at: row.get("created_at"),
            });
        }

        Ok(conflicts)
    }

    async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: pangolin_core::model::ConflictResolution) -> Result<()> {
        let resolution_json = serde_json::to_value(&resolution)?;

        sqlx::query("UPDATE merge_conflicts SET resolution = $1 WHERE id = $2")
            .bind(resolution_json)
            .bind(conflict_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE merge_operations SET conflicts = array_append(conflicts, $1) WHERE id = $2 AND NOT ($1 = ANY(conflicts))"
        )
        .bind(conflict_id)
        .bind(operation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Federated Catalog Operations
    async fn sync_federated_catalog(&self, tenant_id: Uuid, catalog_name: &str) -> Result<()> {
        let stats = SyncStats {
            last_synced_at: Some(Utc::now()),
            sync_status: "Success".to_string(),
            tables_synced: 0,
            namespaces_synced: 0,
            error_message: None,
        };
        
        sqlx::query("INSERT INTO federated_sync_stats (tenant_id, catalog_name, stats) VALUES ($1, $2, $3) ON CONFLICT(tenant_id, catalog_name) DO UPDATE SET stats = $4")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(serde_json::to_value(&stats)?)
            .bind(serde_json::to_value(&stats)?)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }

    async fn get_federated_catalog_stats(&self, tenant_id: Uuid, catalog_name: &str) -> Result<SyncStats> {
        let row = sqlx::query("SELECT stats FROM federated_sync_stats WHERE tenant_id = $1 AND catalog_name = $2")
            .bind(tenant_id)
            .bind(catalog_name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(serde_json::from_value(row.get("stats"))?)
        } else {
            Ok(SyncStats {
                last_synced_at: None,
                sync_status: "Never Synced".to_string(),
                tables_synced: 0,
                namespaces_synced: 0,
                error_message: None,
            })
        }
    }

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        self.create_commit(tenant_id, commit).await
    }

    async fn get_commit(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        self.get_commit(tenant_id, id).await
    }

    async fn get_commit_ancestry(
        &self, 
        tenant_id: Uuid, 
        head_commit_id: Uuid, 
        limit: usize
    ) -> Result<Vec<Commit>> {
        self.get_commit_ancestry(tenant_id, head_commit_id, limit).await
    }


    // File Operations (Not supported in Postgres directly, use S3 or bytea)
    // For metadata files, we can store in a separate table or just return error if not supported.
    // Ideally, PostgresStore should be paired with S3 for file storage, or store files in a `files` table.
    // Let's add a `files` table to schema or just use S3 for files and Postgres for metadata.
    // Requirement says "Shared Metadata Backend". Files are usually on S3.
    // But `CatalogStore` trait has `read_file`/`write_file`.
    // Let's implement a simple `files` table for small metadata files.
    

    async fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        // Use metadata cache for metadata.json files
        if path.ends_with("metadata.json") || path.ends_with(".metadata.json") {
            return self.metadata_cache.get_or_fetch(path, || async {
                self.read_file_uncached(path).await
            }).await;
        }
        
        // Non-metadata files bypass cache
        self.read_file_uncached(path).await
    }


    async fn write_file(&self, path: &str, data: Vec<u8>) -> Result<()> {
        // Invalidate metadata cache on write
        if path.ends_with("metadata.json") || path.ends_with(".metadata.json") {
            self.metadata_cache.invalidate(path).await;
        }
        
        // Try to look up warehouse credentials first
        if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
            if path.starts_with("s3://") || path.starts_with("az://") || path.starts_with("gs://") {
                // Use cached object store
                let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
                let store = self.object_store_cache.try_get_or_insert::<_, anyhow::Error>(cache_key, || {
                    let os: Box<dyn object_store::ObjectStore> = crate::object_store_factory::create_object_store(&warehouse.storage_config, path)
                        .map_err(|e| anyhow::anyhow!("Failed to create object store for warehouse: {}", e))?;
                    let arc_os: Arc<dyn object_store::ObjectStore> = Arc::from(os);
                    Ok(arc_os)
                })?;

                tracing::info!("write_file: using warehouse='{}' for path='{}'", warehouse.name, path);
                
                // Extract key relative to bucket
                let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                     rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
                } else {
                     path
                };
                
                store.put(&object_store::path::Path::from(key), data.into()).await?;
                return Ok(());
            }
        }

        // Fallback to existing logic (Global Env Vars)
        if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            tracing::info!("write_file: s3 path='{}' bucket='{}' key='{}'", path, bucket, key);
            
            // Use cached object store for fallback
            let store = self.object_store_cache.try_get_or_insert::<_, anyhow::Error>("fallback_s3".to_string(), || {
                tracing::info!("Initializing fallback S3 store for bucket '{}'", bucket);
                let mut builder = AmazonS3Builder::new()
                    .with_bucket_name(bucket)
                    .with_allow_http(true);
                    
                 if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                     builder = builder.with_endpoint(endpoint);
                 }
                 if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                     builder = builder.with_access_key_id(key_id);
                 }
                 if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                     builder = builder.with_secret_access_key(secret);
                 }
                 if let Ok(region) = std::env::var("AWS_REGION") {
                     builder = builder.with_region(region);
                 }
                 
                 let os: Box<dyn object_store::ObjectStore> = Box::new(builder.build()?);
                 Ok(Arc::from(os))
            })?;

            store.put(&object_store::path::Path::from(key), data.into()).await?;
            return Ok(());
             
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Postgres store"))
        }
    }

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        self.create_tag(tenant_id, catalog_name, tag).await
    }

    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        self.get_tag(tenant_id, catalog_name, name).await
    }

    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<PaginationParams>) -> Result<Vec<Tag>> {
        self.list_tags(tenant_id, catalog_name, pagination).await
    }

    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        self.delete_tag(tenant_id, catalog_name, name).await
    }

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> {
        self.log_audit_event(tenant_id, event).await
    }

    async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        self.list_audit_events(tenant_id, filter).await
    }
    
    async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> {
        self.get_audit_event(tenant_id, event_id).await
    }
    
    async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = $1");
        
        let mut conditions = Vec::new();
        let mut param_count = 2;
        
        // Build same WHERE clause as list_audit_events
        if let Some(ref f) = filter {
            if f.user_id.is_some() {
                conditions.push(format!("user_id = ${}", param_count));
                param_count += 1;
            }
            if f.action.is_some() {
                conditions.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if f.resource_type.is_some() {
                conditions.push(format!("resource_type = ${}", param_count));
                param_count += 1;
            }
            if f.resource_id.is_some() {
                conditions.push(format!("resource_id = ${}", param_count));
                param_count += 1;
            }
            if f.start_time.is_some() {
                conditions.push(format!("timestamp >= ${}", param_count));
                param_count += 1;
            }
            if f.end_time.is_some() {
                conditions.push(format!("timestamp <= ${}", param_count));
                param_count += 1;
            }
            if f.result.is_some() {
                conditions.push(format!("result = ${}", param_count));
                param_count += 1;
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        // Build and execute query
        let mut query_builder = sqlx::query(&query).bind(tenant_id);
        
        if let Some(f) = filter {
            if let Some(user_id) = f.user_id {
                query_builder = query_builder.bind(user_id);
            }
            if let Some(action) = f.action {
                query_builder = query_builder.bind(action);
            }
            if let Some(resource_type) = f.resource_type {
                query_builder = query_builder.bind(resource_type);
            }
            if let Some(resource_id) = f.resource_id {
                query_builder = query_builder.bind(resource_id);
            }
            if let Some(start_time) = f.start_time {
                query_builder = query_builder.bind(start_time);
            }
            if let Some(end_time) = f.end_time {
                query_builder = query_builder.bind(end_time);
            }
            if let Some(result) = f.result {
                query_builder = query_builder.bind(result);
            }
        }
        
        let row = query_builder.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }

    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }

    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }

    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>> {
        let mut sql = String::from(
            "SELECT 
                a.id, a.tenant_id, a.catalog_name, a.namespace_path, a.name, a.asset_type, a.metadata_location, a.properties as asset_properties, a.branch_name,
                m.id as meta_id, m.description, m.tags, m.properties as meta_properties, m.discoverable, m.created_by as meta_created_by, 
                m.created_at as meta_created_at, m.updated_by as meta_updated_by, m.updated_at as meta_updated_at
            FROM assets a
            LEFT JOIN business_metadata m ON a.id = m.asset_id
            WHERE a.tenant_id = $1 AND (a.name ILIKE $2 OR m.description ILIKE $2)"
        );

        let query_pattern = format!("%{}%", query);
        let mut param_index = 3;
        
        if let Some(ref tag_list) = tags {
            if !tag_list.is_empty() {
                sql.push_str(&format!(" AND m.tags @> ${}::jsonb", param_index));
                param_index += 1;
            }
        }

        let mut query_builder = sqlx::query(&sql)
            .bind(tenant_id)
            .bind(&query_pattern);

        if let Some(ref tag_list) = tags {
            if !tag_list.is_empty() {
                let tags_json = serde_json::to_value(tag_list)?;
                query_builder = query_builder.bind(tags_json);
            }
        }

        let rows = query_builder.fetch_all(&self.pool).await?;
        let mut results = Vec::new();

        for row in rows {
            let asset_type_str: String = row.get("asset_type");
            let kind = match asset_type_str.as_str() {
                "IcebergTable" => pangolin_core::model::AssetType::IcebergTable,
                "View" => pangolin_core::model::AssetType::View,
                _ => pangolin_core::model::AssetType::IcebergTable,
            };

            let asset = Asset {
                id: row.get("id"),
                name: row.get("name"),
                kind,
                location: row.get::<Option<String>, _>("metadata_location").unwrap_or_default(),
                properties: serde_json::from_value(row.get("asset_properties")).unwrap_or_default(),
            };

            let meta_id: Option<Uuid> = row.get("meta_id");
            let metadata = if let Some(id) = meta_id {
                Some(pangolin_core::business_metadata::BusinessMetadata {
                    id,
                    asset_id: asset.id,
                    description: row.get("description"),
                    tags: serde_json::from_value(row.get("tags")).unwrap_or_default(),
                    properties: serde_json::from_value(row.get("meta_properties")).unwrap_or_default(),
                    discoverable: row.get("discoverable"),
                    created_by: row.get("meta_created_by"),
                    created_at: row.get::<DateTime<Utc>, _>("meta_created_at"),
                    updated_by: row.get("meta_updated_by"),
                    updated_at: row.get::<DateTime<Utc>, _>("meta_updated_at"),
                })
            } else {
                None
            };
            
            let catalog_name: String = row.get("catalog_name");
            let namespace_path: String = row.get("namespace_path");
            let namespace: Vec<String> = namespace_path.split('\x1F').map(String::from).collect();

            results.push((asset, metadata, catalog_name, namespace));
        }

        Ok(results)
    }

    async fn search_catalogs(&self, tenant_id: Uuid, query: &str) -> Result<Vec<Catalog>> {
        let query_pattern = format!("%{}%", query);
        let rows = sqlx::query("SELECT id, name, catalog_type, warehouse_name, storage_location, federated_config, properties FROM catalogs WHERE tenant_id = $1 AND name ILIKE $2")
            .bind(tenant_id)
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut catalogs = Vec::new();
        for row in rows {
            catalogs.push(Catalog {
                id: row.get("id"),
                name: row.get("name"),
                catalog_type: {
                    let s: String = row.get("catalog_type");
                    if s.to_lowercase() == "federated" { pangolin_core::model::CatalogType::Federated } else { pangolin_core::model::CatalogType::Local }
                },
                warehouse_name: row.get("warehouse_name"),
                storage_location: row.get("storage_location"),
                federated_config: row.get::<Option<serde_json::Value>, _>("federated_config").and_then(|v| serde_json::from_value(v).ok()),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            });
        }
        Ok(catalogs)
    }

    async fn search_namespaces(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Namespace, String)>> {
        let query_pattern = format!("%{}%", query);
        // Postgres stores namespace_path as TEXT[]
        let rows = sqlx::query("SELECT catalog_name, namespace_path, properties FROM namespaces WHERE tenant_id = $1 AND array_to_string(namespace_path, '.') ILIKE $2")
            .bind(tenant_id)
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut namespaces = Vec::new();
        for row in rows {
            namespaces.push((Namespace {
                name: row.get("namespace_path"),
                properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            }, row.get("catalog_name")));
        }
        Ok(namespaces)
    }

    async fn search_branches(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        let query_pattern = format!("%{}%", query);
        let rows = sqlx::query("SELECT catalog_name, name, head_commit_id, branch_type, assets FROM branches WHERE tenant_id = $1 AND name ILIKE $2")
            .bind(tenant_id)
            .bind(&query_pattern)
            .fetch_all(&self.pool)
            .await?;

        let mut branches = Vec::new();
        for row in rows {
            branches.push((Branch {
                name: row.get("name"),
                head_commit_id: row.get("head_commit_id"),
                branch_type: {
                    let s: String = row.get("branch_type");
                    if s.to_lowercase() == "ingest" { pangolin_core::model::BranchType::Ingest } else { pangolin_core::model::BranchType::Experimental }
                },
                assets: row.get("assets"),
            }, row.get("catalog_name")));
        }
        Ok(branches)
    }

    // Access Request Operations
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        self.create_access_request(request).await
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        self.get_access_request(id).await
    }

    async fn list_access_requests(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<AccessRequest>> {
        self.list_access_requests(tenant_id, pagination).await
    }

    async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        self.update_access_request(request).await
    }

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
        // Postgres stores namespace_path as TEXT[]
        let row = sqlx::query("SELECT metadata_location FROM assets WHERE tenant_id = $1 AND catalog_name = $2 AND branch_name = $3 AND namespace_path = $4 AND name = $5")
            .bind(tenant_id)
            .bind(catalog_name)
            .bind(&branch_name)
            .bind(&namespace)
            .bind(table)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            Ok(row.get("metadata_location"))
        } else {
            Ok(None)
        }
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
        let result = if let Some(expected) = expected_location {
            // Update only if current location matches expected
            // Sync properties JSONB as well
            sqlx::query("UPDATE assets SET metadata_location = $1, properties = jsonb_set(COALESCE(properties, '{}'::jsonb), '{metadata_location}', to_jsonb($1::text)) WHERE tenant_id = $2 AND catalog_name = $3 AND branch_name = $4 AND namespace_path = $5 AND name = $6 AND metadata_location = $7")
                .bind(&new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&branch_name)
                .bind(&namespace)
                .bind(&table)
                .bind(&expected)
                .execute(&self.pool)
                .await?
        } else {
            // Update if no metadata_location exists (create or first commit)
            sqlx::query("UPDATE assets SET metadata_location = $1, properties = jsonb_set(COALESCE(properties, '{}'::jsonb), '{metadata_location}', to_jsonb($1::text)) WHERE tenant_id = $2 AND catalog_name = $3 AND branch_name = $4 AND namespace_path = $5 AND name = $6 AND metadata_location IS NULL")
                .bind(&new_location)
                .bind(tenant_id)
                .bind(catalog_name)
                .bind(&branch_name)
                .bind(&namespace)
                .bind(&table)
                .execute(&self.pool)
                .await?
        };

        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("CAS check failed: Metadata location mismatch or asset not found"));
        }

        Ok(())
    }



    // User Operations
    async fn create_user(&self, user: User) -> Result<()> {
        self.create_user(user).await
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<User>> {
        self.get_user(user_id).await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        self.get_user_by_username(username).await
    }

    async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<PaginationParams>) -> Result<Vec<User>> {
        self.list_users(tenant_id, pagination).await
    }

    async fn update_user(&self, user: User) -> Result<()> {
        self.update_user(user).await
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        self.delete_user(user_id).await
    }

    // Role Operations
    async fn create_role(&self, role: Role) -> Result<()> {
        self.create_role(role).await
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<Role>> {
        self.get_role(role_id).await
    }

    async fn list_roles(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Role>> {
        self.list_roles(tenant_id, pagination).await
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        self.delete_role(role_id).await
    }

    async fn update_role(&self, role: Role) -> Result<()> {
        self.update_role(role).await
    }

    async fn assign_role(&self, user_role: UserRoleAssignment) -> Result<()> {
        self.assign_role(user_role).await
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        self.revoke_role(user_id, role_id).await
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRoleAssignment>> {
        self.get_user_roles(user_id).await
    }

    // Direct Permission Operations
    // Direct Permission Operations
    async fn create_permission(&self, permission: Permission) -> Result<()> {
        self.create_permission(permission).await
    }

    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        self.revoke_permission(permission_id).await
    }

    async fn list_user_permissions(&self, user_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>> {
        self.list_user_permissions(user_id, pagination).await
    }

    async fn list_permissions(&self, tenant_id: Uuid, pagination: Option<PaginationParams>) -> Result<Vec<Permission>> {
        self.list_permissions(tenant_id, pagination).await
    }

    // Token Revocation Operations
    async fn revoke_token(&self, token_id: Uuid, expires_at: chrono::DateTime<chrono::Utc>, reason: Option<String>) -> Result<()> {
        self.revoke_token(token_id, expires_at, reason).await
    }

    async fn is_token_revoked(&self, token_id: Uuid) -> Result<bool> {
        self.is_token_revoked(token_id).await
    }

    async fn cleanup_expired_tokens(&self) -> Result<usize> {
        self.cleanup_expired_tokens().await
    }
}



#[async_trait]
impl Signer for PostgresStore {
    async fn get_table_credentials(&self, location: &str) -> Result<crate::signer::Credentials> {
        // 1. Find the warehouse that owns this location by querying all warehouses
        let rows = sqlx::query("SELECT id, name, storage_config, use_sts, vending_strategy FROM warehouses")
            .fetch_all(&self.pool)
            .await?;
        
        let mut target_warehouse = None;
        
        for row in rows {
            let storage_config_json: serde_json::Value = row.get("storage_config");
            let storage_config: HashMap<String, String> = serde_json::from_value(storage_config_json)?;
            
            // Check matches
             // Check AWS S3
            if let Some(bucket) = storage_config.get("s3.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
            // Check Azure
            if let Some(container) = storage_config.get("azure.container") {
                if location.contains(container) {
                     target_warehouse = Some(row);
                     break;
                }
            }
            // Check GCP
            if let Some(bucket) = storage_config.get("gcp.bucket") {
                if location.contains(bucket) {
                    target_warehouse = Some(row);
                    break;
                }
            }
        }
        
        let row = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;
        
        let storage_config_json: serde_json::Value = row.get("storage_config");
        let storage_config: HashMap<String, String> = serde_json::from_value(storage_config_json)?;
        let use_sts: bool = row.try_get("use_sts").unwrap_or(false);
        let vending_strategy: Option<VendingStrategy> = row.try_get("vending_strategy")
            .ok()
            .and_then(|v: serde_json::Value| serde_json::from_value(v).ok());
            
        // 2. Check Vending Strategy
        if let Some(strategy) = vending_strategy {
             match strategy {
                 VendingStrategy::AwsSts { role_arn: _, external_id: _ } => {
                     Err(anyhow::anyhow!("AWS STS vending not implemented yet via VendingStrategy in PostgresStore"))
                 }
                 VendingStrategy::AwsStatic { access_key_id, secret_access_key } => {
                     Ok(Credentials::Aws {
                         access_key_id,
                         secret_access_key,
                         session_token: None,
                         expiration: None,
                     })
                 }
                 VendingStrategy::AzureSas { account_name, account_key } => {
                     let signer = crate::azure_signer::AzureSigner::new(account_name.clone(), account_key);
                     let sas_token = signer.generate_sas_token(location).await?;
                     Ok(Credentials::Azure {
                         sas_token,
                         account_name,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::GcpDownscoped { service_account_email, private_key } => {
                     let signer = crate::gcp_signer::GcpSigner::new(service_account_email, private_key);
                     let access_token = signer.generate_downscoped_token(location).await?;
                     Ok(Credentials::Gcp {
                         access_token,
                         expiration: chrono::Utc::now() + chrono::Duration::hours(1),
                     })
                 }
                 VendingStrategy::None => Err(anyhow::anyhow!("Vending disabled")),
             }
        } else {
            // Legacy Logic
            let access_key = storage_config.get("s3.access-key-id")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id"))?;
            let secret_key = storage_config.get("s3.secret-access-key")
                .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key"))?;
                
            if use_sts {
                // Existing STS Logic restored for backward compatibility
                let region = storage_config.get("s3.region")
                    .map(|s| s.as_str())
                    .unwrap_or("us-east-1");
                    
                let endpoint = storage_config.get("s3.endpoint")
                    .map(|s| s.as_str());

                let creds = aws_credential_types::Credentials::new(
                    access_key.to_string(),
                    secret_key.to_string(),
                    None,
                    None,
                    "legacy_provider"
                );
                
                let config_loader = aws_config::from_env()
                    .region(aws_config::Region::new(region.to_string()))
                    .credentials_provider(creds);
                    
                let config = if let Some(ep) = endpoint {
                     config_loader.endpoint_url(ep).load().await
                } else {
                     config_loader.load().await
                };
                
                let client = aws_sdk_sts::Client::new(&config);
                
                // Assume Role logic was present in legacy? Or just GetSessionToken?
                // Checking SqliteStore, it tried AssumeRole if role-arn present, else GetSessionToken.
                // Replicating that for consistency.
                
                let role_arn = storage_config.get("s3.role-arn").map(|s| s.as_str());
            
                if let Some(arn) = role_arn {
                     let resp = client.assume_role()
                        .role_arn(arn)
                        .role_session_name("pangolin-legacy-session")
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
                        
                     let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
                     Ok(Credentials::Aws {
                         access_key_id: c.access_key_id,
                         secret_access_key: c.secret_access_key,
                         session_token: Some(c.session_token),
                         expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                     })
                } else {
                     let resp = client.get_session_token()
                        .send()
                        .await
                        .map_err(|e| anyhow::anyhow!("STS GetSessionToken failed: {}", e))?;
                        
                     let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in GetSessionToken response"))?;
                     Ok(Credentials::Aws {
                         access_key_id: c.access_key_id,
                         secret_access_key: c.secret_access_key,
                         session_token: Some(c.session_token),
                         expiration: chrono::DateTime::from_timestamp(c.expiration.secs(), c.expiration.subsec_nanos()),
                     })
                }
            } else {
                Ok(Credentials::Aws {
                    access_key_id: access_key.clone(),
                    secret_access_key: secret_key.clone(),
                    session_token: None,
                    expiration: None,
                })
            }
        }
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("PostgresStore does not support presigning yet"))
    }


}




// Helper methods for PostgresStore (private implementation)
impl PostgresStore {
    async fn get_warehouse_for_location(&self, location: &str) -> Result<Option<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, tenant_id, storage_config, use_sts, vending_strategy FROM warehouses")
            .fetch_all(&self.pool)
            .await?;

        tracing::info!("DEBUG_POSTGRES: get_warehouse_for_location checking {} warehouses for location: {}", rows.len(), location);

        for row in rows {
            let config: serde_json::Value = row.try_get("storage_config")?;
            tracing::info!("DEBUG_POSTGRES: Checking warehouse ID: {:?}, config: {:?}", row.try_get::<Uuid, _>("id"), config);
            
            if let Some(config_map) = config.as_object() {
                let bucket_from_loc = if let Some(rest) = location.strip_prefix("s3://") {
                    rest.split_once('/').map(|(b, _)| b).unwrap_or(rest)
                } else if let Some(rest) = location.strip_prefix("gs://") {
                    rest.split_once('/').map(|(b, _)| b).unwrap_or(rest)
                } else if let Some(rest) = location.strip_prefix("az://") {
                    rest.split_once('/').map(|(b, _)| b).unwrap_or(rest)
                } else {
                    ""
                };

                // Check if location contains the bucket/container name (like MemoryStore/SQLiteStore)
                let s3_match = config_map.get("s3.bucket")
                    .or_else(|| config_map.get("bucket"))
                    .and_then(|v| v.as_str())
                    .map(|b| b == bucket_from_loc || location.contains(b))
                    .unwrap_or(false);
                let azure_match = config_map.get("azure.container")
                    .and_then(|v| v.as_str())
                    .map(|c| c == bucket_from_loc || location.contains(c))
                    .unwrap_or(false);
                let gcp_match = config_map.get("gcp.bucket")
                    .and_then(|v| v.as_str())
                    .map(|b| b == bucket_from_loc || location.contains(b))
                    .unwrap_or(false);

                if s3_match || azure_match || gcp_match {
                    let storage_config: HashMap<String, String> = serde_json::from_value(config)?;
                    tracing::info!("DEBUG_POSTGRES: Match found!");
                    return Ok(Some(Warehouse {
                        id: row.try_get("id")?,
                        tenant_id: row.try_get("tenant_id")?,
                        name: row.try_get("name")?,
                        use_sts: row.try_get("use_sts")?,
                        storage_config,
                        vending_strategy: row.try_get("vending_strategy")
                            .ok()
                            .and_then(|v: serde_json::Value| serde_json::from_value(v).ok()),
                    }));
                } else {
                     tracing::info!("DEBUG_POSTGRES: No match. s3_match={}, azure_match={}, gcp_match={}", s3_match, azure_match, gcp_match);
                }
            }
        }
        Ok(None)
    }

    fn get_object_store_cache_key(&self, config: &HashMap<String, String>, location: &str) -> String {
        let endpoint = config.get("s3.endpoint").or_else(|| config.get("endpoint")).or_else(|| config.get("azure.endpoint")).or_else(|| config.get("gcp.endpoint")).map(|s| s.as_str()).unwrap_or("");
        let bucket = config.get("s3.bucket").or_else(|| config.get("bucket")).or_else(|| config.get("azure.container")).or_else(|| config.get("gcp.bucket")).map(|s| s.as_str()).unwrap_or_else(|| {
            location.strip_prefix("s3://").or_else(|| location.strip_prefix("az://")).or_else(|| location.strip_prefix("gs://")).and_then(|s| s.split('/').next()).unwrap_or("")
        });
        let access_key = config.get("s3.access-key-id").or_else(|| config.get("access_key_id")).or_else(|| config.get("azure.account-name")).or_else(|| config.get("gcp.service-account-key")).map(|s| s.as_str()).unwrap_or("");
        let region = config.get("s3.region").or_else(|| config.get("region")).or_else(|| config.get("azure.region")).or_else(|| config.get("gcp.region")).map(|s| s.as_str()).unwrap_or("");
        crate::ObjectStoreCache::cache_key(endpoint, &bucket, access_key, region)
    }

    // Helper method for reading files without cache (used by read_file with metadata cache)
    async fn read_file_uncached(&self, path: &str) -> Result<Vec<u8>> {
        // Try to look up warehouse credentials first
        if let Some(warehouse) = self.get_warehouse_for_location(path).await? {
            if path.starts_with("s3://") || path.starts_with("az://") || path.starts_with("gs://") {
                // Use cached object store
                let cache_key = self.get_object_store_cache_key(&warehouse.storage_config, path);
                let store = self.object_store_cache.try_get_or_insert::<_, anyhow::Error>(cache_key, || {
                    let os: Box<dyn object_store::ObjectStore> = crate::object_store_factory::create_object_store(&warehouse.storage_config, path)
                        .map_err(|e| anyhow::anyhow!("Failed to create object store for warehouse: {}", e))?;
                    let arc_os: Arc<dyn object_store::ObjectStore> = Arc::from(os);
                    Ok(arc_os)
                })?;
                // Extract key relative to bucket
                let key = if let Some(rest) = path.strip_prefix("s3://").or_else(|| path.strip_prefix("az://")).or_else(|| path.strip_prefix("gs://")) {
                     rest.split_once('/').map(|(_, k)| k).unwrap_or(rest)
                } else {
                     path
                };
                
                match store.get(&object_store::path::Path::from(key)).await {
                    Ok(result) => return Ok(result.bytes().await?.to_vec()),
                    Err(e) => {
                         tracing::warn!("Failed to read from warehouse-configured store for {}, falling back to global env: {}", path, e);
                    }
                }
            }
        }

        // Fallback to existing logic (Global Env Vars)
        if let Some(rest) = path.strip_prefix("s3://") {
            let (bucket, key) = rest.split_once('/').ok_or_else(|| anyhow::anyhow!("Invalid S3 path"))?;
            
            let mut builder = AmazonS3Builder::new()
                .with_bucket_name(bucket)
                .with_allow_http(true);
                
             if let Ok(endpoint) = std::env::var("S3_ENDPOINT") {
                 builder = builder.with_endpoint(endpoint);
             }
             if let Ok(key_id) = std::env::var("AWS_ACCESS_KEY_ID") {
                 builder = builder.with_access_key_id(key_id);
             }
             if let Ok(secret) = std::env::var("AWS_SECRET_ACCESS_KEY") {
                 builder = builder.with_secret_access_key(secret);
             }
             if let Ok(region) = std::env::var("AWS_REGION") {
                 builder = builder.with_region(region);
             }
             
             let store = builder.build()?;
             let result = store.get(&ObjPath::from(key)).await?;
             let bytes = result.bytes().await?;
             Ok(bytes.to_vec())
        } else {
             Err(anyhow::anyhow!("Only s3:// paths are supported in Postgres store"))
        }
    }
}
