use super::main::{to_bson_uuid, with_binary_uuids};
use super::MongoStore;
use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::bson::{doc, Bson, Document};
use pangolin_core::permission::Permission;
use pangolin_core::user::User;
use uuid::Uuid;

impl MongoStore {
    pub async fn grant_permission(&self, permission: Permission) -> Result<()> {
        // The overrides used *snake_case* names while `Permission` serializes
        // as kebab-case, so this added a second set of fields rather than
        // replacing the first: the document carried both `user-id` (a string)
        // and `user_id` (Binary). Filters matched the Binary copy, but
        // deserializing the record back into `Permission` read the string and
        // failed with "invalid type: string, expected bytes".
        let doc = with_binary_uuids(
            mongodb::bson::to_document(&permission)?,
            &[
                ("id", permission.id),
                ("user-id", permission.user_id),
                ("tenant-id", permission.tenant_id),
                ("granted-by", permission.granted_by),
            ],
        );

        self.db
            .collection::<Document>("permissions")
            .insert_one(doc)
            .await?;
        Ok(())
    }

    pub async fn revoke_permission(&self, id: Uuid) -> Result<()> {
        let filter = doc! { "id": to_bson_uuid(id) };
        self.db
            .collection::<Document>("permissions")
            .delete_one(filter)
            .await?;
        Ok(())
    }

    pub async fn list_user_permissions(
        &self,
        user_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Permission>> {
        // 1. Fetch direct permissions
        let filter = doc! { "user-id": to_bson_uuid(user_id) };
        let cursor = self
            .db
            .collection::<Permission>("permissions")
            .find(filter)
            .await?;
        let mut perms: Vec<Permission> = cursor.try_collect().await?;

        // 2. Fetch role-based permissions
        let user_roles = self.get_user_roles(user_id).await?;
        for ur in user_roles {
            if let Some(role) = self.get_role(ur.role_id).await? {
                // Determine tenant_id from role
                let role_tenant_id = role.tenant_id;

                for grant in role.permissions {
                    perms.push(Permission {
                        id: Uuid::new_v4(), // Synthesized ID
                        user_id,
                        tenant_id: role_tenant_id,
                        scope: grant.scope,
                        actions: grant.actions,
                        granted_by: role.created_by,
                        granted_at: role.created_at,
                    });
                }
            }
        }

        // 3. In-memory Pagination
        if let Some(p) = pagination {
            let limit = p.limit.unwrap_or(usize::MAX);
            let offset = p.offset.unwrap_or(0);

            if offset >= perms.len() {
                return Ok(vec![]);
            }

            let end = std::cmp::min(offset + limit, perms.len());
            Ok(perms[offset..end].to_vec())
        } else {
            Ok(perms)
        }
    }

    pub async fn list_permissions(
        &self,
        tenant_id: Uuid,
        pagination: Option<crate::PaginationParams>,
    ) -> Result<Vec<Permission>> {
        // 1. Get all user IDs for the tenant
        let user_filter = doc! { "tenant-id": to_bson_uuid(tenant_id) };
        let user_cursor = self.users().find(user_filter).await?;
        let users: Vec<User> = user_cursor.try_collect().await?;
        let user_ids: Vec<Bson> = users.iter().map(|u| to_bson_uuid(u.id)).collect();

        if user_ids.is_empty() {
            return Ok(vec![]);
        }

        // 2. Get permissions for those users
        let perm_filter = doc! { "user-id": { "$in": user_ids } };

        let collection = self.db.collection::<Permission>("permissions");
        let mut find = collection.find(perm_filter);
        if let Some(p) = pagination {
            if let Some(l) = p.limit {
                find = find.limit(l as i64);
            }
            if let Some(o) = p.offset {
                find = find.skip(o as u64);
            }
        }

        let perm_cursor = find.await?;
        let perms: Vec<Permission> = perm_cursor.try_collect().await?;
        Ok(perms)
    }
}
