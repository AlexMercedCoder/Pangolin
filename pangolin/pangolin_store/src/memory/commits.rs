use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::model::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_commit_internal(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
            let key = (tenant_id, commit.id);
            self.commits.insert(key, commit);
            Ok(())
        }
    pub(crate) async fn get_commit_internal(&self, tenant_id: Uuid, commit_id: Uuid) -> Result<Option<Commit>> {
            let key = (tenant_id, commit_id);
            if let Some(c) = self.commits.get(&key) {
                Ok(Some(c.value().clone()))
            } else {
                Ok(None)
            }
        }

    pub(crate) async fn get_commit_ancestry_internal(&self, tenant_id: Uuid, head_commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> {
        let mut ancestry = Vec::new();
        let mut current_id = Some(head_commit_id);
        
        while let Some(id) = current_id {
            if ancestry.len() >= limit {
                break;
            }
            
            if let Some(commit) = self.get_commit_internal(tenant_id, id).await? {
                current_id = commit.parent_id;
                ancestry.push(commit);
            } else {
                break;
            }
        }
        
        Ok(ancestry)
    }
}
