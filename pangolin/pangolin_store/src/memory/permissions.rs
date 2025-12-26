use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::permission::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn revoke_permission_internal(&self, permission_id: Uuid) -> Result<()> {
            self.permissions.remove(&permission_id);
            Ok(())
        }
    pub(crate) async fn list_permissions_internal(&self, _user_id: Option<Uuid>, _resource_id: Option<Uuid>) -> Result<Vec<Permission>> {
            // MemoryStore simple implementation: just return all for now or implement filtering if needed
            Ok(self.permissions.iter().map(|e| e.value().clone()).collect())
        }

    pub(crate) async fn create_permission_internal(&self, permission: pangolin_core::permission::Permission) -> Result<()> {
        self.permissions.insert(permission.id, permission);
        Ok(())
    }
}
