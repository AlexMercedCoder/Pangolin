use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc};
use pangolin_core::user::{User, OAuthProvider};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_user_internal(&self, user: User) -> Result<()> {
        self.users().insert_one(user).await?;
        Ok(())
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let filter = doc! { "username": username };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        let filter = doc! { "email": email };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    pub async fn get_user_by_oauth(&self, provider: OAuthProvider, external_id: &str) -> Result<Option<User>> {
        let filter = doc! { 
            "oauth-provider": format!("{:?}", provider), 
            "oauth-external-id": external_id 
        };
        let user = self.users().find_one(filter).await?;
        Ok(user)
    }

    pub async fn list_users(&self, tenant_id: Option<Uuid>, pagination: Option<crate::PaginationParams>) -> Result<Vec<User>> {
        let filter = if let Some(tid) = tenant_id {
            doc! { "tenant-id": to_bson_uuid(tid) }
        } else {
            doc! {}
        };
        
        let collection = self.users();
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
        let users: Vec<User> = cursor.try_collect().await?;
        Ok(users)
    }

    pub async fn update_user(&self, user: User) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(user.id) };
        self.users().replace_one(filter, user).await?;
        Ok(())
    }

    pub async fn update_user_fields(&self, id: Uuid, username: Option<String>, email: Option<String>, active: Option<bool>) -> Result<User> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let mut update_doc = doc! {};
        
        if let Some(uname) = username {
            update_doc.insert("username", uname);
        }
        if let Some(em) = email {
            update_doc.insert("email", em);
        }
        if let Some(act) = active {
            update_doc.insert("active", act);
        }
        
        if update_doc.is_empty() {
            return self.get_user(id).await?
                .ok_or_else(|| anyhow::anyhow!("User not found"));
        }
        
        let update = doc! { "$set": update_doc };
        self.users().update_one(filter, update).await?;
        
        self.get_user(id).await?
            .ok_or_else(|| anyhow::anyhow!("User not found"))
    }

    pub async fn delete_user(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let result = self.users().delete_one(filter).await?;
        if result.deleted_count == 0 {
            return Err(anyhow::anyhow!("User not found"));
        }
        Ok(())
    }
}
