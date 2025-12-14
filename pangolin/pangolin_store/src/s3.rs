use crate::CatalogStore;
use async_trait::async_trait;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tenant, Catalog, Warehouse, Tag};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use object_store::{ObjectStore, path::Path};
use object_store::aws::AmazonS3Builder;
use std::sync::Arc;
use futures::TryStreamExt;
use bytes::Bytes;
use crate::signer::{Signer, Credentials};
use aws_config::BehaviorVersion;
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

#[derive(Clone)]
pub struct S3Store {
    store: Arc<dyn ObjectStore>,
    bucket: String,
    prefix: String,
    s3_client: aws_sdk_s3::Client,
}

impl S3Store {
    pub fn new() -> Result<Self> {
        let bucket = std::env::var("PANGOLIN_S3_BUCKET").unwrap_or_else(|_| "pangolin".to_string());
        let prefix = std::env::var("PANGOLIN_S3_PREFIX").unwrap_or_else(|_| "data".to_string());
        
        let mut builder = AmazonS3Builder::from_env().with_bucket_name(&bucket);
        
        if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
            builder = builder.with_endpoint(endpoint);
        }
        
        if let Ok(allow_http) = std::env::var("AWS_ALLOW_HTTP") {
             if allow_http == "true" {
                builder = builder.with_allow_http(true);
             }
        }

        let store = Arc::new(builder.build()?);

        // Initialize AWS SDK Client
        let config = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut loader = aws_config::defaults(BehaviorVersion::latest());
                if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
                     loader = loader.endpoint_url(endpoint);
                }
                loader.load().await
            })
        });
        
        let s3_client = aws_sdk_s3::Client::new(&config);

        Ok(Self {
            store,
            bucket,
            prefix,
            s3_client,
        })
    }

    // Helper to construct paths
    // Structure: tenants/{tenant_id}/catalogs/{catalog_name}/namespaces/{namespace}/assets/{asset_name}.json
    
    fn tenant_path(&self, tenant_id: Uuid) -> Path {
        Path::from(format!("tenants/{}/tenant.json", tenant_id))
    }

    fn warehouse_path(&self, tenant_id: Uuid, name: &str) -> Path {
        Path::from(format!("tenants/{}/warehouses/{}/warehouse.json", tenant_id, name))
    }

    fn catalog_path(&self, tenant_id: Uuid, name: &str) -> Path {
        Path::from(format!("tenants/{}/catalogs/{}/catalog.json", tenant_id, name))
    }

    fn namespace_path(&self, tenant_id: Uuid, catalog_name: &str, namespace: &[String]) -> Path {
        Path::from(format!("tenants/{}/catalogs/{}/namespaces/{}/namespace.json", tenant_id, catalog_name, namespace.join("/")))
    }

    fn asset_path(&self, tenant_id: Uuid, catalog_name: &str, branch: &str, namespace: &[String], name: &str) -> Path {
        Path::from(format!("tenants/{}/catalogs/{}/branches/{}/namespaces/{}/assets/{}.json", tenant_id, catalog_name, branch, namespace.join("/"), name))
    }

    fn branch_path(&self, tenant_id: Uuid, catalog_name: &str, name: &str) -> Path {
        Path::from(format!("tenants/{}/catalogs/{}/branches/{}/branch.json", tenant_id, catalog_name, name))
    }

    fn commit_path(&self, tenant_id: Uuid, commit_id: Uuid) -> Path {
        Path::from(format!("tenants/{}/commits/{}.json", tenant_id, commit_id))
    }

    fn tag_path(&self, tenant_id: Uuid, catalog_name: &str, name: &str) -> Path {
        Path::from(format!("tenants/{}/catalogs/{}/tags/{}.json", tenant_id, catalog_name, name))
    }
}

#[async_trait]
impl CatalogStore for S3Store {
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        let path = self.tenant_path(tenant.id);
        let data = serde_json::to_vec(&tenant)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>> {
        let path = self.tenant_path(tenant_id);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let tenant: Tenant = serde_json::from_slice(&bytes)?;
                Ok(Some(tenant))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        let path = self.catalog_path(tenant_id, &catalog.name);
        let data = serde_json::to_vec(&catalog)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let path = self.catalog_path(tenant_id, &name);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let catalog: Catalog = serde_json::from_slice(&bytes)?;
                Ok(Some(catalog))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        let prefix = format!("tenants/{}/catalogs/", tenant_id);
        let path = Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        
        let mut catalogs = Vec::new();
        while let Some(meta) = stream.try_next().await? {
            if meta.location.as_ref().ends_with("catalog.json") {
                 let bytes = self.store.get(&meta.location).await?.bytes().await?;
                 let catalog: Catalog = serde_json::from_slice(&bytes)?;
                 catalogs.push(catalog);
            }
        }
        Ok(catalogs)
    }

    async fn delete_catalog(&self, _tenant_id: Uuid, _name: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }

    // Rest of the implementation continues...
    async fn create_namespace(&self, _tenant_id: Uuid, _catalog_name: &str, _namespace: Namespace) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_namespaces(&self, _tenant_id: Uuid, _catalog_name: &str, _parent: Option<String>) -> Result<Vec<Namespace>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_namespace(&self, _tenant_id: Uuid, _catalog_name: &str, _namespace: Vec<String>) -> Result<Option<Namespace>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn delete_namespace(&self, _tenant_id: Uuid, _catalog_name: &str, _namespace: Vec<String>) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn update_namespace_properties(&self, _tenant_id: Uuid, _catalog_name: &str, _namespace: Vec<String>, _properties: std::collections::HashMap<String, String>) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn create_asset(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _asset: Asset) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_asset(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _name: String) -> Result<Option<Asset>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_assets(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>) -> Result<Vec<Asset>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn delete_asset(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _name: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn rename_asset(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _source_namespace: Vec<String>, _source_name: String, _dest_namespace: Vec<String>, _dest_name: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn create_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Branch) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<Option<Branch>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_branches(&self, _tenant_id: Uuid, _catalog_name: &str) -> Result<Vec<Branch>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn merge_branch(&self, _tenant_id: Uuid, _catalog_name: &str, _source_branch: String, _target_branch: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn create_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _tag: Tag) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<Option<Tag>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_tags(&self, _tenant_id: Uuid, _catalog_name: &str) -> Result<Vec<Tag>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn delete_tag(&self, _tenant_id: Uuid, _catalog_name: &str, _name: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn create_commit(&self, _tenant_id: Uuid, _commit: Commit) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_commit(&self, _tenant_id: Uuid, _commit_id: Uuid) -> Result<Option<Commit>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn create_warehouse(&self, _tenant_id: Uuid, _warehouse: Warehouse) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_warehouse(&self, _tenant_id: Uuid, _name: String) -> Result<Option<Warehouse>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn list_warehouses(&self, _tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn get_metadata_location(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String) -> Result<Option<String>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn update_metadata_location(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _expected_location: Option<String>, _new_location: String) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn read_file(&self, _location: &str) -> Result<Vec<u8>> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn write_file(&self, _location: &str, _content: Vec<u8>) -> Result<()> {
        Err(anyhow!("S3Store not fully implemented"))
    }
    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        Ok(())
    }
    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        Ok(())
    }
    async fn log_audit_event(&self, _tenant_id: Uuid, _event: pangolin_core::audit::AuditLogEntry) -> Result<()> {
        Ok(())
    }
    async fn list_audit_events(&self, _tenant_id: Uuid) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> {
        Ok(vec![])
    }
}

#[async_trait]
impl Signer for S3Store {
    async fn get_table_credentials(&self, _location: &str) -> Result<Credentials> {
        Err(anyhow!("Credential vending not supported by S3Store"))
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow!("Presigning not supported by S3Store"))
    }
}
