use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::model::Tag;
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_tag(&self, tenant_id: Uuid, catalog_name: &str, tag: Tag) -> Result<()> {
        let doc = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": &tag.name,
            "commit_id": to_bson_uuid(tag.commit_id)
        };
        self.db.collection::<Document>("tags").insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<Option<Tag>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        let doc = self.db.collection::<Tag>("tags").find_one(filter).await?;
        Ok(doc)
    }

    pub async fn list_tags(&self, tenant_id: Uuid, catalog_name: &str, pagination: Option<crate::PaginationParams>) -> Result<Vec<Tag>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name
        };
        
        let collection = self.db.collection::<Tag>("tags");
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
        let tags: Vec<Tag> = cursor.try_collect().await?;
        Ok(tags)
    }

    pub async fn delete_tag(&self, tenant_id: Uuid, catalog_name: &str, name: String) -> Result<()> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "catalog_name": catalog_name,
            "name": name
        };
        self.db.collection::<Document>("tags").delete_one(filter).await?;
        Ok(())
    }
}
