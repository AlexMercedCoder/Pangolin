use super::MongoStore;
use super::main::to_bson_uuid;
use anyhow::Result;
use mongodb::bson::{doc, Document};
use pangolin_core::model::Commit;
use uuid::Uuid;
use futures::stream::TryStreamExt;

impl MongoStore {
    pub async fn create_commit_internal(&self, tenant_id: Uuid, commit: Commit) -> Result<()> {
        let mut doc = mongodb::bson::to_document(&commit)?;
        doc.insert("tenant_id", to_bson_uuid(tenant_id));
        // Ensure ID is stored as Binary
        doc.insert("id", to_bson_uuid(commit.id));
        
        if let Some(pid) = commit.parent_id {
            doc.insert("parent_id", to_bson_uuid(pid));
        }
        
        self.db.collection::<Document>("commits").insert_one(doc).await?;
        Ok(())
    }

    pub async fn get_commit_internal(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<Commit>> {
        let filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id),
            "id": to_bson_uuid(id)
        };
        let doc = self.db.collection::<Commit>("commits").find_one(filter).await?;
        Ok(doc)
    }

    pub async fn list_commits_internal(&self, tenant_id: Uuid, branch: Option<String>) -> Result<Vec<Commit>> {
        let mut filter = doc! {
            "tenant_id": to_bson_uuid(tenant_id)
        };
        
        if let Some(b) = branch {
            filter.insert("metadata.branch", b);
        }
        
        let cursor = self.db.collection::<Commit>("commits").find(filter).await?;
        let commits: Vec<Commit> = cursor.try_collect().await?;
        Ok(commits)
    }

    pub async fn get_commit_ancestry(&self, tenant_id: Uuid, commit_id: Uuid, limit: usize) -> Result<Vec<Commit>> {
        let pipeline = vec![
            doc! {
                "$match": {
                    "tenant_id": to_bson_uuid(tenant_id),
                    "id": to_bson_uuid(commit_id)
                }
            },
            doc! {
                "$graphLookup": {
                    "from": "commits",
                    "startWith": "$parent_id",
                    "connectFromField": "parent_id",
                    "connectToField": "id",
                    "as": "ancestry",
                    "maxDepth": (limit as i64) - 1, // Depth is 0-indexed
                    "restrictSearchWithMatch": { "tenant_id": to_bson_uuid(tenant_id) }
                }
            },
            doc! {
                "$project": {
                    // Include the root commit itself + flattened ancestry
                    "commits": {
                        "$concatArrays": [ [ "$$ROOT" ], "$ancestry" ]
                    }
                }
            },
            doc! { "$unwind": "$commits" },
            doc! { "$replaceRoot": { "newRoot": "$commits" } },
            doc! { "$unset": "ancestry" }, // Clean up
            doc! { "$sort": { "timestamp": -1 } },
            doc! { "$limit": limit as i64 }
        ];

        let cursor = self.db.collection::<Commit>("commits").aggregate(pipeline).await?;
        let commits: Vec<Commit> = cursor.with_type::<Commit>().try_collect().await?;
        
        Ok(commits)
    }
}
