use super::PostgresStore;
use anyhow::Result;
use pangolin_core::model::Warehouse;
use uuid::Uuid;
use sqlx::Row;

impl PostgresStore {
    // Warehouse Operations
    pub async fn create_warehouse(&self, tenant_id: Uuid, warehouse: Warehouse) -> Result<()> {
        sqlx::query("INSERT INTO warehouses (id, tenant_id, name, use_sts, storage_config, vending_strategy) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(warehouse.id)
            .bind(tenant_id)
            .bind(&warehouse.name)
            .bind(warehouse.use_sts)
            .bind(serde_json::to_value(&warehouse.storage_config)?)
            .bind(serde_json::to_value(&warehouse.vending_strategy)?)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_warehouse(&self, tenant_id: Uuid, name: String) -> Result<Option<Warehouse>> {
        let row: Option<sqlx::postgres::PgRow> = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(Warehouse {
                id: row.get("id"),
                name: row.get("name"),
                tenant_id: row.get("tenant_id"),
                use_sts: row.try_get("use_sts").unwrap_or(false),
                storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
                vending_strategy: serde_json::from_value(row.get("vending_strategy")).ok(),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn list_warehouses(&self, tenant_id: Uuid) -> Result<Vec<Warehouse>> {
        let rows = sqlx::query("SELECT id, name, tenant_id, use_sts, storage_config, vending_strategy FROM warehouses WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await?;

        let mut warehouses = Vec::new();
        for row in rows {
            warehouses.push(Warehouse {
                id: row.get("id"),
                name: row.get("name"),
                tenant_id: row.get("tenant_id"),
                use_sts: row.try_get("use_sts").unwrap_or(false),
                storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
                vending_strategy: serde_json::from_value(row.get("vending_strategy")).ok(),
            });
        }
        Ok(warehouses)
    }

    pub async fn update_warehouse(&self, tenant_id: Uuid, name: String, updates: pangolin_core::model::WarehouseUpdate) -> Result<Warehouse> {
        let mut query = String::from("UPDATE warehouses SET ");
        let mut set_clauses = Vec::new();
        let mut bind_count = 1;

        if updates.use_sts.is_some() {
            set_clauses.push(format!("use_sts = ${}", bind_count));
            bind_count += 1;
        }
        if updates.storage_config.is_some() {
            set_clauses.push(format!("storage_config = ${}", bind_count));
            bind_count += 1;
        }
        if updates.vending_strategy.is_some() {
            set_clauses.push(format!("vending_strategy = ${}", bind_count));
            bind_count += 1;
        }

        if set_clauses.is_empty() {
             return self.get_warehouse(tenant_id, name).await?
                .ok_or_else(|| anyhow::anyhow!("Warehouse not found"));
        }

        query.push_str(&set_clauses.join(", "));
        query.push_str(&format!(" WHERE tenant_id = ${} AND name = ${} RETURNING id, name, tenant_id, use_sts, storage_config, vending_strategy", bind_count, bind_count + 1));

        let mut q = sqlx::query(&query);
        if let Some(use_sts) = &updates.use_sts {
            q = q.bind(use_sts);
        }
        if let Some(storage_config) = &updates.storage_config {
            q = q.bind(serde_json::to_value(storage_config)?);
        }
        if let Some(vending_strategy) = &updates.vending_strategy {
             q = q.bind(serde_json::to_value(vending_strategy)?);
        }
        q = q.bind(tenant_id).bind(&name);

        let row: sqlx::postgres::PgRow = q.fetch_one(&self.pool).await?;

        Ok(Warehouse {
            id: row.get("id"),
            name: row.get("name"),
            tenant_id: row.get("tenant_id"),
            use_sts: row.try_get("use_sts").unwrap_or(false),
            storage_config: serde_json::from_value(row.get("storage_config")).unwrap_or_default(),
            vending_strategy: serde_json::from_value(row.get("vending_strategy")).ok(),
        })
    }

    pub async fn delete_warehouse(&self, tenant_id: Uuid, name: String) -> Result<()> {
        let result = sqlx::query("DELETE FROM warehouses WHERE tenant_id = $1 AND name = $2")
            .bind(tenant_id)
            .bind(&name)
            .execute(&self.pool)
            .await?;
        
        if result.rows_affected() == 0 {
            return Err(anyhow::anyhow!("Warehouse '{}' not found", name));
        }
        Ok(())
    }
}
