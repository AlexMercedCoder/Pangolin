use crate::CatalogStore;
use async_trait::async_trait;
use dashmap::DashMap;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tenant, Catalog};
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryStore {
    tenants: Arc<DashMap<Uuid, Tenant>>,
    catalogs: Arc<DashMap<(Uuid, String), Catalog>>, // Key: (TenantId, CatalogName)
    namespaces: Arc<DashMap<(Uuid, String, String), Namespace>>, // Key: (TenantId, CatalogName, NamespaceString)
    // Key: (TenantId, CatalogName, BranchName, NamespaceString, AssetName)
    assets: Arc<DashMap<(Uuid, String, String, String, String), Asset>>, 
    branches: Arc<DashMap<(Uuid, String, String), Branch>>, // Key: (TenantId, CatalogName, BranchName)
    commits: Arc<DashMap<(Uuid, Uuid), Commit>>, // Key: (TenantId, CommitId)
    files: Arc<DashMap<String, Vec<u8>>>, // Key: Location
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(DashMap::new()),
            catalogs: Arc::new(DashMap::new()),
            namespaces: Arc::new(DashMap::new()),
            assets: Arc::new(DashMap::new()),
            branches: Arc::new(DashMap::new()),
            commits: Arc::new(DashMap::new()),
            files: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl CatalogStore for MemoryStore {
    async fn create_tenant(&self, tenant: Tenant) -> Result<()> {
        self.tenants.insert(tenant.id, tenant);
        Ok(())
    }

    async fn get_tenant(&self, tenant_id: Uuid) -> Result<Option<Tenant>> {
        if let Some(t) = self.tenants.get(&tenant_id) {
            Ok(Some(t.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn create_catalog(&self, tenant_id: Uuid, catalog: Catalog) -> Result<()> {
        let key = (tenant_id, catalog.name.clone());
        self.catalogs.insert(key, catalog);
        Ok(())
    }

    async fn get_catalog(&self, tenant_id: Uuid, name: String) -> Result<Option<Catalog>> {
        let key = (tenant_id, name);
        if let Some(c) = self.catalogs.get(&key) {
            Ok(Some(c.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_catalogs(&self, tenant_id: Uuid) -> Result<Vec<Catalog>> {
        let mut catalogs = Vec::new();
        for entry in self.catalogs.iter() {
            let (tid, _) = entry.key();
            if *tid == tenant_id {
                catalogs.push(entry.value().clone());
            }
        }
        Ok(catalogs)
    }

    async fn create_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Namespace) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), namespace.to_string());
        self.namespaces.insert(key, namespace);
        Ok(())
    }

    async fn list_namespaces(&self, tenant_id: Uuid, catalog_name: &str, parent: Option<String>) -> Result<Vec<Namespace>> {
        let parent_prefix = parent.unwrap_or_default();
        let mut namespaces = Vec::new();
        for entry in self.namespaces.iter() {
            let (tid, cat, ns_str) = entry.key();
            if *tid == tenant_id && cat == catalog_name && (parent_prefix.is_empty() || ns_str.starts_with(&parent_prefix)) {
                namespaces.push(entry.value().clone());
            }
        }
        Ok(namespaces)
    }

    async fn get_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<Option<Namespace>> {
        let key = (tenant_id, catalog_name.to_string(), namespace.join("."));
        if let Some(n) = self.namespaces.get(&key) {
            Ok(Some(n.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete_namespace(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), namespace.join("."));
        self.namespaces.remove(&key);
        Ok(())
    }

    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("."), asset.name.clone());
        self.assets.insert(key, asset);
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("."), name);
        if let Some(a) = self.assets.get(&key) {
            Ok(Some(a.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let ns_str = namespace.join(".");
        let mut assets = Vec::new();
        for entry in self.assets.iter() {
            let (tid, cat, b_name, ns, _) = entry.key();
            if *tid == tenant_id && cat == catalog_name && *b_name == branch_name && *ns == ns_str {
                assets.push(entry.value().clone());
            }
        }
        Ok(assets)
    }

    async fn delete_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("."), name);
        self.assets.remove(&key);
        Ok(())
    }

    async fn create_branch(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), branch.name.clone());
        self.branches.insert(key, branch);
        Ok(())
    }

    async fn get_branch(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
        let key = (tenant_id, catalog_name.to_string(), name);
        if let Some(b) = self.branches.get(&key) {
            Ok(Some(b.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_branches(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Branch>> {
        let mut branches = Vec::new();
        for entry in self.branches.iter() {
            let (tid, cat, _) = entry.key();
            if *tid == tenant_id && cat == catalog_name {
                branches.push(entry.value().clone());
            }
        }
        Ok(branches)
    }

    async fn create_commit(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        let key = (tenant_id, commit.id);
        self.commits.insert(key, commit);
        Ok(())
    }

    async fn get_commit(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>> {
        let key = (tenant_id, commit_id);
        if let Some(c) = self.commits.get(&key) {
            Ok(Some(c.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String) -> Result<Option<String>> {
        // For memory store, we can store metadata location in the Asset properties or a separate map.
        // Let's assume it's in Asset properties for now, key "metadata_location".
        if let Some(asset) = self.get_asset(tenant_id, catalog_name, branch, namespace, table).await? {
            Ok(asset.properties.get("metadata_location").cloned())
        } else {
            Ok(None)
        }
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, location: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name.clone(), namespace.join("."), table.clone());
        
        if let Some(mut asset) = self.assets.get_mut(&key) {
            asset.properties.insert("metadata_location".to_string(), location);
            Ok(())
        } else {
            // Asset doesn't exist? Should probably error or create it?
            // For now, let's assume it exists.
             Err(anyhow::anyhow!("Table not found"))
        }
    }

    async fn read_file(&self, location: &str) -> Result<Vec<u8>> {
        if let Some(data) = self.files.get(location) {
            Ok(data.value().clone())
        } else {
            Err(anyhow::anyhow!("File not found: {}", location))
        }
    }

    async fn write_file(&self, location: &str, content: Vec<u8>) -> Result<()> {
        self.files.insert(location.to_string(), content);
        Ok(())
    }
}
