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
    pub(crate) async fn list_permissions_internal(&self, user_id: Option<Uuid>, _resource_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<Permission>> {
        let iter = self.permissions.iter()
            .filter(|entry| {
                let p = entry.value();
                match user_id {
                    Some(uid) => p.user_id == uid,
                    None => true
                }
            })
            .map(|entry| entry.value().clone());

        let permissions = if let Some(p) = pagination {
            iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
        } else {
            iter.collect()
        };
        Ok(permissions)
    }

    pub(crate) async fn create_permission_internal(&self, permission: pangolin_core::permission::Permission) -> Result<()> {
        self.permissions.insert(permission.id, permission);
        Ok(())
    }
}
