//! Business metadata for the Postgres backend.
//!
//! These three methods did not exist. `PostgresStore` inherited the trait's
//! "Operation not supported by this store" defaults, while `search_assets`
//! joined a `business_metadata` table that no migration created - so on
//! Postgres, business metadata could not be written *and* asset search failed
//! with a hard SQL error rather than returning nothing.
//!
//! Found by the cross-backend parity suite on its first run against a live
//! Postgres. The suite asserts tag filtering works identically on all four
//! backends, which is not something a per-backend test can check.

use super::PostgresStore;
use anyhow::Result;
use pangolin_core::business_metadata::BusinessMetadata;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    /// Insert or replace an asset's business metadata.
    ///
    /// Keyed on `asset_id` rather than `id`: an asset has at most one metadata
    /// record, and callers construct a fresh `BusinessMetadata` (with a new
    /// `id`) when updating. Conflicting on `id` would insert a duplicate row
    /// per update and break the `asset_id` unique constraint.
    pub async fn upsert_business_metadata(&self, metadata: BusinessMetadata) -> Result<()> {
        sqlx::query(
            "INSERT INTO business_metadata (
                 id, asset_id, description, tags, properties,
                 discoverable, created_by, created_at, updated_by, updated_at
             ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
             ON CONFLICT (asset_id) DO UPDATE SET
                 description  = EXCLUDED.description,
                 tags         = EXCLUDED.tags,
                 properties   = EXCLUDED.properties,
                 discoverable = EXCLUDED.discoverable,
                 updated_by   = EXCLUDED.updated_by,
                 updated_at   = EXCLUDED.updated_at",
        )
        .bind(metadata.id)
        .bind(metadata.asset_id)
        .bind(&metadata.description)
        .bind(serde_json::to_value(&metadata.tags)?)
        .bind(serde_json::to_value(&metadata.properties)?)
        .bind(metadata.discoverable)
        .bind(metadata.created_by)
        .bind(metadata.created_at)
        .bind(metadata.updated_by)
        .bind(metadata.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_business_metadata(&self, asset_id: Uuid) -> Result<Option<BusinessMetadata>> {
        let row = sqlx::query(
            "SELECT id, asset_id, description, tags, properties,
                    discoverable, created_by, created_at, updated_by, updated_at
             FROM business_metadata
             WHERE asset_id = $1",
        )
        .bind(asset_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(BusinessMetadata {
            id: row.get("id"),
            asset_id: row.get("asset_id"),
            description: row.get("description"),
            tags: serde_json::from_value(row.get("tags")).unwrap_or_default(),
            properties: serde_json::from_value(row.get("properties")).unwrap_or_default(),
            discoverable: row.get("discoverable"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
            updated_by: row.get("updated_by"),
            updated_at: row.get("updated_at"),
        }))
    }

    /// Delete an asset's entire metadata record.
    ///
    /// Deleting something that is not there is not an error: callers use this
    /// to ensure absence.
    pub async fn delete_business_metadata(&self, asset_id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM business_metadata WHERE asset_id = $1")
            .bind(asset_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
