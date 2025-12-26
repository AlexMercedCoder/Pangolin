use super::PostgresStore;
use anyhow::Result;
use pangolin_core::audit::AuditLogEntry;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Audit Operations
    pub async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_logs (
                id, tenant_id, user_id, username, action, resource_type,
                resource_id, resource_name, timestamp, ip_address, user_agent,
                result, error_message, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"
        )
        .bind(event.id)
        .bind(tenant_id)
        .bind(event.user_id)
        .bind(event.username)
        .bind(event.action)
        .bind(event.resource_type)
        .bind(event.resource_id)
        .bind(event.resource_name)
        .bind(event.timestamp)
        .bind(event.ip_address)
        .bind(event.user_agent)
        .bind(event.result)
        .bind(event.error_message)
        .bind(event.metadata)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        let mut query = String::from(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = $1"
        );
        
        let mut conditions = Vec::new();
        let mut param_count = 2;
        
        // Build dynamic WHERE clause
        if let Some(ref f) = filter {
            if f.user_id.is_some() {
                conditions.push(format!("user_id = ${}", param_count));
                param_count += 1;
            }
            if f.action.is_some() {
                conditions.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if f.resource_type.is_some() {
                conditions.push(format!("resource_type = ${}", param_count));
                param_count += 1;
            }
            if f.resource_id.is_some() {
                conditions.push(format!("resource_id = ${}", param_count));
                param_count += 1;
            }
            if f.start_time.is_some() {
                conditions.push(format!("timestamp >= ${}", param_count));
                param_count += 1;
            }
            if f.end_time.is_some() {
                conditions.push(format!("timestamp <= ${}", param_count));
                param_count += 1;
            }
            if f.result.is_some() {
                conditions.push(format!("result = ${}", param_count));
                param_count += 1;
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        // Add pagination
        if let Some(ref f) = filter {
            if let Some(limit) = f.limit {
                query.push_str(&format!(" LIMIT {}", limit));
            } else {
                query.push_str(" LIMIT 100");
            }
            if let Some(offset) = f.offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        } else {
            query.push_str(" LIMIT 100");
        }
        
        // Build and execute query with dynamic binding
        let mut query_builder = sqlx::query(&query).bind(tenant_id);
        
        if let Some(f) = filter {
            if let Some(user_id) = f.user_id {
                query_builder = query_builder.bind(user_id);
            }
            if let Some(action) = f.action {
                query_builder = query_builder.bind(action);
            }
            if let Some(resource_type) = f.resource_type {
                query_builder = query_builder.bind(resource_type);
            }
            if let Some(resource_id) = f.resource_id {
                query_builder = query_builder.bind(resource_id);
            }
            if let Some(start_time) = f.start_time {
                query_builder = query_builder.bind(start_time);
            }
            if let Some(end_time) = f.end_time {
                query_builder = query_builder.bind(end_time);
            }
            if let Some(result) = f.result {
                query_builder = query_builder.bind(result);
            }
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(AuditLogEntry {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                action: row.get("action"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                resource_name: row.get("resource_name"),
                timestamp: row.get("timestamp"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                result: row.get("result"),
                error_message: row.get("error_message"),
                metadata: row.get("metadata"),
            });
        }
        Ok(events)
    }
    
    pub async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = $1 AND id = $2"
        )
        .bind(tenant_id)
        .bind(event_id)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(AuditLogEntry {
                id: row.get("id"),
                tenant_id: row.get("tenant_id"),
                user_id: row.get("user_id"),
                username: row.get("username"),
                action: row.get("action"),
                resource_type: row.get("resource_type"),
                resource_id: row.get("resource_id"),
                resource_name: row.get("resource_name"),
                timestamp: row.get("timestamp"),
                ip_address: row.get("ip_address"),
                user_agent: row.get("user_agent"),
                result: row.get("result"),
                error_message: row.get("error_message"),
                metadata: row.get("metadata"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = $1");
        
        let mut conditions = Vec::new();
        let mut param_count = 2;
        
        // Build same WHERE clause as list_audit_events
        if let Some(ref f) = filter {
            if f.user_id.is_some() {
                conditions.push(format!("user_id = ${}", param_count));
                param_count += 1;
            }
            if f.action.is_some() {
                conditions.push(format!("action = ${}", param_count));
                param_count += 1;
            }
            if f.resource_type.is_some() {
                conditions.push(format!("resource_type = ${}", param_count));
                param_count += 1;
            }
            if f.resource_id.is_some() {
                conditions.push(format!("resource_id = ${}", param_count));
                param_count += 1;
            }
            if f.start_time.is_some() {
                conditions.push(format!("timestamp >= ${}", param_count));
                param_count += 1;
            }
            if f.end_time.is_some() {
                conditions.push(format!("timestamp <= ${}", param_count));
                param_count += 1;
            }
            if f.result.is_some() {
                conditions.push(format!("result = ${}", param_count));
                param_count += 1;
            }
        }
        
        if !conditions.is_empty() {
            query.push_str(" AND ");
            query.push_str(&conditions.join(" AND "));
        }
        
        let mut query_builder = sqlx::query_scalar::<_, i64>(&query).bind(tenant_id);
        
        if let Some(f) = filter {
            if let Some(user_id) = f.user_id {
                query_builder = query_builder.bind(user_id);
            }
            if let Some(action) = f.action {
                query_builder = query_builder.bind(action);
            }
            if let Some(resource_type) = f.resource_type {
                query_builder = query_builder.bind(resource_type);
            }
            if let Some(resource_id) = f.resource_id {
                query_builder = query_builder.bind(resource_id);
            }
            if let Some(start_time) = f.start_time {
                query_builder = query_builder.bind(start_time);
            }
            if let Some(end_time) = f.end_time {
                query_builder = query_builder.bind(end_time);
            }
            if let Some(result) = f.result {
                query_builder = query_builder.bind(result);
            }
        }
        
        let count = query_builder.fetch_one(&self.pool).await?;
        Ok(count as usize)
    }
}
