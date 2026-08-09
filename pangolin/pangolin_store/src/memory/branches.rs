use super::MemoryStore;
use anyhow::Result;
use async_trait::async_trait;
use pangolin_core::model::*;
use uuid::Uuid;

impl MemoryStore {
    pub(crate) async fn search_branches_internal(
        &self,
        tenant_id: Uuid,
        query: &str,
    ) -> Result<Vec<(Branch, String)>> {
        let query = query.to_lowercase();
        let results = self
            .branches
            .iter()
            .filter(|entry| {
                entry.key().0 == tenant_id && entry.value().name.to_lowercase().contains(&query)
            })
            .map(|entry| {
                let (_, catalog_name, _) = entry.key();
                (entry.value().clone(), catalog_name.clone())
            })
            .collect();
        Ok(results)
    }

    pub(crate) async fn create_branch_internal(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        branch: Branch,
    ) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), branch.name.clone());
        self.branches.insert(key, branch);
        Ok(())
    }
    pub(crate) async fn get_branch_internal(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        name: String,
    ) -> Result<Option<Branch>> {
        let key = (tenant_id, catalog_name.to_string(), name);
        if let Some(b) = self.branches.get(&key) {
            Ok(Some(b.value().clone()))
        } else {
            Ok(None)
        }
    }
    pub(crate) async fn list_branches_internal(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Branch>> {
        let iter = self
            .branches
            .iter()
            .filter(|entry| {
                let (tid, cat, _) = entry.key();
                *tid == tenant_id && cat == catalog_name
            })
            .map(|entry| entry.value().clone());

        let branches: Vec<Branch> = if let Some(p) = pagination {
            iter.skip(p.offset.unwrap_or(0))
                .take(p.limit.unwrap_or(usize::MAX))
                .collect()
        } else {
            iter.collect()
        };
        Ok(branches)
    }
    pub(crate) async fn delete_branch_internal(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        name: String,
    ) -> Result<()> {
        let key = (tenant_id, catalog_name.to_string(), name.clone());
        if self.branches.remove(&key).is_some() {
            // Also remove assets associated with this branch
            self.assets
                .retain(|k, _| !(k.0 == tenant_id && k.1 == catalog_name && k.2 == name));
            Ok(())
        } else {
            Err(anyhow::anyhow!("Branch '{}' not found", name))
        }
    }
    /// Merge every asset on `source_branch_name` into `target_branch_name`.
    ///
    /// This used to iterate `source_branch.assets`, a list only populated when
    /// the branch was created, so an asset created *on* the branch afterwards
    /// was never merged. Assets are now enumerated from the store itself, which
    /// is the actual state of the branch. The tracked list is kept in sync as a
    /// by-product.
    pub(crate) async fn merge_branch_internal(
        &self,
        tenant_id: Uuid,
        catalog_name: &str,
        source_branch_name: String,
        target_branch_name: String,
    ) -> Result<()> {
        self.get_branch_internal(tenant_id, catalog_name, source_branch_name.clone())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Source branch '{}' not found", source_branch_name))?;

        let mut target_branch = self
            .get_branch_internal(tenant_id, catalog_name, target_branch_name.clone())
            .await?
            .ok_or_else(|| anyhow::anyhow!("Target branch '{}' not found", target_branch_name))?;

        // Snapshot the source branch's assets before writing, so we are not
        // iterating the map while mutating it.
        let source_assets: Vec<(String, Asset)> = self
            .assets
            .iter()
            .filter(|entry| {
                let (tid, cat, branch, _ns, _name) = entry.key();
                *tid == tenant_id && cat == catalog_name && *branch == source_branch_name
            })
            .map(|entry| (entry.key().3.clone(), entry.value().clone()))
            .collect();

        for (namespace_key, asset) in source_assets {
            let namespace_parts: Vec<String> =
                namespace_key.split('\x1F').map(|s| s.to_string()).collect();

            self.create_asset_internal(
                tenant_id,
                catalog_name,
                Some(target_branch_name.clone()),
                namespace_parts.clone(),
                asset.clone(),
            )
            .await?;

            let qualified = format!("{}.{}", namespace_parts.join("."), asset.name);
            if !target_branch.assets.contains(&qualified) {
                target_branch.assets.push(qualified);
            }
        }

        self.create_branch_internal(tenant_id, catalog_name, target_branch)
            .await?;

        Ok(())
    }
}
