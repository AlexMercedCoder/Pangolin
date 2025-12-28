use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::business_metadata::AccessRequest;
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        self.access_requests().insert_one(request).await?;
        Ok(())
    }

    pub async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let filter = doc! { "id": to_bson_uuid(id) };
        let req = self.access_requests().find_one(filter).await?;
        Ok(req)
    }

    pub async fn list_access_requests(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<AccessRequest>> {
        let mut pipeline = vec![
            doc! {
                "$lookup": {
                    "from": "users",
                    "localField": "user-id",
                    "foreignField": "id",
                    "as": "user"
                }
            },
            doc! { "$unwind": "$user" },
            doc! { "$match": { "user.tenant-id": to_bson_uuid(tenant_id) } },
            doc! {
                "$project": {
                    "user": 0
                }
            }
        ];

        if let Some(p) = pagination {
            if let Some(o) = p.offset {
                pipeline.push(doc! { "$skip": o as i64 });
            }
            if let Some(l) = p.limit {
                pipeline.push(doc! { "$limit": l as i64 });
            }
        }

        let cursor = self.access_requests().aggregate(pipeline).await?;
        let docs: Vec<Document> = cursor.try_collect().await?;
        
        let mut reqs = Vec::new();
        for d in docs {
             reqs.push(mongodb::bson::from_document(d)?);
        }
        Ok(reqs)
    }

    pub async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(request.id) };
        self.access_requests().replace_one(filter, request).await?;
        Ok(())
    }
}
