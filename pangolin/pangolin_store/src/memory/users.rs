use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use pangolin_core::user::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn create_user_internal(&self, user: User) -> Result<()> {
            self.users.insert(user.id, user);
            Ok(())
        }
    pub(crate) async fn get_user_internal(&self, user_id: Uuid) -> Result<Option<User>> {
            if let Some(user) = self.users.get(&user_id) {
                Ok(Some(user.value().clone()))
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn get_user_by_username_internal(&self, username: &str) -> Result<Option<User>> {
            // Linear search for now, could add index
            for entry in self.users.iter() {
                if entry.value().username == username {
                    return Ok(Some(entry.value().clone()));
                }
            }
            Ok(None)
        }
    pub(crate) async fn list_users_internal(&self, tenant_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<User>> {
            let iter = self.users.iter()
                .filter(|entry| {
                    match tenant_id {
                        Some(tid) => entry.value().tenant_id == Some(tid),
                        None => true // Root listing or all users
                    }
                })
                .map(|entry| entry.value().clone());

            let users = if let Some(p) = pagination {
                iter.skip(p.offset.unwrap_or(0)).take(p.limit.unwrap_or(usize::MAX)).collect()
            } else {
                iter.collect()
            };
            Ok(users)
        }
    pub(crate) async fn update_user_internal(&self, user: User) -> Result<()> {
            if self.users.contains_key(&user.id) {
                self.users.insert(user.id, user);
                Ok(())
            } else {
                 Err(anyhow::anyhow!("User not found"))
            }
        }
    pub(crate) async fn delete_user_internal(&self, user_id: Uuid) -> Result<()> {
            if self.users.remove(&user_id).is_some() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("User not found"))
            }
        }
}
