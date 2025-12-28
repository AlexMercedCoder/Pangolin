use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_asset_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, asset: Asset) -> Result<()> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
        
            // 1. Insert Asset
            let asset_full_name = format!("{}.{}", namespace.join("."), asset.name);
            let key = (tenant_id, catalog_name.to_string(), branch_name.clone(), namespace.join("\x1F"), asset.name.clone());
            self.assets.insert(key, asset.clone());

            // 2. Update optimized lookups
            self.assets_by_id.insert(asset.id, (catalog_name.to_string(), namespace.clone(), Some(branch_name.clone()), asset.name.clone()));

            // 2. Ensure Branch Exists and Update Asset List
            let mut branch_obj = self.get_branch_internal(tenant_id, catalog_name, branch_name.clone()).await?
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
                self.create_branch_internal(tenant_id, catalog_name, branch_obj).await?;
            }

            Ok(())
        }
    pub(crate) async fn get_asset_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<Option<Asset>> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
            let key = (tenant_id, catalog_name.to_string(), branch_name, namespace.join("\x1F"), name);
            if let Some(a) = self.assets.get(&key) {
                Ok(Some(a.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn get_asset_by_id_internal(&self, tenant_id: Uuid, asset_id: Uuid) -> Result<Option<(Asset, String, Vec<String>)>> {
            if let Some(entry) = self.assets_by_id.get(&asset_id) {
                let (catalog_name, namespace, branch, name) = entry.value().clone();
                // Verify tenant ownership (implicit via proper key lookup) purely for safety
                let branch_name = branch.unwrap_or_else(|| "main".to_string());
                let key = (tenant_id, catalog_name.clone(), branch_name, namespace.join("\x1F"), name);
            
                if let Some(asset) = self.assets.get(&key) {
                   return Ok(Some((asset.value().clone(), catalog_name, namespace)));
                }
            }
            Ok(None)
        }
    pub(crate) async fn list_assets_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Asset>> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
            let ns_str = namespace.join("\x1F");
            
            let iter = self.assets.iter()
                .filter(|entry| {
                    let (tid, cat, b_name, ns, _) = entry.key();
                    *tid == tenant_id && cat == catalog_name && *b_name == branch_name && *ns == ns_str
                })
                .map(|entry| entry.value().clone());

            let assets: Vec<Asset> = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(assets)
        }
    pub(crate) async fn delete_asset_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>, name: String) -> Result<()> {
            let branch_name = branch.unwrap_or_else(|| "main".to_string());
            let ns_str = namespace.join("\x1F");
            let key = (tenant_id, catalog_name.to_string(), branch_name, ns_str, name);
            if let Some((_, asset)) = self.assets.remove(&key) {
                self.assets_by_id.remove(&asset.id);
            }
            Ok(())
        }
    pub(crate) async fn rename_asset_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, source_namespace: Vec<String>, source_name: String, dest_namespace: Vec<String>, dest_name: String) -> Result<()> {
            let branch_val = branch.unwrap_or_else(|| "main".to_string());
            let src_ns_str = source_namespace.join("\x1F");
            let src_key = (tenant_id, catalog_name.to_string(), branch_val.clone(), src_ns_str, source_name);
            let dest_key = (tenant_id, catalog_name.to_string(), branch_val.clone(), dest_namespace.join("\x1F"), dest_name.clone());

            if let Some((_, mut asset)) = self.assets.remove(&src_key) {
                asset.name = dest_name;
                // Update index
                self.assets_by_id.insert(asset.id, (catalog_name.to_string(), dest_namespace, Some(branch_val), asset.name.clone()));
            
                self.assets.insert(dest_key, asset);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Asset not found"))
            }
        }
    pub(crate) async fn count_assets_internal(&self, tenant_id: Uuid) -> Result<usize> {
            // Efficient counting for MemoryStore
            let count = self.assets.iter()
                .filter(|entry| entry.key().0 == tenant_id)
                .count();
            Ok(count)
        }

    pub(crate) async fn search_assets_internal(&self, tenant_id: Uuid, query: &str, tags: Option<Vec<String>>) -> Result<Vec<(Asset, Option<pangolin_core::business_metadata::BusinessMetadata>, String, Vec<String>)>> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for entry in self.assets.iter() {
            let (tid, cat, _, ns_str, name) = entry.key();
            // Basic filtering
            if *tid == tenant_id {
                let asset = entry.value().clone();
                let metadata = self.business_metadata.get(&asset.id).map(|m| m.value().clone());
                
                let name_matches = name.to_lowercase().contains(&query_lower);
                
                let description_matches = if let Some(ref meta) = metadata {
                    if let Some(ref desc) = meta.description {
                         desc.to_lowercase().contains(&query_lower)
                    } else {
                         false
                    }
                } else {
                    false
                };

                let tags_match = if let Some(ref search_tags) = tags {
                    if let Some(ref meta) = metadata {
                        search_tags.iter().any(|tag| meta.tags.contains(tag))
                    } else {
                        false
                    }
                } else {
                     true
                };

                if (name_matches || description_matches) && tags_match {
                    let namespace: Vec<String> = ns_str.split('\x1F').map(|s| s.to_string()).collect();
                    results.push((asset, metadata, cat.clone(), namespace));
                }
            }
        }
        Ok(results)
    }

    pub(crate) async fn copy_assets_bulk_internal(
        &self, 
        tenant_id: Uuid, 
        catalog_name: &str, 
        src_branch: &str, 
        dest_branch: &str, 
        namespace: Option<String>
    ) -> Result<usize> {
        let mut assets_to_copy: Vec<(Vec<String>, Asset)> = Vec::new();
        
        // 1. Collect assets to copy
        // We collect first to avoid deadlock or concurrent modification issues if we tried to write while iterating
        for entry in self.assets.iter() {
            let (tid, cat, b_name, ns_str, _) = entry.key();
            
            if *tid == tenant_id && cat == catalog_name && b_name == src_branch {
                // If namespace filter is provided, check it
                if let Some(ref ns_filter) = namespace {
                     if ns_str != ns_filter {
                         continue;
                     }
                }

                let ns: Vec<String> = ns_str.split('\x1F').map(|s| s.to_string()).collect();
                assets_to_copy.push((ns, entry.value().clone()));
            }
        }

        let count = assets_to_copy.len();
        let mut dest_assets_names = Vec::new();

        // 2. Insert new assets
        for (ns, mut asset) in assets_to_copy {
            // Generate new ID for the copy
            asset.id = Uuid::new_v4();
            
            let asset_full_name = format!("{}.{}", ns.join("."), asset.name);
            let key = (tenant_id, catalog_name.to_string(), dest_branch.to_string(), ns.join("\x1F"), asset.name.clone());
            
            self.assets.insert(key, asset.clone());
            
            // Update lookup index
            self.assets_by_id.insert(asset.id, (catalog_name.to_string(), ns.clone(), Some(dest_branch.to_string()), asset.name.clone()));
            
            dest_assets_names.push(asset_full_name);
        }

        // 3. Update destination branch asset list
         let mut branch_obj = self.get_branch_internal(tenant_id, catalog_name, dest_branch.to_string()).await?
            .unwrap_or_else(|| {
                Branch {
                    name: dest_branch.to_string(),
                    head_commit_id: None,
                    branch_type: BranchType::Experimental,
                    assets: vec![],
                }
            });

        for name in dest_assets_names {
            if !branch_obj.assets.contains(&name) {
                branch_obj.assets.push(name);
            }
        }
        self.create_branch_internal(tenant_id, catalog_name, branch_obj).await?;

        Ok(count)
    }
}
