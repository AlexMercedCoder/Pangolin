use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use serde_json::Value;
use pangolin_cli_common::optimization_types::DashboardStats;

pub async fn handle_get_system_settings(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/config/settings").await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to get system settings ({}): {}", status, error_text)));
    }
    
    let settings: Value = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    println!("{}", serde_json::to_string_pretty(&settings).unwrap());
    Ok(())
}

pub async fn handle_update_system_settings(
    client: &PangolinClient,
    allow_public_signup: Option<bool>,
    default_warehouse_bucket: Option<String>,
    default_retention_days: Option<i32>,
) -> Result<(), CliError> {
    let mut payload = serde_json::Map::new();
    
    if let Some(signup) = allow_public_signup {
        payload.insert("allow_public_signup".to_string(), Value::Bool(signup));
    }
    if let Some(bucket) = default_warehouse_bucket {
        payload.insert("default_warehouse_bucket".to_string(), Value::String(bucket));
    }
    if let Some(days) = default_retention_days {
        payload.insert("default_retention_days".to_string(), Value::Number(days.into()));
    }
    
    let res = client.put("/api/v1/config/settings", &Value::Object(payload)).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update system settings ({}): {}", status, error_text)));
    }
    
    println!("✅ System settings updated successfully!");
    Ok(())
}

pub async fn handle_stats(client: &PangolinClient) -> Result<(), CliError> {
    let res = client.get("/api/v1/dashboard/stats").await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get stats: {}", res.status())));
    }
    
    let stats: DashboardStats = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Dashboard Statistics (Scope: {})", stats.scope);
    println!("=====================================");
    
    // Show tenants count for Root users (system scope)
    if stats.scope == "system" {
        if let Some(count) = stats.tenants_count {
            println!("  Tenants:     {}", count);
        }
    }
    
    println!("  Catalogs:    {}", stats.catalogs_count);
    println!("  Warehouses:  {}", stats.warehouses_count);
    println!("  Namespaces:  {}", stats.namespaces_count);
    println!("  Tables:      {}", stats.tables_count);
    println!("  Users:       {}", stats.users_count);
    println!("  Branches:    {}", stats.branches_count);
    
    Ok(())
}
