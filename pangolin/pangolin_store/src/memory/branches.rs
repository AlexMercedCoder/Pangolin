use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn search_branches_internal(&self, tenant_id: Uuid, query: &str) -> Result<Vec<(Branch, String)>> {
        let query = query.to_lowercase();
        let results = self.branches.iter()
            .filter(|entry| {
                entry.key().0 == tenant_id && 
                entry.value().name.to_lowercase().contains(&query)
            })
            .map(|entry| {
                let (_, catalog_name, _) = entry.key();
                (entry.value().clone(), catalog_name.clone())
            })
            .collect();
        Ok(results)
    }

    pub(crate) async fn create_branch_internal(&self, tenant_id: Uuid, catalog_name: &str, branch: Branch) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), branch.name.clone());
            self.branches.insert(key, branch);
            Ok(())
        }
    pub(crate) async fn get_branch_internal(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Branch>> {
            let key = (tenant_id, catalog_name.to_string(), name);
            if let Some(b) = self.branches.get(&key) {
                Ok(Some(b.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn list_branches_internal(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Branch>> {
            let iter = self.branches.iter()
                .filter(|entry| {
                    let (tid, cat, _) = entry.key();
                    *tid == tenant_id && cat == catalog_name
                })
                .map(|entry| entry.value().clone());

            let branches: Vec<Branch> = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(branches)
        }
    pub(crate) async fn delete_branch_internal(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
            let key = (tenant_id, catalog_name.to_string(), name.clone());
            if self.branches.remove(&key).is_some() {
                // Also remove assets associated with this branch
                self.assets.retain(|k, _| !(k.0 == tenant_id && k.1 == catalog_name && k.2 == name));
                Ok(())
            } else {
                Err(anyhow::anyhow!("Branch '{}' not found", name))
            }
        }
    pub(crate) async fn merge_branch_internal(&self, tenant_id: Uuid, catalog_name: &str, source_branch_name: String, target_branch_name: String) -> Result<()> {
            // 1. Get Source Branch
            let source_branch = self.get_branch_internal(tenant_id, catalog_name, source_branch_name.clone()).await?
                .ok_or_else(|| anyhow::anyhow!("Source branch not found"))?;

            // 2. Get Target Branch
            let mut target_branch = self.get_branch_internal(tenant_id, catalog_name, target_branch_name.clone()).await?
                .ok_or_else(|| anyhow::anyhow!("Target branch not found"))?;

            // 3. Iterate assets tracked by source branch
            for asset_str in &source_branch.assets {
                let parts: Vec<&str> = asset_str.split('.').collect();
                if parts.len() < 2 { continue; }
            
                let asset_name = parts.last().unwrap().to_string();
                let namespace_parts: Vec<String> = parts[0..parts.len()-1].iter().map(|s| s.to_string()).collect();

                // Get asset from source
                if let Some(asset) = self.get_asset_internal(tenant_id, catalog_name, Some(source_branch_name.clone()), namespace_parts.clone(), asset_name.clone()).await? {
                    tracing::info!("MemoryStore: Merging asset {} from {} to {}. Location: {:?}", asset_name, source_branch_name, target_branch_name, asset.properties.get("metadata_location"));
                    // Write to target
                    self.create_asset_internal(tenant_id, catalog_name, Some(target_branch_name.clone()), namespace_parts.clone(), asset).await?;
                    
                    // Verify
                    if let Some(updated) = self.get_asset_internal(tenant_id, catalog_name, Some(target_branch_name.clone()), namespace_parts.clone(), asset_name.clone()).await? {
                         tracing::info!("MemoryStore: VERIFICATION: Asset {} on {} is now at {:?}", asset_name, target_branch_name, updated.properties.get("metadata_location"));
                    }
                
                    // Ensure branch exists
                    let mut branch = self.get_branch_internal(tenant_id, catalog_name, target_branch_name.clone()).await?
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
                        self.create_branch_internal(tenant_id, catalog_name, branch).await?;
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
            // The original `// 4. Update Target Branch asset list` and `self.create_branch_internal(tenant_id, catalog_name, target_branch).await?;`
            // are still present in the original code. The instruction does not remove them.
            // So, I will keep them as is, even if they might be logically redundant after the change.

            // 4. Update Target Branch asset list
            for asset_name in source_branch.assets {
                 if !target_branch.assets.contains(&asset_name) {
                     target_branch.assets.push(asset_name);
                 }
            }
        
            self.create_branch_internal(tenant_id, catalog_name, target_branch).await?;

            Ok(())
        }
}
