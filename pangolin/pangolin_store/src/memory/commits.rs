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
}
