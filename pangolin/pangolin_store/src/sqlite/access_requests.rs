/// Access Request operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::{DateTime, Utc, TimeZone};
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};

impl SqliteStore {
    pub async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        sqlx::query(
            "INSERT INTO access_requests (id, tenant_id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(request.id.to_string())
        .bind(request.tenant_id.to_string())
        .bind(request.user_id.to_string())
        .bind(request.asset_id.to_string())
        .bind(request.reason)
        .bind(request.requested_at.timestamp_millis())
        .bind(format!("{:?}", request.status)) // Enum to string
        .bind(request.reviewed_by.map(|u: Uuid| u.to_string()))
        .bind(request.reviewed_at.map(|t: DateTime<Utc>| t.timestamp_millis()))
        .bind(request.review_comment)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let row = sqlx::query("SELECT * FROM access_requests WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_access_request(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_access_requests(&self, tenant_id: Uuid) -> Result<Vec<AccessRequest>> {
        let rows = sqlx::query(
            "SELECT * FROM access_requests WHERE tenant_id = ?"
        )
        .bind(tenant_id.to_string())
        .fetch_all(&self.pool)
        .await?;
        
        let mut requests = Vec::new();
        for row in rows {
            requests.push(self.row_to_access_request(row)?);
        }
        Ok(requests)
    }

    pub async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        sqlx::query(
            "UPDATE access_requests SET status = ?, reviewed_by = ?, reviewed_at = ?, review_comment = ? WHERE id = ?"
        )
        .bind(format!("{:?}", request.status))
        .bind(request.reviewed_by.map(|u: Uuid| u.to_string()))
        .bind(request.reviewed_at.map(|t: DateTime<Utc>| t.timestamp_millis()))
        .bind(request.review_comment)
        .bind(request.id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub fn row_to_access_request(&self, row: sqlx::sqlite::SqliteRow) -> Result<AccessRequest> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Pending" => RequestStatus::Pending,
            "Approved" => RequestStatus::Approved,
            "Rejected" => RequestStatus::Rejected,
            _ => RequestStatus::Pending,
        };

        Ok(AccessRequest {
            id: Uuid::parse_str(&row.get::<String, _>("id"))?,
            tenant_id: Uuid::parse_str(&row.get::<String, _>("tenant_id"))?,
            user_id: Uuid::parse_str(&row.get::<String, _>("user_id"))?,
            asset_id: Uuid::parse_str(&row.get::<String, _>("asset_id"))?,
            reason: row.get("reason"),
            requested_at: Utc.timestamp_millis_opt(row.get("requested_at")).single().unwrap_or_default(),
            status,
            reviewed_by: row.get::<Option<String>, _>("reviewed_by").map(|s| Uuid::parse_str(&s)).transpose()?,
            reviewed_at: row.get::<Option<i64>, _>("reviewed_at").map(|t| Utc.timestamp_millis_opt(t).single().unwrap_or_default()),
            review_comment: row.get("review_comment"),
        })
    }
}
