use crate::CatalogStore;
use crate::signer::{Signer, Credentials};
use async_trait::async_trait;
use dashmap::DashMap;
use pangolin_core::model::{Asset, Branch, Namespace, BranchType, Tenant, Catalog, Warehouse, Commit, Tag};
use pangolin_core::user::User;
use pangolin_core::permission::{Role, UserRole, Permission};
use pangolin_core::audit::AuditLogEntry;
use uuid::Uuid;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
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
        // For memory store, we can store metadata location in the Asset properties or a separate map.
        // Let's assume it's in Asset properties for now, key "metadata_location".
        if let Some(asset) = self.get_asset(tenant_id, catalog_name, branch, namespace, table).await? {
            Ok(asset.properties.get("metadata_location").cloned())
        } else {
            Ok(None)
        }
    }

    async fn update_metadata_location(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, table: String, expected_location: Option<String>, new_location: String) -> Result<()> {
        // CAS implementation for MemoryStore
        // We need to lock or use atomic operations. DashMap provides some atomicity.
        // We will fetch, check, and update.
        
        // Note: This is not strictly atomic in this implementation because we do get then insert.
        // For a real memory store, we'd need a mutex around the specific asset or use `entry` API carefully.
        // Since we are using DashMap, we can use `entry` but our key structure is complex.
        
        // Simplified CAS:
        let current_asset = self.get_asset(tenant_id, catalog_name, branch.clone(), namespace.clone(), table.clone()).await?;
        
        if let Some(mut asset) = current_asset {
            let current_loc = asset.properties.get("metadata_location").cloned();
            
            if current_loc != expected_location {
                return Err(anyhow::anyhow!("Commit failed: Metadata location mismatch. Expected {:?}, found {:?}", expected_location, current_loc));
            }
            
            asset.properties.insert("metadata_location".to_string(), new_location);
            
            // Update the asset
            // In a real implementation, we'd want to ensure no one else updated it in between.
            // For MemoryStore (dev/test), this race condition is acceptable-ish, or we can improve.
            // Let's just overwrite for now.
            self.create_asset(tenant_id, catalog_name, branch, namespace, asset).await?;
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

    async fn create_access_request(&self, request: pangolin_core::business_metadata::AccessRequest) -> Result<()> {
        self.access_requests.insert(request.id, request);
        Ok(())
    }

    async fn get_access_request(&self, id: Uuid) -> Result<Option<pangolin_core::business_metadata::AccessRequest>> {
        Ok(self.access_requests.get(&id).map(|r| r.value().clone()))
    }

    async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<pangolin_core::business_metadata::AccessRequest>> {
        let mut requests = Vec::new();
        // Inefficient scan to filter by tenant via asset ownership
        // Ideally AccessRequest should have tenant_id or we should look up differently
        for req in self.access_requests.iter() {
            let asset_id = req.value().asset_id;
            let mut belongs_to_tenant = false;
            // Scan assets to find the one this request is for
            for asset_entry in self.assets.iter() {
                if asset_entry.value().id == asset_id {
                    let (tid, _, _, _, _) = asset_entry.key();
                    if *tid == tenant_id {
                        belongs_to_tenant = true;
                    }
                    break;
                }
            }
            
            if belongs_to_tenant {
                requests.push(req.value().clone());
            }
        }
        Ok(requests)
    }

    async fn update_access_request(&self, request: pangolin_core::business_metadata::AccessRequest) -> Result<()> {
        self.access_requests.insert(request.id, request);
        Ok(())
    }
}

#[async_trait]
impl Signer for MemoryStore {
    async fn get_table_credentials(&self, _location: &str) -> Result<Credentials> {
        Err(anyhow::anyhow!("MemoryStore does not support credential vending"))
    }

    async fn presign_get(&self, _location: &str) -> Result<String> {
        Err(anyhow::anyhow!("MemoryStore does not support presigning"))
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

        // Delete
        store.delete_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        let deleted = store.get_asset(tenant_id, catalog, None, namespace.clone(), "tbl2".to_string()).await.unwrap();
        assert!(deleted.is_none());
    }
}
