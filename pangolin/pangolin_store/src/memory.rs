use crate::CatalogStore;
use crate::signer::{Signer, Credentials};
use async_trait::async_trait;
use dashmap::DashMap;
use pangolin_core::model::{
    Catalog, CatalogType, Namespace, Warehouse, Asset, Commit, Branch, Tag, BranchType, Tenant
};
use pangolin_core::user::User;
use pangolin_core::permission::{Role, UserRole, Permission};
use pangolin_core::audit::AuditLogEntry;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;
use pangolin_core::business_metadata::{BusinessMetadata, AccessRequest};

use tracing;


#[derive(Clone)]
pub struct MemoryStore {
    tenants: Arc<DashMap<Uuid, Tenant>>,
    warehouses: Arc<DashMap<(Uuid, String), Warehouse>>, // Key: (TenantId, WarehouseName)
    catalogs: Arc<DashMap<(Uuid, String), Catalog>>, // Key: (TenantId, CatalogName)
    namespaces: Arc<DashMap<(Uuid, String, String), Namespace>>, // Key: (TenantId, CatalogName, NamespaceString)
    // Key: (TenantId, CatalogName, BranchName, NamespaceString, AssetName)
    assets: Arc<DashMap<(Uuid, String, String, String, String), Asset>>, 
    branches: Arc<DashMap<(Uuid, String, String), Branch>>, // Key: (TenantId, CatalogName, BranchName)
    tags: Arc<DashMap<(Uuid, String, String), Tag>>, // Key: (TenantId, CatalogName, TagName)
    commits: Arc<DashMap<(Uuid, Uuid), Commit>>, // Key: (TenantId, CommitId)
    files: Arc<DashMap<String, Vec<u8>>>, // Key: Location
    audit_events: Arc<DashMap<Uuid, Vec<AuditLogEntry>>>, // Changed to DashMap for consistency, key tenant_id
    // New fields
    users: Arc<DashMap<Uuid, User>>,
    roles: Arc<DashMap<Uuid, Role>>,
    signer: crate::signer::SignerImpl,
    user_roles: Arc<DashMap<(Uuid, Uuid), UserRole>>,
    permissions: Arc<DashMap<Uuid, Permission>>,
    business_metadata: Arc<DashMap<Uuid, pangolin_core::business_metadata::BusinessMetadata>>,
    access_requests: Arc<DashMap<Uuid, pangolin_core::business_metadata::AccessRequest>>,
    service_users: Arc<DashMap<Uuid, pangolin_core::user::ServiceUser>>,
    merge_operations: Arc<DashMap<Uuid, pangolin_core::model::MergeOperation>>,
    merge_conflicts: Arc<DashMap<Uuid, pangolin_core::model::MergeConflict>>,
    // Optimization: Direct lookup for assets by ID
    // Key: AssetID, Value: (CatalogName, Namespace, Branch, AssetName)
    assets_by_id: Arc<DashMap<Uuid, (String, Vec<String>, Option<String>, String)>>, 
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
            tags: Arc::new(DashMap::new()),
            commits: Arc::new(DashMap::new()),
            files: Arc::new(DashMap::new()),
            audit_events: Arc::new(DashMap::new()),
            users: Arc::new(DashMap::new()),
            roles: Arc::new(DashMap::new()),
            signer: crate::signer::SignerImpl::new("memory_key".to_string()),
            user_roles: Arc::new(DashMap::new()),
            permissions: Arc::new(DashMap::new()),
            business_metadata: Arc::new(DashMap::new()),
            access_requests: Arc::new(DashMap::new()),
            service_users: Arc::new(DashMap::new()),
            merge_operations: Arc::new(DashMap::new()),
            merge_conflicts: Arc::new(DashMap::new()),
            assets_by_id: Arc::new(DashMap::new()),
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

    async fn update_tenant(&self, tenant_id: Uuid, updates: pangolin_core::model::TenantUpdate) -> Result<Tenant> {
        if let Some(mut tenant) = self.tenants.get_mut(&tenant_id) {
            if let Some(name) = updates.name {
                tenant.name = name;
            }
            if let Some(properties) = updates.properties {
                tenant.properties.extend(properties);
            }
            Ok(tenant.clone())
        } else {
            Err(anyhow::anyhow!("Tenant not found"))
        }
    }

    async fn delete_tenant(&self, tenant_id: Uuid) -> Result<()> {
        if self.tenants.remove(&tenant_id).is_some() {
            // TODO: Cascade delete warehouses and catalogs
            Ok(())
        } else {
            Err(anyhow::anyhow!("Tenant not found"))
        }
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

    async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
        let key = (tenant_id, name.clone());
        if let Some(mut warehouse) = self.warehouses.get_mut(&key) {
            if let Some(new_name) = updates.name {
                // If name is changing, we need to remove old key and insert with new key
                let mut w = warehouse.clone();
                w.name = new_name.clone();
                drop(warehouse); // Release the mutable reference
                self.warehouses.remove(&key);
                let new_key = (tenant_id, new_name);
                self.warehouses.insert(new_key, w.clone());
                return Ok(w);
            }
            if let Some(config) = updates.storage_config {
                warehouse.storage_config.extend(config);
            }
            if let Some(use_sts) = updates.use_sts {
                warehouse.use_sts = use_sts;
            }
            Ok(warehouse.clone())
        } else {
            Err(anyhow::anyhow!("Warehouse '{}' not found", name))
        }
    }

    async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let key = (tenant_id, name.clone());
        
        if self.warehouses.remove(&key).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Warehouse '{}' not found", name))
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

    async fn update_catalog(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::CatalogUpdate) -> Result<Catalog> {
        let key = (tenant_id, name.clone());
        if let Some(mut catalog) = self.catalogs.get_mut(&key) {
            if let Some(warehouse_name) = updates.warehouse_name {
                catalog.warehouse_name = Some(warehouse_name);
            }
            if let Some(storage_location) = updates.storage_location {
                catalog.storage_location = Some(storage_location);
            }
            if let Some(properties) = updates.properties {
                catalog.properties.extend(properties);
            }
            Ok(catalog.clone())
        } else {
            Err(anyhow::anyhow!("Catalog '{}' not found", name))
        }
    }

    async fn delete_catalog(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let key = (tenant_id, name.clone());
        if self.catalogs.remove(&key).is_some() {
            // Cascade delete: Remove all associated resources
            // Note: In a real database, this would be handled by foreign keys.
            // In MemoryStore, we must manually iterate and remove.
            
            // Remove Namespaces
            self.namespaces.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
            // Remove Assets
            self.assets.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
            // Remove Branches
            self.branches.retain(|k, _| !(k.0 == tenant_id && k.1 == name));
            
            // Remove Tags
            self.tags.retain(|k, _| !(k.0 == tenant_id && k.1 == name));

            // Clean up assets_by_id index
            // This is expensive O(N) since we have to scan the whole index
            // But deletion is rare.
            self.assets_by_id.retain(|_, v| v.0 != name);

            Ok(())
        } else {
            Err(anyhow::anyhow!("Catalog not found"))
        }
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
        if self.namespaces.remove(&key).is_some() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Namespace not found"))
        }
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
        
        // 1. Insert Asset
        let asset_full_name = format!("{}.{}", namespace.join("."), asset.name);
        let key = (tenant_id, catalog_name.to_string(), branch_name.clone(), namespace.join("\x1F"), asset.name.clone());
        self.assets.insert(key, asset);

        // 2. Ensure Branch Exists and Update Asset List
        let mut branch_obj = self.get_branch(tenant_id, catalog_name, branch_name.clone()).await?
            .unwrap_or_else(|| {
                Branch {
                    name: branch_name.clone(),
                    head_commit_id: None,
                    branch_type: BranchType::Experimental,
                    assets: vec![],
                }
            });

        if !branch_obj.assets.contains(&asset_full_name) {
            branch_obj.assets.push(asset_full_name);
            self.create_branch(tenant_id, catalog_name, branch_obj).await?;
        }

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
        let branch_val = branch.unwrap_or_else(|| "main".to_string());
        let src_ns_str = source_namespace.join("\x1F");
        let src_key = (tenant_id, catalog_name.to_string(), branch_val.clone(), src_ns_str, source_name);
        let dest_key = (tenant_id, catalog_name.to_string(), branch_val.clone(), dest_namespace.join("\x1F"), dest_name.clone());

        if let Some((_, mut asset)) = self.assets.remove(&src_key) {
            asset.name = dest_name;
            // Assuming Asset struct has namespace field to update?
            // Actually Asset struct usually only has name. Namespace is implied by location.
            // But let's check Asset struct definition in core/model.rs if possible.
            // Assuming we just update the name in the Asset object if it stores it.
            
            // Update index
            self.assets_by_id.insert(asset.id, (catalog_name.to_string(), dest_namespace, Some(branch_val), asset.name.clone()));
            
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

    async fn merge_branch(&self, tenant_id: Uuid, catalog_name: &str, source_branch_name: String, target_branch_name: String) -> Result<()> {
        // 1. Get Source Branch
        let source_branch = self.get_branch(tenant_id, catalog_name, source_branch_name.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;

        // 2. Get Target Branch
        let mut target_branch = self.get_branch(tenant_id, catalog_name, target_branch_name.clone()).await?
            .ok_or_else(|| anyhow::anyhow!("Target branch not found"))?;

        // 3. Iterate assets tracked by source branch
        for asset_str in &source_branch.assets {
            let parts: Vec<&str> = asset_str.split('.').collect();
            if parts.len() < 2 { continue; }
            
            let asset_name = parts.last().unwrap().to_string();
            let namespace_parts: Vec<String> = parts[0..parts.len()-1].iter().map(|s| s.to_string()).collect();

            // Get asset from source
            if let Some(asset) = self.get_asset(tenant_id, catalog_name, Some(source_branch_name.clone()), namespace_parts.clone(), asset_name).await? {
                // Write to target
                self.create_asset(tenant_id, catalog_name, Some(target_branch_name.clone()), namespace_parts.clone(), asset).await?;
                
                // Ensure branch exists
                let mut branch = self.get_branch(tenant_id, catalog_name, target_branch_name.clone()).await?
                    .unwrap_or_else(|| {
                        tracing::info!("MemoryStore: Branch {} not found, creating new struct", target_branch_name);
                        Branch {
                        name: target_branch_name.clone(),
                        head_commit_id: None,
                        branch_type: BranchType::Experimental,
                        assets: vec![],
                    }});
                
                let full_asset_name = asset_str.to_string();
                if !branch.assets.contains(&full_asset_name) {
                    tracing::info!("MemoryStore: Adding asset {} to branch {}", full_asset_name, target_branch_name);
                    branch.assets.push(full_asset_name.clone());
                    self.create_branch(tenant_id, catalog_name, branch).await?;
                } else {
                    tracing::info!("MemoryStore: Asset {} already in branch {}", full_asset_name, target_branch_name);
                }
            }
        }

        // 4. Update Target Branch asset list
        // This block is now redundant because assets are added to the target branch within the loop.
        // Keeping it commented out or removing it depends on desired behavior.
        // For now, let's assume the in-loop update is sufficient.
        // for asset_name in source_branch.assets {
        //      if !target_branch.assets.contains(&asset_name) {
        //          target_branch.assets.push(asset_name);
        //      }
        // }
        
        // The target_branch variable might not be fully up-to-date if `create_branch` was called inside the loop.
        // Re-fetch or ensure `create_branch` updates the existing one.
        // Given `create_branch` inserts, it effectively overwrites if key exists.
        // So, the loop's `create_branch` calls would update the branch.
        // This final `create_branch` call might be redundant or intended to ensure the final state.
        // Let's remove the redundant update of target_branch.assets and the final create_branch call
        // if the loop already handles it.
        // Based on the instruction, the new code is inserted *inside* the `if let Some(asset) = ...` block.
        // The original `// 4. Update Target Branch asset list` and `self.create_branch(tenant_id, catalog_name, target_branch).await?;`
        // are still present in the original code. The instruction does not remove them.
        // So, I will keep them as is, even if they might be logically redundant after the change.

        // 4. Update Target Branch asset list
        for asset_name in source_branch.assets {
             if !target_branch.assets.contains(&asset_name) {
                 target_branch.assets.push(asset_name);
             }
        }
        
        self.create_branch(tenant_id, catalog_name, target_branch).await?;

        Ok(())
    }

    // Tag Operations
    async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), tag.name.clone());
        self.tags.insert(key, tag);
        Ok(())
    }

    async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let key = (tenant_id, catalog_name.to_string(), name);
        if let Some(tag) = self.tags.get(&key) {
            Ok(Some(tag.value().clone()))
        } else {
            Ok(None)
        }
    }

    async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();
        for r in self.tags.iter() {
            let (tid, cname, _) = r.key();
            if *tid == tenant_id && cname == catalog_name {
                tags.push(r.value().clone());
            }
        }
        Ok(tags)
    }

    async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), name);
        self.tags.remove(&key);
        Ok(())
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
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), table);
        
        if let Some(asset) = self.assets.get(&key) {
            let loc = asset.properties.get("metadata_location").cloned().unwrap_or(asset.location.clone());
            Ok(Some(loc))
        } else {
            Ok(None)
        }
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        let branch_name = branch.unwrap_or_else(|| "main".to_string());
        let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), table);
        
        if let Some(mut asset) = self.assets.get_mut(&key) {
            let current_loc = asset.properties.get("metadata_location").cloned().unwrap_or(asset.location.clone());
            
            // CAS Check
            if let Some(expected) = expected_location {
                if current_loc != expected {
                    return Err(anyhow::anyhow!("CAS failure: expected {} but found {}", expected, current_loc));
                }
            }
            
            asset.location = new_location.clone();
            asset.properties.insert("metadata_location".to_string(), new_location);
            Ok(())
        } else {
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

    async fn expire_snapshots(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _retention_ms: i64) -> Result<()> {
        tracing::info!("MemoryStore: Expiring snapshots (placeholder)");
        Ok(())
    }

    async fn remove_orphan_files(&self, _tenant_id: Uuid, _catalog_name: &str, _branch: Option<String>, _namespace: Vec<String>, _table: String, _older_than_ms: i64) -> Result<()> {
        tracing::info!("MemoryStore: Removing orphan files (placeholder)");
        Ok(())
    }

    // Audit Operations
    async fn log_audit_event(&self, tenant_id: Uuid, event: pangolin_core::audit::AuditLogEntry) -> Result<()> {
        // Log to tracing
        tracing::info!("AUDIT: {:?}", event);
        // Store in map
        self.audit_events.entry(tenant_id).or_insert_with(Vec::new).push(event);
        Ok(())
    }

    async fn list_audit_events(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> {
        // Just return all for now or filter by tenant if we keyed by implementation detail
        if let Some(events) = self.audit_events.get(&tenant_id) {
            Ok(events.clone())
        } else {
             Ok(vec![])
        }
    }

    // User Operations
    async fn create_user(&self, user: pangolin_core::user::User) -> Result<()> {
        self.users.insert(user.id, user);
        Ok(())
    }

    async fn get_user(&self, user_id: Uuid) -> Result<Option<pangolin_core::user::User>> {
        Ok(self.users.get(&user_id).map(|u| u.value().clone()))
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<pangolin_core::user::User>> {
        Ok(self.users.iter()
            .find(|u| u.value().username == username)
            .map(|u| u.value().clone()))
    }

    async fn list_users(&self, tenant_id: Option<Uuid>) -> Result<Vec<pangolin_core::user::User>> {
        Ok(self.users.iter()
            .filter(|u| tenant_id.is_none() || u.value().tenant_id == tenant_id)
            .map(|u| u.value().clone())
            .collect())
    }

    // Role Operations
    async fn create_role(&self, role: pangolin_core::permission::Role) -> Result<()> {
        self.roles.insert(role.id, role);
        Ok(())
    }

    async fn get_role(&self, role_id: Uuid) -> Result<Option<pangolin_core::permission::Role>> {
         Ok(self.roles.get(&role_id).map(|r| r.value().clone()))
    }

    async fn list_roles(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::permission::Role>> {
        Ok(self.roles.iter()
            .filter(|r| r.value().tenant_id == tenant_id)
            .map(|r| r.value().clone())
            .collect())
    }

    async fn assign_role(&self, user_role: UserRole) -> Result<()> {
        let key = (user_role.user_id, user_role.role_id);
        self.user_roles.insert(key, user_role);
        Ok(())
    }

    async fn revoke_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let key = (user_id, role_id);
        self.user_roles.remove(&key);
        Ok(())
    }

    async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRole>> {
        Ok(self.user_roles.iter()
            .filter(|r| r.key().0 == user_id)
            .map(|r| r.value().clone())
            .collect())
    }

    async fn delete_role(&self, role_id: Uuid) -> Result<()> {
        self.roles.remove(&role_id);
        Ok(())
    }

    // New methods for User operations
    async fn update_user(&self, user: pangolin_core::user::User) -> Result<()> {
        if self.users.contains_key(&user.id) {
            self.users.insert(user.id, user);
            Ok(())
        } else {
            Err(anyhow::anyhow!("User not found"))
        }
    }

    async fn delete_user(&self, user_id: Uuid) -> Result<()> {
        self.users.remove(&user_id);
        Ok(())
    }

    async fn update_role(&self, role: Role) -> Result<()> {
        // Just overwrite
        self.roles.insert(role.id, role);
        Ok(())
    }

    async fn create_permission(&self, permission: Permission) -> Result<()> {
        self.permissions.insert(permission.id, permission);
        Ok(())
    }

    async fn revoke_permission(&self, permission_id: Uuid) -> Result<()> {
        self.permissions.remove(&permission_id);
        Ok(())
    }

    async fn list_user_permissions(&self, user_id: Uuid) -> Result<Vec<Permission>> {
        Ok(self.permissions.iter()
            .filter(|p| p.value().user_id == user_id)
            .map(|p| p.value().clone())
            .collect())
    }

    async fn upsert_business_metadata(&self, metadata: pangolin_core::business_metadata::BusinessMetadata) -> Result<()> {
        self.business_metadata.insert(metadata.asset_id, metadata);
        Ok(())
    }

    async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<pangolin_core::business_metadata::BusinessMetadata>> {
        Ok(self.business_metadata.get(&asset_id).map(|m| m.value().clone()))
    }

    async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        self.business_metadata.remove(&asset_id);
        Ok(())
    }

    async fn search_assets(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>)>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        // Iterate through all assets for this tenant
        for entry in self.assets.iter() {
            let key = entry.key();
            if key.0 != tenant_id {
                continue;
            }

            let asset = entry.value().clone();
            let metadata = self.business_metadata.get(&asset.id).map(|m| m.value().clone());

            // Check if asset matches search criteria
            let name_matches = asset.name.to_lowercase().contains(&query_lower);
            
            let tags_match = if let Some(ref search_tags) = tags {
                if let Some(ref meta) = metadata {
                    search_tags.iter().any(|tag| meta.tags.contains(tag))
                } else {
                    false
                }
            } else {
                true // No tag filter
            };

            if name_matches && tags_match {
                results.push((asset, metadata));
            }
        }

        Ok(results)
    }

    // Access Request Operations
    async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        self.access_requests.insert(request.id, request);
        Ok(())
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        Ok(self.access_requests.get(&id).map(|r| r.value().clone()))
    }

    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> {
        let mut requests = Vec::new();
        // Efficient scan filtering by tenant_id directly
        for req in self.access_requests.iter() {
            if req.value().tenant_id == tenant_id {
                requests.push(req.value().clone());
            }
        }
        Ok(requests)
    }

    async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        self.access_requests.insert(request.id, request);
        Ok(())
    }

    // Service User Operations
    async fn create_service_user(&self, service_user: pangolin_core::user::ServiceUser) -> Result<()> {
        self.service_users.insert(service_user.id, service_user);
        Ok(())
    }

    async fn get_service_user(&self, id: Uuid) -> Result<Option<pangolin_core::user::ServiceUser>> {
        Ok(self.service_users.get(&id).map(|r| r.value().clone()))
    }

    async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<pangolin_core::user::ServiceUser>> {
        // Linear search through all service users to find matching hash
        for entry in self.service_users.iter() {
            if entry.value().api_key_hash == api_key_hash {
                return Ok(Some(entry.value().clone()));
            }
        }
        Ok(None)
    }

    async fn list_service_users(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::user::ServiceUser>> {
        Ok(self.service_users
            .iter()
            .filter(|entry| entry.value().tenant_id == tenant_id)
            .map(|entry| entry.value().clone())
            .collect())
    }

    async fn update_service_user(
        &self,
        id: Uuid,
        name: Option<String>,
        description: Option<String>,
        active: Option<bool>,
    ) -> Result<()> {
        if let Some(mut service_user) = self.service_users.get_mut(&id) {
            if let Some(n) = name {
                service_user.name = n;
            }
            if let Some(d) = description {
                service_user.description = Some(d);
            }
            if let Some(a) = active {
                service_user.active = a;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service user not found"))
        }
    }

    async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        self.service_users.remove(&id);
        Ok(())
    }

    async fn update_service_user_last_used(&self, id: Uuid, timestamp: chrono::DateTime<chrono::Utc>) -> Result<()> {
        if let Some(mut service_user) = self.service_users.get_mut(&id) {
            service_user.last_used = Some(timestamp);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Service user not found"))
        }
    }

    // Merge Operation Methods
    async fn create_merge_operation(&self, operation: pangolin_core::model::MergeOperation) -> Result<()> {
        self.merge_operations.insert(operation.id, operation);
        Ok(())
    }
    
    async fn get_merge_operation(&self, operation_id: Uuid) -> Result<Option<pangolin_core::model::MergeOperation>> {
        Ok(self.merge_operations.get(&operation_id).map(|r| r.value().clone()))
    }
    
    async fn list_merge_operations(&self, tenant_id: Uuid, catalog_name: &str) -> Result<Vec<pangolin_core::model::MergeOperation>> {
        Ok(self.merge_operations
            .iter()
            .filter(|r| r.value().tenant_id == tenant_id && r.value().catalog_name == catalog_name)
            .map(|r| r.value().clone())
            .collect())
    }
    
    async fn update_merge_operation_status(&self, operation_id: Uuid, status: pangolin_core::model::MergeStatus) -> Result<()> {
        if let Some(mut operation) = self.merge_operations.get_mut(&operation_id) {
            operation.status = status;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }
    
    async fn complete_merge_operation(&self, operation_id: Uuid, result_commit_id: Uuid) -> Result<()> {
        if let Some(mut operation) = self.merge_operations.get_mut(&operation_id) {
            operation.status = pangolin_core::model::MergeStatus::Completed;
            operation.result_commit_id = Some(result_commit_id);
            operation.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }
    
    async fn abort_merge_operation(&self, operation_id: Uuid) -> Result<()> {
        if let Some(mut operation) = self.merge_operations.get_mut(&operation_id) {
            operation.status = pangolin_core::model::MergeStatus::Aborted;
            operation.completed_at = Some(chrono::Utc::now());
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }

    // Merge Conflict Methods
    async fn create_merge_conflict(&self, conflict: pangolin_core::model::MergeConflict) -> Result<()> {
        self.merge_conflicts.insert(conflict.id, conflict);
        Ok(())
    }
    
    async fn get_merge_conflict(&self, conflict_id: Uuid) -> Result<Option<pangolin_core::model::MergeConflict>> {
        Ok(self.merge_conflicts.get(&conflict_id).map(|r| r.value().clone()))
    }
    
    async fn list_merge_conflicts(&self, operation_id: Uuid) -> Result<Vec<pangolin_core::model::MergeConflict>> {
        Ok(self.merge_conflicts
            .iter()
            .filter(|r| r.value().merge_operation_id == operation_id)
            .map(|r| r.value().clone())
            .collect())
    }
    
    async fn resolve_merge_conflict(&self, conflict_id: Uuid, resolution: pangolin_core::model::ConflictResolution) -> Result<()> {
        if let Some(mut conflict) = self.merge_conflicts.get_mut(&conflict_id) {
            conflict.resolution = Some(resolution);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge conflict not found"))
        }
    }
    
    async fn add_conflict_to_operation(&self, operation_id: Uuid, conflict_id: Uuid) -> Result<()> {
        if let Some(mut operation) = self.merge_operations.get_mut(&operation_id) {
            if !operation.conflicts.contains(&conflict_id) {
                operation.conflicts.push(conflict_id);
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Merge operation not found"))
        }
    }
}

#[async_trait]
impl Signer for MemoryStore {
    async fn get_table_credentials(&self, location: &str) -> Result<Credentials> {
        // 1. Find the warehouse that owns this location
        // Iterate over all tenants
        let warehouses_map: Vec<Warehouse> = self.warehouses
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        // Simple prefix match. In real world, we might want more robust matching.
        // Assumes location starts with warehouse storage path if standard layout.
        // OR we check the storage_config "bucket" or "prefix".
        
        let mut target_warehouse = None;
        
        for warehouse in warehouses_map {
            // Check storage config
            if let Some(bucket) = warehouse.storage_config.get("s3.bucket") {
                // Check if location contains bucket. 
                // location format: s3://bucket/key
                if location.contains(bucket) {
                    target_warehouse = Some(warehouse);
                    break;
                }
            }
        }

        let warehouse = target_warehouse.ok_or_else(|| anyhow::anyhow!("No warehouse found for location: {}", location))?;

        // 2. Extract credentials from storage_config
        let access_key = warehouse.storage_config.get("s3.access-key-id")
            .ok_or_else(|| anyhow::anyhow!("Missing s3.access-key-id in warehouse config"))?;
            
        let secret_key = warehouse.storage_config.get("s3.secret-access-key")
            .ok_or_else(|| anyhow::anyhow!("Missing s3.secret-access-key in warehouse config"))?;
            
        // 3. Check STS flag
        if warehouse.use_sts {
            // STS Mode
            // We need to create an STS client using the warehouse's "root" credentials
            // and assume a role (or get a session token).
            let region = warehouse.storage_config.get("s3.region")
                .map(|s| s.as_str())
                .unwrap_or("us-east-1");
                
            let endpoint = warehouse.storage_config.get("s3.endpoint")
                .map(|s| s.as_str());

            let creds = aws_credential_types::Credentials::new(
                access_key.to_string(),
                secret_key.to_string(),
                None,
                None,
                "static"
            );

            let config_loader = aws_config::defaults(aws_config::BehaviorVersion::latest())
                .region(aws_config::Region::new(region.to_string()))
                .credentials_provider(creds);
                
            let config_loader = if let Some(ep) = endpoint {
                 config_loader.endpoint_url(ep)
            } else {
                config_loader
            };
            
            let config = config_loader.load().await;
            let client = aws_sdk_sts::Client::new(&config);
            
            // Assume Role or GetSessionToken
            let role_arn = warehouse.storage_config.get("s3.role-arn").map(|s| s.as_str());
            
            let (temp_ak, temp_sk, temp_token, expiration) = if let Some(arn) = role_arn {
                 let resp = client.assume_role()
                    .role_arn(arn)
                    .role_session_name("pangolin-vended-session")
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("STS AssumeRole failed: {}", e))?;
                    
                 let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in AssumeRole response"))?;
                 (
                     c.access_key_id, 
                     c.secret_access_key, 
                     c.session_token, 
                     Some(c.expiration.to_string())
                 )
            } else {
                 let resp = client.get_session_token()
                    .send()
                    .await
                    .map_err(|e| anyhow::anyhow!("STS GetSessionToken failed: {}", e))?;
                    
                 let c = resp.credentials.ok_or_else(|| anyhow::anyhow!("No credentials in GetSessionToken response"))?;
                 (
                     c.access_key_id, 
                     c.secret_access_key, 
                     c.session_token, 
                     Some(c.expiration.to_string())
                 )
            };
            
            Ok(Credentials {
                access_key_id: temp_ak,
                secret_access_key: temp_sk,
                session_token: Some(temp_token),
                expiration,
            })

        } else {
            // Static Mode
            Ok(Credentials {
                access_key_id: access_key.to_string(),
                secret_access_key: secret_key.to_string(),
                session_token: None,
                expiration: None,
            })
        }
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        // Stub: Presigning requires keeping an ObjectStore client around or rebuilding it.
        // For this task, we focus on table credentials.
        Err(anyhow::anyhow!("MemoryStore does not support presigning yet"))
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
            use_sts: false,
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
            id: Uuid::new_v4(),
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

        store.delete_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        let deleted = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_asset_update_consistency() {
        let store = MemoryStore::new();
        crate::tests::test_asset_update_consistency(&store).await;
    }
}
