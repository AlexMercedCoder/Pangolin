/// Warehouse operations for SqliteStore
use super::SqliteStore;
use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;
use pangolin_core::model::{Warehouse, WarehouseUpdate};

impl SqliteStore {
    pub async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        sqlx::query("INSERT INTO warehouses (id, tenant_id, name, use_sts, storage_config, vending_strategy) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(warehouse.id.to_string())
            .bind(tenant_id.to_string())
            .bind(&warehouse.name)
            .bind(if warehouse.use_sts { 1 } else { 0 })
            .bind(serde_json::to_string(&warehouse.storage_config)?)
            .bind(warehouse.vending_strategy.map(|s| serde_json::to_string(&s).unwrap_or_default()))
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let row = sqlx::query("SELECT id, name, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(&name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let vending_strategy_str: Option<String> = row.get("vending_strategy");
            let vending_strategy = vending_strategy_str
                .and_then(|s| serde_json::from_str(&s).ok());

            Ok(Some(Warehouse {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                name: row.get("name"),
                use_sts: row.get::<i32, _>("use_sts") != 0,
                storage_config: serde_json::from_str(&row.get::<String, _>("storage_config"))?,
                vending_strategy,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_warehouses(&self, tenant_id: Uuid, pagination: Option<crate::PaginationParams>) -> Result<Vec<Warehouse>> {
        let limit = pagination.map(|p| p.limit.unwrap_or(i64::MAX as usize) as i64).unwrap_or(-1);
        let offset = pagination.map(|p| p.offset.unwrap_or(0) as i64).unwrap_or(0);

        let rows = sqlx::query("SELECT id, name, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = ? LIMIT ? OFFSET ?")
            .bind(tenant_id.to_string())
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        let mut warehouses = Vec::new();
        for row in rows {
            let vending_strategy_str: Option<String> = row.get("vending_strategy");
            let vending_strategy = vending_strategy_str
                .and_then(|s| serde_json::from_str(&s).ok());

            warehouses.push(Warehouse {
                id: Uuid::parse_str(&row.get::<String, _>("id"))?,
                tenant_id,
                name: row.get("name"),
                use_sts: row.get::<i32, _>("use_sts") != 0,
                storage_config: serde_json::from_str(&row.get::<String, _>("storage_config"))?,
                vending_strategy,
            });
        }
        Ok(warehouses)
    }

    pub async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: WarehouseUpdate) -> Result<Warehouse> {
        let mut query = String::from("UPDATE warehouses SET ");
        let mut set_clauses = Vec::new();
        
        if updates.name.is_some() {
            set_clauses.push("name = ?");
        }
        if updates.storage_config.is_some() {
            set_clauses.push("storage_config = ?");
        }
        if updates.use_sts.is_some() {
            set_clauses.push("use_sts = ?");
        }
        if updates.vending_strategy.is_some() {
            set_clauses.push("vending_strategy = ?");
        }
        
        if set_clauses.is_empty() {
            return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }
        
        query.push_str(&set_clauses.join(", "));
        query.push_str(" WHERE tenant_id = ? AND name = ?");
        
        let mut q = sqlx::query(&query);
        if let Some(new_name) = &updates.name {
            q = q.bind(new_name);
        }
        if let Some(config) = &updates.storage_config {
            q = q.bind(serde_json::to_string(config)?);
        }
        if let Some(use_sts) = updates.use_sts {
            q = q.bind(use_sts as i32);
        }
        if let Some(vending_strategy) = &updates.vending_strategy {
            q = q.bind(serde_json::to_string(vending_strategy)?);
        }
        q = q.bind(tenant_id.to_string()).bind(&name);
        
        q.execute(&self.pool).await?;

        let new_name = updates.name.unwrap_or(name);
        self.get_warehouse(tenant_id, new_name).await?
            .ok_or_else(|| anyhow::anyhow!("Warehouse not found"))
    }

    pub async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM warehouses WHERE tenant_id = ? AND name = ?")
            .bind(tenant_id.to_string())
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Warehouse '{}' not found", name));
        }
        Ok(())
    }
}
