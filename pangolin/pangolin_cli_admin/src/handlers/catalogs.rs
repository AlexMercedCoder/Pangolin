use pangolin_cli_common::client::PangolinClient;
use pangolin_cli_common::error::CliError;
use pangolin_cli_common::utils::print_table;
use serde_json::Value;
use pangolin_cli_common::optimization_types::CatalogSummary;

pub async fn handle_list_catalogs(client: &PangolinClient) -> Result<(), CliError> {
     let res = client.get("/api/v1/catalogs").await?;
    if !res.status().is_success() { return Err(CliError::ApiError(format!("Error: {}", res.status()))); }
    let items: Vec<Value> = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    let rows = items.iter().map(|i| vec![
        i["name"].as_str().unwrap_or("").to_string(),
        i["type"].as_str().unwrap_or("").to_string()
    ]).collect();
    print_table(vec!["Name", "Type"], rows);
    Ok(())
}

pub async fn handle_create_catalog(client: &PangolinClient, name: String, warehouse: String) -> Result<(), CliError> {
    if client.config.tenant_id.is_none() {
        return Err(CliError::ApiError("Root user cannot create catalogs. Please login as Tenant Admin.".to_string()));
    }

    let body = serde_json::json!({
        "name": name,
        "warehouse": warehouse,
        "type": "pangea" // defaulting to internal type
    });
    
    let res = client.post("/api/v1/catalogs", &body).await?;
    if !res.status().is_success() { 
        let s = res.status();
        let t = res.text().await.unwrap_or_default();
        return Err(CliError::ApiError(format!("{} - {}", s, t))); 
    }
    println!("✅ Catalog '{}' created.", name);
    Ok(())
}

pub async fn handle_delete_catalog(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let res = client.delete(&format!("/api/v1/catalogs/{}", name)).await?;
    if !res.status().is_success() {
         let status = res.status();
         let t = res.text().await.unwrap_or_default();
         return Err(CliError::ApiError(format!("Failed to delete catalog: {} - {}", status, t)));
    }
    println!("✅ Catalog '{}' deleted.", name);
    Ok(())
}

pub async fn handle_update_catalog(
    client: &PangolinClient,
    id: String,
    name: Option<String>,
) -> Result<(), CliError> {
    let mut payload = serde_json::json!({});
    
    if let Some(n) = name {
        payload["name"] = serde_json::Value::String(n);
    }
    
    let res = client.put(&format!("/api/v1/catalogs/{}", id), &payload).await?;
    
    if !res.status().is_success() {
        let status = res.status();
        let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(CliError::ApiError(format!("Failed to update catalog ({}): {}", status, error_text)));
    }
    
    println!("✅ Catalog updated successfully!");
    
    Ok(())
}

pub async fn handle_catalog_summary(client: &PangolinClient, name: String) -> Result<(), CliError> {
    let url = format!("/api/v1/catalogs/{}/summary", name);
    let res = client.get(&url).await?;
    
    if !res.status().is_success() {
        return Err(CliError::ApiError(format!("Failed to get catalog summary: {}", res.status())));
    }
    
    let summary: CatalogSummary = res.json().await.map_err(|e| CliError::ApiError(e.to_string()))?;
    
    println!("Catalog: {}", summary.name);
    println!("  Namespaces: {}", summary.namespace_count);
    println!("  Tables:     {}", summary.table_count);
    println!("  Branches:   {}", summary.branch_count);
    if let Some(location) = summary.storage_location {
        println!("  Storage:    {}", location);
    }
    
    Ok(())
}
