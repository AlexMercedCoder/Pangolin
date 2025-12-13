pub mod memory;
pub mod s3;
pub mod signer;
pub mod postgres;
pub mod mongo;

pub use memory::MemoryStore;
pub use s3::S3Store;
pub use postgres::PostgresStore;
pub use mongo::MongoStore;

use async_trait::async_trait;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tag, Tenant, Catalog, Warehouse};
use uuid::Uuid;
use anyhow::Result;

use crate::signer::Signer;

#[async_trait]
pub trait CatalogStore: Send + Sync + Signer {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()>;
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>>;
    async fn list_tenants(&self) -> Result<Vec<Tenant>>;

    // Warehouse Operations
    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()>;
    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>>;
    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>>;

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()>;
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>>;
    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>>;

    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()>;
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>>;
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>>;
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()>;
    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()>;

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()>;
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>>;
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>>;
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()>;
    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()>;

    // Branch Operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()>;
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>>;
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>>;
    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch: String, target_branch: String) -> Result<()>;

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()>;
    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>>;
    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>>;
    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()>;

    // Commit Operations
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()>;
    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>>;

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>>;
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()>;
    
    // Generic File IO (for metadata files)
    async fn read_file(&self, location: &str) -> Result<Vec<u8>>;
    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()>;

    // Maintenance Operations
    async fn expire_snapshots(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, retention_ms: i64) -> Result<()>;
    async fn remove_orphan_files(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, older_than_ms: i64) -> Result<()>;

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: pangolin_core::audit::AuditLogEntry) -> Result<()>;
    async fn list_audit_events(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::audit::AuditLogEntry>>;

    // User Operations
    async fn create_user(&self, _user: pangolin_core::user::User) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_user(&self, _user_id: Uuid) -> Result<Option<pangolin_core::user::User>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_user_by_username(&self, _username: &str) -> Result<Option<pangolin_core::user::User>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn list_users(&self, _tenant_id: Option<Uuid>) -> Result<Vec<pangolin_core::user::User>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    
    // Role Operations
    async fn create_role(&self, _role: pangolin_core::permission::Role) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_role(&self, _role_id: Uuid) -> Result<Option<pangolin_core::permission::Role>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn list_roles(&self, _tenant_id: Uuid) -> Result<Vec<pangolin_core::permission::Role>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn assign_role(&self, _user_role: pangolin_core::permission::UserRole) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn revoke_role(&self, _user_id: Uuid, _role_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_user_roles(&self, _user_id: Uuid) -> Result<Vec<pangolin_core::permission::UserRole>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }

    async fn delete_role(&self, _role_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn update_role(&self, _role: pangolin_core::permission::Role) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }

    // Direct Permission Operations
    async fn create_permission(&self, _permission: pangolin_core::permission::Permission) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn revoke_permission(&self, _permission_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn list_user_permissions(&self, _user_id: Uuid) -> Result<Vec<pangolin_core::permission::Permission>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }

    // Business Metadata Operations
    async fn upsert_business_metadata(&self, _metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_business_metadata(&self, _asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn delete_business_metadata(&self, _asset_id: Uuid) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }

    // Access Request Operations
    async fn create_access_request(&self, _request: pangolin_core::business_metadata::AccessRequest) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn get_access_request(&self, _id: Uuid) -> Result<Option<pangolin_core::business_metadata::AccessRequest>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn list_access_requests(&self, _tenant_id: Uuid) -> Result<Vec<pangolin_core::business_metadata::AccessRequest>> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
    async fn update_access_request(&self, _request: pangolin_core::business_metadata::AccessRequest) -> Result<()> {
        Err(anyhow::anyhow!("Operation not supported by this store"))
    }
}
