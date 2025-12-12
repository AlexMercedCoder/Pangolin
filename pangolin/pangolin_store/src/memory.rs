use crate::CatalogStore;
use async_trait::async_trait;
use dashmap::DashMap;
use pangolin_core::model::{Asset, Branch, Commit, Namespace, Tenant, Catalog, Warehouse};
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryStore {
    tenants: Arc<DashMap<Uuid, Tenant>>,
    warehouses: Arc<DashMap<(Uuid, String), Warehouse>>, // Key: (TenantId, WarehouseName)
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
            warehouses: Arc::new(DashMap::new()),
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

    async fn list_tenants(&self) -> Result<Vec<Tenant>> {
        let tenants = self.tenants.iter().map(|t| t.value().clone()).collect();
        Ok(tenants)
    }

    async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        let key = (tenant_id, warehouse.name.clone());
        self.warehouses.insert(key, warehouse);
        Ok(())
    }

    async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let key = (tenant_id, name);
        if let Some(w) = self.warehouses.get(&key) {
            Ok(Some(w.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let warehouses = self.warehouses.iter()
            .filter(|r| r.key().0 == tenant_id)
            .map(|r| r.value().clone())
            .collect();
        Ok(warehouses)
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
        let ns_str = namespace.join("\x1F");
        let key = (tenant_id, catalog_name.to_string(), ns_str);
        self.namespaces.remove(&key);
        Ok(())
    }

    async fn update_namespace_properties(&self, tenant_id: Uuid, catalog_name: &str, namespace: Vec<String>, properties: std::collections::HashMap<String, String>) -> Result<()> {
        let ns_str = namespace.join("\x1F");
        let key = (tenant_id, catalog_name.to_string(), ns_str);
        
        if let Some(mut ns) = self.namespaces.get_mut(&key) {
            ns.properties.extend(properties);
            Ok(())
        } else {
             Err(anyhow::anyhow!("Namespace not found"))
        }
    }

    async fn create_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), asset.name.clone());
        self.assets.insert(key, asset);
        Ok(())
    }

    async fn get_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), name);
        if let Some(a) = self.assets.get(&key) {
            Ok(Some(a.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_assets(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let ns_str = namespace.join("\x1F");
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
        let ns_str = namespace.join("\x1F");
        let key = (tenant_id, catalog_name.to_string(), branch_name, ns_str, name);
        self.assets.remove(&key);
        Ok(())
    }

    async fn rename_asset(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let src_ns_str = source_namespace.join("\x1F");
        let src_key = (tenant_id, catalog_name.to_string(), branch_name.clone(), src_ns_str, source_name);

        if let Some((_, mut asset)) = self.assets.remove(&src_key) {
            asset.name = dest_name.clone();
            let dest_ns_str = dest_namespace.join("\x1F");
            let dest_key = (tenant_id, catalog_name.to_string(), branch_name, dest_ns_str, dest_name);
            self.assets.insert(dest_key, asset);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Asset not found"))
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use pangolin_core::model::{Tenant, Warehouse, AssetType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_tenant_operations() {
        let store = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        let tenant = Tenant {
            id: tenant_id,
            name: "test_tenant".to_string(),
            properties: HashMap::new(),
        };

        store.create_tenant(tenant.clone()).await.unwrap();
        let fetched = store.get_tenant(tenant_id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "test_tenant");

        let list = store.list_tenants().await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_warehouse_operations() {
        let store = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        let warehouse = Warehouse {
            id: Uuid::new_v4(),
            name: "main_warehouse".to_string(),
            tenant_id,
            storage_config: HashMap::new(),
        };

        store.create_warehouse(tenant_id, warehouse.clone()).await.unwrap();
        let fetched = store.get_warehouse(tenant_id, "main_warehouse".to_string()).await.unwrap();
        assert!(fetched.is_some());
        
        let list = store.list_warehouses(tenant_id).await.unwrap();
        assert_eq!(list.len(), 1);
    }

    #[tokio::test]
    async fn test_asset_operations() {
        let store = MemoryStore::new();
        let tenant_id = Uuid::new_v4();
        let catalog = "default";
        let namespace = vec!["ns1".to_string()];
        
        let asset = Asset {
            name: "tbl1".to_string(),
            kind: AssetType::IcebergTable,
            location: "s3://loc".to_string(),
            properties: HashMap::new(),
        };

        store.create_asset(tenant_id, catalog, None, namespace.clone(), asset.clone()).await.unwrap();
        
        let fetched = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl1".to_string()).await.unwrap();
        assert!(fetched.is_some());

        // Rename
        store.rename_asset(tenant_id, catalog, None, namespace.clone(), "tbl1".to_string(), namespace.clone(), "tbl2".to_string()).await.unwrap();
        let old = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl1".to_string()).await.unwrap();
        assert!(old.is_none());
        let new = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        assert!(new.is_some());

        // Delete
        store.delete_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        let deleted = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        assert!(deleted.is_none());
    }
}
