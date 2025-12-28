use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::user::{ServiceUser, UserRole};
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_service_user(&self, service_user: ServiceUser) -> Result<()> {
        let doc = mongodb::bson::to_document(&service_user)?;
        self.service_users().insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_service_user(&self, id: Uuid) -> Result<Option<ServiceUser>> {
        let filter = doc! { "_id": id.to_string() };
        if let Some(doc) = self.service_users().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    pub async fn get_service_user_by_api_key_hash(&self, api_key_hash: &str) -> Result<Option<ServiceUser>> {
        let filter = doc! { "api_key_hash": api_key_hash };
        if let Some(doc) = self.service_users().find_one(filter).await? {
            Ok(Some(mongodb::bson::from_document(doc)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_service_users(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<ServiceUser>> {
        let filter = doc! { "tenant_id": to_bson_uuid(tenant_id) };
        
        let collection = self.service_users();
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
        let docs: Vec<Document> = cursor.try_collect().await?;
        let mut users = Vec::new();
        for doc in docs {
            users.push(mongodb::bson::from_document(doc)?);
        }
        Ok(users)
    }

    pub async fn update_service_user(&self, id: Uuid, name: Option<String>, description: Option<String>, role: Option<UserRole>, active: Option<bool>) -> Result<ServiceUser> {
        let filter = doc! { "_id": id.to_string() };
        let mut update_doc = doc! {};
        
        if let Some(n) = name { update_doc.insert("name", n); }
        if let Some(d) = description { update_doc.insert("description", d); }
        if let Some(r) = role { update_doc.insert("role", format!("{:?}", r)); }
        if let Some(a) = active { update_doc.insert("active", a); }
        
        if !update_doc.is_empty() {
            let update = doc! { "$set": update_doc };
            self.service_users().update_one(filter, update).await?;
        }
        
        self.get_service_user(id).await?.ok_or_else(|| anyhow::anyhow!("Service user not found"))
    }

    pub async fn delete_service_user(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "_id": id.to_string() };
        self.service_users().delete_one(filter).await?;
        Ok(())
    }

    pub async fn update_service_user_last_used(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "_id": id.to_string() };
        let update = doc! { "$set": { "last_used_at": chrono::Utc::now() } };
        self.service_users().update_one(filter, update).await?;
        Ok(())
    }
}
