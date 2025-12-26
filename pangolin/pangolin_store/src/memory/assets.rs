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
    pub(crate) async fn list_assets_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Option<String>, namespace: Vec<String>) -> Result<Vec<Asset>> {
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
}
