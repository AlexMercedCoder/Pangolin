pub mod memory;
pub mod s3;

pub use memory::MemoryStore;
pub use s3::S3Store;

use async_trait::async_trait;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tag, Tenant, Catalog};
use uuid::Uuid;
use anyhow::Result;

#[async_trait]
pub trait CatalogStore: Send + Sync {
    // Tenant Operations
    async fn create_tenant(&self, tenant: Tenant) -> Result<()>;
    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>>;

    // Catalog Operations
    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()>;
    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>>;
    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>>;

    // Namespace Operations
    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()>;
    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>>;
    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>>;
    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()>;

    // Asset Operations
    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()>;
    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>>;
    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>>;
    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()>;

    // Branch & Commit Operations
    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()>;
    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>>;
    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>>;
    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()>;
    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>>;

    // Metadata IO
    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>>;
    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, location: String) -> Result<()>;
    
    // Generic File IO (for metadata files)
    async fn read_file(&self, location: &str) -> Result<Vec<u8>>;
    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()>;
}
