use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::permission::{Role, UserRole};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_role(&self, role: Role) -> Result<()> {
        self.roles().insert_one(role).await?;
        Ok(())
    }

    pub async fn get_role(&self, id: Uuid) -> Result<Option<Role>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let role = self.roles().find_one(filter).await?;
        Ok(role)
    }

    pub async fn get_role_by_name(&self, tenant_id: Uuid, name: &str) -> Result<Option<Role>> {
        let filter = doc! { 
            "tenant-id": to_bson_uuid(tenant_id),
            "name": name 
        };
        let role = self.roles().find_one(filter).await?;
        Ok(role)
    }

    pub async fn list_roles(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Role>> {
        let filter = doc! { "tenant-id": to_bson_uuid(tenant_id) };
        
        let collection = self.roles();
        let mut find = collection.find(filter);
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                find = find.limit(l as i64);
            }
            if let Some(o) = p.offset {
                find = find.skip(o as u64);
            }
        }

        let cursor = find.await?;
        let roles: Vec<Role> = cursor.try_collect().await?;
        Ok(roles)
    }

    pub async fn delete_role(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let result = self.roles().delete_one(filter).await?;
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("Role not found"));
        }
        Ok(())
    }

    pub async fn update_role(&self, role: Role) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(role.id) };
        self.roles().replace_one(filter, role).await?;
        Ok(())
    }

    // Role Assignment
    pub async fn assign_role(&self, assignment: UserRole) -> Result<()> {
        let doc = mongodb::bson::to_document(&assignment)?;
        self.db.collection::<Document>("user_roles").insert_one(doc).await?;
        Ok(())
    }

    pub async fn remove_role(&self, user_id: Uuid, role_id: Uuid) -> Result<()> {
        let filter = doc! { 
            "user_id": to_bson_uuid(user_id),
            "role_id": to_bson_uuid(role_id)
        };
        self.db.collection::<Document>("user_roles").delete_one(filter).await?;
        Ok(())
    }

    pub async fn get_user_roles(&self, user_id: Uuid) -> Result<Vec<UserRole>> {
        let filter = doc! { "user_id": to_bson_uuid(user_id) };
        let cursor = self.db.collection::<UserRole>("user_roles").find(filter).await?;
        let assignments: Vec<UserRole> = cursor.try_collect().await?;
        Ok(assignments)
    }
}
