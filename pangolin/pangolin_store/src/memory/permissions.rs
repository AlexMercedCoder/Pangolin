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
    pub(crate) async fn list_permissions_internal(&self, user_id: Option<Uuid>, _resource_id: Option<Uuid>) -> Result<Vec<Permission>> {
        let mut permissions = Vec::new();
        for entry in self.permissions.iter() {
            let p = entry.value();
            if let Some(uid) = user_id {
                if p.user_id == uid {
                    permissions.push(p.clone());
                }
            } else {
                permissions.push(p.clone());
            }
        }
        Ok(permissions)
    }

    pub(crate) async fn create_permission_internal(&self, permission: pangolin_core::permission::Permission) -> Result<()> {
        self.permissions.insert(permission.id, permission);
        Ok(())
    }
}
