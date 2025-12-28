use super::PostgresStore;
use anyhow::Result;
use uuid::Uuid;
use sqlx::Row;
use pangolin_core::business_metadata::{AccessRequest, RequestStatus};

impl PostgresStore {
    // Access Request Operations
    pub async fn create_access_request(&self, request: AccessRequest) -> Result<()> {
        let status_str = match request.status {
            RequestStatus::Pending => "Pending",
            RequestStatus::Approved => "Approved",
            RequestStatus::Rejected => "Rejected",
        };

        sqlx::query(
            "INSERT INTO access_requests (id, tenant_id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment) 
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
        )
        .bind(request.id)
        .bind(request.tenant_id)
        .bind(request.user_id)
        .bind(request.asset_id)
        .bind(request.reason)
        .bind(request.requested_at)
        .bind(status_str)
        .bind(request.reviewed_by)
        .bind(request.reviewed_at)
        .bind(request.review_comment)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_access_request(&self, id: Uuid) -> Result<Option<AccessRequest>> {
        let row = sqlx::query("SELECT id, tenant_id, user_id, asset_id, reason, requested_at, status, reviewed_by, reviewed_at, review_comment FROM access_requests WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_access_request(row)?))
        } else {
            Ok(None)
        }
    }

    pub async fn list_access_requests(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<AccessRequest>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(i64::MAX);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query(
            "SELECT ar.id, ar.tenant_id, ar.user_id, ar.asset_id, ar.reason, ar.requested_at, ar.status, ar.reviewed_by, ar.reviewed_at, ar.review_comment FROM access_requests ar
             WHERE ar.tenant_id = $1 ORDER BY ar.requested_at DESC LIMIT $2 OFFSET $3"
        )
        .bind(tenant_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let mut requests = Vec::new();
        for row in rows {
            requests.push(self.row_to_access_request(row)?);
        }
        Ok(requests)
    }

    pub async fn update_access_request(&self, request: AccessRequest) -> Result<()> {
        let status_str = match request.status {
            RequestStatus::Pending => "Pending",
            RequestStatus::Approved => "Approved",
            RequestStatus::Rejected => "Rejected",
        };

        sqlx::query(
            "UPDATE access_requests SET status = $1, reviewed_by = $2, reviewed_at = $3, review_comment = $4 WHERE id = $5"
        )
        .bind(status_str)
        .bind(request.reviewed_by)
        .bind(request.reviewed_at)
        .bind(request.review_comment)
        .bind(request.id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub(crate) fn row_to_access_request(&self, row: sqlx::postgres::PgRow) -> Result<AccessRequest> {
        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "Pending" => RequestStatus::Pending,
            "Approved" => RequestStatus::Approved,
            "Rejected" => RequestStatus::Rejected,
            _ => RequestStatus::Pending,
        };

        Ok(AccessRequest {
            id: row.get("id"),
            tenant_id: row.get("tenant_id"),
            user_id: row.get("user_id"),
            asset_id: row.get("asset_id"),
            reason: row.get("reason"),
            requested_at: row.get("requested_at"),
            status,
            reviewed_by: row.get("reviewed_by"),
            reviewed_at: row.get("reviewed_at"),
            review_comment: row.get("review_comment"),
        })
    }
}
