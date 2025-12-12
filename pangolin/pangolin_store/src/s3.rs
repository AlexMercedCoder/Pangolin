use crate::CatalogStore;
use async_trait::async_trait;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tenant, Catalog, Warehouse};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use object_store::{ObjectStore, path::Path};
use object_store::aws::AmazonS3Builder;
use std::sync::Arc;
use futures::TryStreamExt;
use bytes::Bytes;

#[derive(Clone)]
pub struct S3Store {
    store: Arc<dyn ObjectStore>,
    bucket: String,
}

impl S3Store {
    pub fn new() -> Result<Self> {
        let region = std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let bucket = std::env::var("AWS_BUCKET").unwrap_or_else(|_| "pangolin-catalog".to_string());
        let endpoint = std::env::var("AWS_ENDPOINT").ok();
        let allow_http = std::env::var("AWS_ALLOW_HTTP").unwrap_or_else(|_| "false".to_string()) == "true";

        let mut builder = AmazonS3Builder::from_env()
            .with_region(region)
            .with_bucket_name(&bucket);

        if let Some(ep) = endpoint {
            builder = builder.with_endpoint(ep);
        }
        
        if allow_http {
            builder = builder.with_allow_http(true);
        }

        let store = builder.build()?;
        
        Ok(Self {
            store: Arc::new(store),
            bucket,
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

    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        let path = self.namespace_path(tenant_id, catalog_name, &namespace.name);
        let data = serde_json::to_vec(&namespace)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let path = self.namespace_path(tenant_id, catalog_name, &namespace);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let ns: Namespace = serde_json::from_slice(&bytes)?;
                Ok(Some(ns))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>> {
        let prefix = format!("tenants/{}/catalogs/{}/namespaces/", tenant_id, catalog_name);
        let path = Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        
        let mut namespaces = Vec::new();
        while let Some(meta) = stream.try_next().await? {
            if meta.location.as_ref().ends_with("namespace.json") {
                 let bytes = self.store.get(&meta.location).await?.bytes().await?;
                 let ns: Namespace = serde_json::from_slice(&bytes)?;
                 
                 if let Some(p) = &parent {
                     if ns.to_string().starts_with(p) {
                         namespaces.push(ns);
                     }
                 } else {
                     namespaces.push(ns);
                 }
            }
        }
        Ok(namespaces)
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let path = self.namespace_path(tenant_id, catalog_name, &namespace);
        self.store.delete(&path).await?;
        Ok(())
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        if let Some(mut ns) = self.get_namespace(tenant_id, catalog_name, namespace.clone()).await? {
            ns.properties.extend(properties);
            self.create_namespace(tenant_id, catalog_name, ns).await?;
            Ok(())
        } else {
             Err(anyhow!("Namespace not found"))
        }
    }

    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let path = self.asset_path(tenant_id, catalog_name, &branch_name, &namespace, &asset.name);
        let data = serde_json::to_vec(&asset)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let path = self.asset_path(tenant_id, catalog_name, &branch_name, &namespace, &name);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let asset: Asset = serde_json::from_slice(&bytes)?;
                Ok(Some(asset))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let prefix = format!("tenants/{}/catalogs/{}/branches/{}/namespaces/{}/assets/", tenant_id, catalog_name, branch_name, namespace.join("/"));
        let path = Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        
        let mut assets = Vec::new();
        while let Some(meta) = stream.try_next().await? {
            if meta.location.as_ref().ends_with(".json") {
                 let bytes = self.store.get(&meta.location).await?.bytes().await?;
                 let asset: Asset = serde_json::from_slice(&bytes)?;
                 assets.push(asset);
            }
        }
        Ok(assets)
    }

    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let path = self.asset_path(tenant_id, catalog_name, &branch_name, &namespace, &name);
        self.store.delete(&path).await?;
        Ok(())
    }

    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
        // Read source
        if let Some(mut asset) = self.get_asset(tenant_id, catalog_name, Some(branch_name.clone()), source_namespace.clone(), source_name.clone()).await? {
            // Update name
            asset.name = dest_name.clone();
            
            // Write to dest
            self.create_asset(tenant_id, catalog_name, Some(branch_name.clone()), dest_namespace.clone(), asset).await?;
            
            // Delete source
            self.delete_asset(tenant_id, catalog_name, Some(branch_name), source_namespace, source_name).await?;
            Ok(())
        } else {
            Err(anyhow!("Asset not found"))
        }
    }

    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        let path = self.branch_path(tenant_id, catalog_name, &branch.name);
        let data = serde_json::to_vec(&branch)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let path = self.branch_path(tenant_id, catalog_name, &name);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let branch: Branch = serde_json::from_slice(&bytes)?;
                Ok(Some(branch))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let prefix = format!("tenants/{}/catalogs/{}/branches/", tenant_id, catalog_name);
        let path = Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        
        let mut branches = Vec::new();
        while let Some(meta) = stream.try_next().await? {
            if meta.location.as_ref().ends_with("branch.json") {
                 let bytes = self.store.get(&meta.location).await?.bytes().await?;
                 let branch: Branch = serde_json::from_slice(&bytes)?;
                 branches.push(branch);
            }
        }
        Ok(branches)
    }

    async fn create_commit(&self, _tenant_id: Uuid, _commit: Commit) -> Result<()> {
        // TODO: Implement commit storage
        Ok(())
    }


    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        // Listing tenants in S3 would require listing the `tenants/` prefix
        // and reading each `tenant.json`.
        // For MVP, returning empty or TODO.
        Ok(vec![])
    }

    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        let path = self.warehouse_path(tenant_id, &warehouse.name);
        let data = serde_json::to_vec(&warehouse)?;
        self.store.put(&path, data.into()).await?;
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let path = self.warehouse_path(tenant_id, &name);
        match self.store.get(&path).await {
            Ok(result) => {
                let bytes = result.bytes().await?;
                let warehouse: Warehouse = serde_json::from_slice(&bytes)?;
                Ok(Some(warehouse))
            },
            Err(object_store::Error::NotFound { .. }) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let prefix = format!("tenants/{}/warehouses/", tenant_id);
        let path = Path::from(prefix);
        let mut stream = self.store.list(Some(&path));
        
        let mut warehouses = Vec::new();
        while let Some(meta) = stream.try_next().await? {
            if meta.location.as_ref().ends_with("warehouse.json") {
                 let bytes = self.store.get(&meta.location).await?.bytes().await?;
                 let warehouse: Warehouse = serde_json::from_slice(&bytes)?;
                 warehouses.push(warehouse);
            }
        }
        Ok(warehouses)
    }

    async fn get_commit(&self, _tenant_id: Uuid, _commit_id: Uuid) -> Result<Option<Commit>> {
        // TODO: Implement commit retrieval
        Ok(None)
    }

    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        // In S3Store, we also store the pointer in the Asset object (which is a JSON file).
        if let Some(asset) = self.get_asset(tenant_id, catalog_name, branch, namespace, table).await? {
            Ok(asset.properties.get("metadata_location").cloned())
        } else {
            Ok(None)
        }
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, location: String) -> Result<()> {
        // We need to read the asset, update property, and write it back.
        // This is not atomic in S3 without a lock or conditional write.
        // For MVP, we just overwrite.
        if let Some(mut asset) = self.get_asset(tenant_id, catalog_name, branch.clone(), namespace.clone(), table.clone()).await? {
            asset.properties.insert("metadata_location".to_string(), location);
            self.create_asset(tenant_id, catalog_name, branch, namespace, asset).await?;
            Ok(())
        } else {
             Err(anyhow!("Table not found"))
        }
    }

    async fn read_file(&self, location: &str) -> Result<Vec<u8>> {
        // Location is full URI e.g. s3://bucket/path
        // We need to parse it or assume it's relative if we are using the store directly?
        // object_store expects a Path.
        // If location starts with s3://, we should strip it or parse it.
        // For simplicity, let's assume we can convert it to Path.
        // But object_store Path is relative to the bucket root usually.
        
        // Hacky parsing for now:
        let path_str = if location.starts_with("s3://") {
             // s3://bucket/key -> key
             let parts: Vec<&str> = location.splitn(4, '/').collect();
             if parts.len() < 4 {
                 return Err(anyhow!("Invalid S3 URI"));
             }
             parts[3]
        } else {
            location
        };

        let path = Path::from(path_str);
        let result = self.store.get(&path).await?;
        let bytes = result.bytes().await?;
        Ok(bytes.to_vec())
    }

    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()> {
        let path_str = if location.starts_with("s3://") {
             let parts: Vec<&str> = location.splitn(4, '/').collect();
             if parts.len() < 4 {
                 return Err(anyhow!("Invalid S3 URI"));
             }
             parts[3]
        } else {
            location
        };

        let path = Path::from(path_str);
        self.store.put(&path, content.into()).await?;
        Ok(())
    }
}
