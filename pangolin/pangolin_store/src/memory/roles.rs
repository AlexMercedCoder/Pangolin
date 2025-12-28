use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::permission::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_role_internal(&self, role: pangolin_core::permission::Role) -> Result<()> {
            self.roles.insert(role.id, role);
            Ok(())
        }
    pub(crate) async fn get_role_internal(&self, role_id: Uuid) -> Result<Option<pangolin_core::permission::Role>> {
             Ok(self.roles.get(&role_id).map(|r| r.value().clone()))
        }
    pub(crate) async fn list_roles_internal(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<pangolin_core::permission::Role>> {
            let iter = self.roles.iter()
                .filter(|r| r.value().tenant_id == tenant_id)
                .map(|r| r.value().clone());

            let roles = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(roles)
        }
    pub(crate) async fn update_role_internal(&self, role: Role) -> Result<()> {
            // Just overwrite
            self.roles.insert(role.id, role);
            Ok(())
        }
    pub(crate) async fn delete_role_internal(&self, role_id: Uuid) -> Result<()> {
            self.roles.remove(&role_id);
            Ok(())
        }
    pub(crate) async fn get_user_roles_internal(&self, user_id: Uuid) -> Result<Vec<UserRole>> {
            Ok(self.user_roles.iter()
                .filter(|r| r.key().0 == user_id)
                .map(|r| r.value().clone())
                .collect())
        }
    pub(crate) async fn assign_role_internal(&self, user_role: pangolin_core::permission::UserRole) -> Result<()> {
        let key = (user_role.user_id, user_role.role_id);
        self.user_roles.insert(key, user_role);
        Ok(())
    }

    pub(crate) async fn revoke_role_internal(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let key = (user_id, role_id);
        self.user_roles.remove(&key);
        Ok(())
    }
}
