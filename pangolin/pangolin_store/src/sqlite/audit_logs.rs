/// Audit Log operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::audit::{AuditLogEntry, AuditLogFilter};

impl SqliteStore {
    pub async fn log_audit_event(&self, tenant_id: Uuid, event: AuditLogEntry) -> Result<()> {
        sqlx::query(
            "INSERT INTO audit_logs (
                id, tenant_id, user_id, username, action, resource_type,
                resource_id, resource_name, timestamp, ip_address, user_agent,
                result, error_message, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(event.id.to_string())
        .bind(tenant_id.to_string())
        .bind(event.user_id.map(|u| u.to_string()))
        .bind(&event.username)
        .bind(format!("{:?}", event.action))
        .bind(format!("{:?}", event.resource_type))
        .bind(event.resource_id.map(|u| u.to_string()))
        .bind(&event.resource_name)
        .bind(event.timestamp.timestamp_millis())
        .bind(event.ip_address.as_deref().unwrap_or(""))
        .bind(event.user_agent.as_deref().unwrap_or(""))
        .bind(format!("{:?}", event.result))
        .bind(event.error_message.as_deref().unwrap_or(""))
        .bind(serde_json::to_string(&event.metadata)?)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_audit_events(&self, tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<Vec<AuditLogEntry>> {
        let mut query = String::from(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = ?"
        );
        
        let mut bind_values: Vec<String> = vec![tenant_id.to_string()];
        
        // Build dynamic WHERE clause
        if let Some(ref f) = filter {
            if let Some(user_id) = f.user_id {
                query.push_str(" AND user_id = ?");
                bind_values.push(user_id.to_string());
            }
            if let Some(ref action) = f.action {
                query.push_str(" AND action = ?");
                bind_values.push(format!("{:?}", action));
            }
            if let Some(ref resource_type) = f.resource_type {
                query.push_str(" AND resource_type = ?");
                bind_values.push(format!("{:?}", resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query.push_str(" AND resource_id = ?");
                bind_values.push(resource_id.to_string());
            }
            if let Some(start_time) = f.start_time {
                query.push_str(" AND timestamp >= ?");
                bind_values.push(start_time.timestamp_millis().to_string());
            }
            if let Some(end_time) = f.end_time {
                query.push_str(" AND timestamp <= ?");
                bind_values.push(end_time.timestamp_millis().to_string());
            }
            if let Some(ref result) = f.result {
                query.push_str(" AND result = ?");
                bind_values.push(format!("{:?}", result));
            }
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        // Add pagination
        let limit = filter.as_ref().and_then(|f| f.limit).unwrap_or(100);
        query.push_str(&format!(" LIMIT {}", limit));
        
        if let Some(offset) = filter.as_ref().and_then(|f| f.offset) {
            query.push_str(&format!(" OFFSET {}", offset));
        }
        
        // Build query with dynamic binding
        let mut query_builder = sqlx::query(&query);
        for value in &bind_values {
            query_builder = query_builder.bind(value);
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        
        let mut events = Vec::new();
        for row in rows {
            let ts_millis: i64 = row.get("timestamp");
            
            // Parse enums from strings
            let action_str: String = row.get("action");
            let action = serde_json::from_str(&format!("\"{}\"", action_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditAction::CreateCatalog);
            
            let resource_type_str: String = row.get("resource_type");
            let resource_type = serde_json::from_str(&format!("\"{}\"", resource_type_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::ResourceType::Catalog);
            
            let result_str: String = row.get("result");
            let result = serde_json::from_str(&format!("\"{}\"", result_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditResult::Success);
            
            events.push(AuditLogEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                user_id: row.get::<Option<String>, _>("user_id").and_then(|s| Uuid::parse_str(&s).ok()),
                username: row.get("username"),
                action,
                resource_type,
                resource_id: row.get::<Option<String>, _>("resource_id").and_then(|s| Uuid::parse_str(&s).ok()),
                resource_name: row.get("resource_name"),
                timestamp: chrono::TimeZone::timestamp_millis_opt(&Utc, ts_millis).single().unwrap_or_default(),
                ip_address: {
                    let s: String = row.get("ip_address");
                    if s.is_empty() { None } else { Some(s) }
                },
                user_agent: {
                    let s: String = row.get("user_agent");
                    if s.is_empty() { None } else { Some(s) }
                },
                result,
                error_message: {
                    let s: String = row.get("error_message");
                    if s.is_empty() { None } else { Some(s) }
                },
                metadata: serde_json::from_str(&row.get::<String, _>("metadata")).unwrap_or_default(),
            });
        }
        Ok(events)
    }
    
    pub async fn get_audit_event(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<AuditLogEntry>> {
        let row = sqlx::query(
            "SELECT id, tenant_id, user_id, username, action, resource_type,
             resource_id, resource_name, timestamp, ip_address, user_agent,
             result, error_message, metadata
             FROM audit_logs WHERE tenant_id = ? AND id = ?"
        )
        .bind(tenant_id.to_string())
        .bind(event_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let ts_millis: i64 = row.get("timestamp");
            
            let action_str: String = row.get("action");
            let action = serde_json::from_str(&format!("\"{}\"", action_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditAction::CreateCatalog);
            
            let resource_type_str: String = row.get("resource_type");
            let resource_type = serde_json::from_str(&format!("\"{}\"", resource_type_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::ResourceType::Catalog);
            
            let result_str: String = row.get("result");
            let result = serde_json::from_str(&format!("\"{}\"", result_str.to_lowercase()))
                .unwrap_or(pangolin_core::audit::AuditResult::Success);
            
            Ok(Some(AuditLogEntry {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                user_id: row.get::<Option<String>, _>("user_id").and_then(|s| Uuid::parse_str(&s).ok()),
                username: row.get("username"),
                action,
                resource_type,
                resource_id: row.get::<Option<String>, _>("resource_id").and_then(|s| Uuid::parse_str(&s).ok()),
                resource_name: row.get("resource_name"),
                timestamp: chrono::TimeZone::timestamp_millis_opt(&Utc, ts_millis).single().unwrap_or_default(),
                ip_address: {
                    let s: String = row.get("ip_address");
                    if s.is_empty() { None } else { Some(s) }
                },
                user_agent: {
                    let s: String = row.get("user_agent");
                    if s.is_empty() { None } else { Some(s) }
                },
                result,
                error_message: {
                    let s: String = row.get("error_message");
                    if s.is_empty() { None } else { Some(s) }
                },
                metadata: serde_json::from_str(&row.get::<String, _>("metadata")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn count_audit_events(&self, tenant_id: Uuid, filter: Option<AuditLogFilter>) -> Result<usize> {
        let mut query = String::from("SELECT COUNT(*) as count FROM audit_logs WHERE tenant_id = ?");
        let mut bind_values: Vec<String> = vec![tenant_id.to_string()];
        
        // Build same WHERE clause as list_audit_events
        if let Some(ref f) = filter {
            if let Some(user_id) = f.user_id {
                query.push_str(" AND user_id = ?");
                bind_values.push(user_id.to_string());
            }
            if let Some(ref action) = f.action {
                query.push_str(" AND action = ?");
                bind_values.push(format!("{:?}", action));
            }
            if let Some(ref resource_type) = f.resource_type {
                query.push_str(" AND resource_type = ?");
                bind_values.push(format!("{:?}", resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query.push_str(" AND resource_id = ?");
                bind_values.push(resource_id.to_string());
            }
            if let Some(start_time) = f.start_time {
                query.push_str(" AND timestamp >= ?");
                bind_values.push(start_time.timestamp_millis().to_string());
            }
            if let Some(end_time) = f.end_time {
                query.push_str(" AND timestamp <= ?");
                bind_values.push(end_time.timestamp_millis().to_string());
            }
            if let Some(ref result) = f.result {
                query.push_str(" AND result = ?");
                bind_values.push(format!("{:?}", result));
            }
        }
        
        // Build query with dynamic binding
        let mut query_builder = sqlx::query(&query);
        for value in &bind_values {
            query_builder = query_builder.bind(value);
        }
        
        let row = query_builder.fetch_one(&self.pool).await?;
        let count: i64 = row.get("count");
        Ok(count as usize)
    }
}
