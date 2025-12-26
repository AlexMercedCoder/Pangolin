use super::MemoryStore;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;
use pangolin_core::audit::*;
use async_trait::async_trait;

impl MemoryStore {
    pub(crate) async fn log_audit_event_internal(&self, tenant_id: Uuid, event: pangolin_core::audit::AuditLogEntry) -> Result<()> {
            // Log to tracing
            tracing::info!("AUDIT: {:?}", event);
            // Store in map
            self.audit_events.entry(tenant_id).or_insert_with(Vec::new).push(event);
            Ok(())
        }
    pub(crate) async fn list_audit_events_internal(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<Vec<pangolin_core::audit::AuditLogEntry>> {
            if let Some(events) = self.audit_events.get(&tenant_id) {
                let mut filtered = events.clone();
            
                // Apply filters if provided
                if let Some(f) = filter {
                    filtered.retain(|event| {
                        // Filter by user_id
                        if let Some(user_id) = f.user_id {
                            if event.user_id != Some(user_id) {
                                return false;
                            }
                        }
                    
                        // Filter by action
                        if let Some(ref action) = f.action {
                            if &event.action != action {
                                return false;
                            }
                        }
                    
                        // Filter by resource_type
                        if let Some(ref resource_type) = f.resource_type {
                            if &event.resource_type != resource_type {
                                return false;
                            }
                        }
                    
                        // Filter by resource_id
                        if let Some(resource_id) = f.resource_id {
                            if event.resource_id != Some(resource_id) {
                                return false;
                            }
                        }
                    
                        // Filter by start_time
                        if let Some(start_time) = f.start_time {
                            if event.timestamp < start_time {
                                return false;
                            }
                        }
                    
                        // Filter by end_time
                        if let Some(end_time) = f.end_time {
                            if event.timestamp > end_time {
                                return false;
                            }
                        }
                    
                        // Filter by result
                        if let Some(ref result) = f.result {
                            if &event.result != result {
                                return false;
                            }
                        }
                    
                        true
                    });
                
                    // Apply pagination
                    let offset = f.offset.unwrap_or(0);
                    let limit = f.limit.unwrap_or(100);
                
                    filtered = filtered.into_iter()
                        .skip(offset)
                        .take(limit)
                        .collect();
                }
            
                Ok(filtered)
            } else {
                Ok(vec![])
            }
        }
    pub(crate) async fn get_audit_event_internal(&self, tenant_id: Uuid, event_id: Uuid) -> Result<Option<pangolin_core::audit::AuditLogEntry>> {
            if let Some(events) = self.audit_events.get(&tenant_id) {
                Ok(events.iter().find(|e| e.id == event_id).cloned())
            } else {
                Ok(None)
            }
        }
    pub(crate) async fn count_audit_events_internal(&self, tenant_id: Uuid, filter: Option<pangolin_core::audit::AuditLogFilter>) -> Result<usize> {
            if let Some(events) = self.audit_events.get(&tenant_id) {
                if let Some(f) = filter {
                    let count = events.iter().filter(|event| {
                        // Same filtering logic as list_audit_events
                        if let Some(user_id) = f.user_id {
                            if event.user_id != Some(user_id) {
                                return false;
                            }
                        }
                        if let Some(ref action) = f.action {
                            if &event.action != action {
                                return false;
                            }
                        }
                        if let Some(ref resource_type) = f.resource_type {
                            if &event.resource_type != resource_type {
                                return false;
                            }
                        }
                        if let Some(resource_id) = f.resource_id {
                            if event.resource_id != Some(resource_id) {
                                return false;
                            }
                        }
                        if let Some(start_time) = f.start_time {
                            if event.timestamp < start_time {
                                return false;
                            }
                        }
                        if let Some(end_time) = f.end_time {
                            if event.timestamp > end_time {
                                return false;
                            }
                        }
                        if let Some(ref result) = f.result {
                            if &event.result != result {
                                return false;
                            }
                        }
                        true
                    }).count();
                    Ok(count)
                } else {
                    Ok(events.len())
                }
            } else {
                Ok(0)
            }
        }
}
